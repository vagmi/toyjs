use std::sync::Once;
use v8;
use bindings::{print_cb, add_cb};

mod bindings;

static INIT: Once = Once::new();

pub fn init_v8() {
    INIT.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}

pub struct JsRuntime {
    isolate: v8::OwnedIsolate,
    context: v8::Global<v8::Context>,
}

impl JsRuntime {
    pub fn new() -> Self {
        init_v8();
        let params = v8::CreateParams::default();
        let mut isolate = v8::Isolate::new(params);
        isolate.set_host_import_module_dynamically_callback(bindings::host_import_module_dynamically_callback);

        let context = {
            let handle_scope = std::pin::pin!(v8::HandleScope::new(&mut isolate));
            let mut scope = handle_scope.init();
            let context = v8::Context::new(&scope, Default::default());

            let scope = &mut v8::ContextScope::new(&mut scope, context);
            Self::setup_bindings(scope);

            v8::Global::new(scope, context)
        };

        // Initialize global module loader
        let _ = crate::modules::FsModuleLoader::global();

        Self { isolate, context }
    }

    fn setup_bindings(scope: &mut v8::PinScope) {
        let global = scope.get_current_context().global(scope);

        let name = v8::String::new(scope, "print").unwrap();

        let func = v8::FunctionTemplate::new(scope, print_cb);
        let func = func.get_function(scope).unwrap();
        global.set(scope, name.into(), func.into());

        let name = v8::String::new(scope, "add").unwrap();
        let func = v8::FunctionTemplate::new(scope, add_cb);
        let func = func.get_function(scope).unwrap();
        global.set(scope, name.into(), func.into());
    }

    pub fn execute_script(&mut self, code: &str) -> String {
        let handle_scope = std::pin::pin!(v8::HandleScope::new(&mut self.isolate));
        let scope = &mut handle_scope.init();
        let context = v8::Local::new(scope, &self.context);
        let scope = &mut v8::ContextScope::new(scope, context);

        let source = v8::String::new(scope, code).unwrap();
        let script = match v8::Script::compile(scope, source, None) {
            Some(script) => script,
            None => return "Error: Compilation failed".to_string(),
        };

        let result = match script.run(scope) {
            Some(result) => result,
            None => return "Error: Execution failed".to_string(),
        };

        let result = result.to_string(scope).unwrap();
        result.to_rust_string_lossy(scope)
    }
    fn module_resolver<'a>(
        context: v8::Local<'a, v8::Context>,
        specifier: v8::Local<'a, v8::String>,
        _import_attributes: v8::Local<'a, v8::FixedArray>,
        referrer: v8::Local<'a, v8::Module>,
    ) -> Option<v8::Local<'a, v8::Module>> {
        let scope_storage = std::pin::pin!(unsafe { v8::CallbackScope::new(context) });
        let scope = &mut scope_storage.init();
        let specifier_str = specifier.to_rust_string_lossy(scope);
        let referrer_hash = referrer.get_identity_hash();

        println!("Module resolver called:");
        println!("  Specifier: {}", specifier_str);
        println!("  Referrer hash: {}", referrer_hash);

        let loader = crate::modules::FsModuleLoader::global();

        let base_path = {
            let loader_guard = loader.lock().unwrap();
            loader_guard.get_path_by_hash(referrer_hash).cloned()
        };

        let base_path = match base_path {
            Some(path) => path,
            None => {
                println!("  -> Could not find referrer path");
                return None;
            }
        };

        println!("  Referrer path: {}", base_path);

        let mut resolved_path = crate::modules::FsModuleLoader::resolve_path(&base_path, &specifier_str)?;

        if !std::path::Path::new(&resolved_path).exists() {
            resolved_path = format!("{}.js", resolved_path);
        }

        println!("  Resolved path: {}", resolved_path);

        {
            let loader_guard = loader.lock().unwrap();
            if let Some(global_module) = loader_guard.get_module(&resolved_path) {
                println!("  -> Returning cached module");
                return Some(v8::Local::new(scope, global_module));
            }
        }

        println!("  -> Loading module from file");
        let code = match std::fs::read_to_string(&resolved_path) {
            Ok(code) => code,
            Err(e) => {
                println!("  -> Failed to read file: {}", e);
                return None;
            }
        };

        let source_str = v8::String::new(scope, &code)?;
        let origin = v8::ScriptOrigin::new(
            scope,
            v8::String::new(scope, &resolved_path)?.into(),
            0,
            0,
            false,
            123,
            None,
            false,
            false,
            true, // is_module
            None,
        );
        let mut source = v8::script_compiler::Source::new(source_str, Some(&origin));

        let module = v8::script_compiler::compile_module(scope, &mut source)?;
        let module_hash = module.get_identity_hash();

        // Store in global cache
        let global_module = v8::Global::new(scope, module);
        {
            let mut loader_guard = loader.lock().unwrap();
            loader_guard.store_module(resolved_path.clone(), global_module, module_hash);
        }

        println!("  -> Compiled and cached module");
        Some(module)
    }

    pub fn execute_script_module(&mut self, code: &str) -> String {
        let handle_scope = std::pin::pin!(v8::HandleScope::new(&mut self.isolate));
        let scope = &mut handle_scope.init();
        let context = v8::Local::new(scope, &self.context);
        let scope = &mut v8::ContextScope::new(scope, context);
        let tc_scope_storage = std::pin::pin!(v8::TryCatch::new(scope));
        let tc_scope = &mut tc_scope_storage.init();

        let source_str = v8::String::new(tc_scope, code).unwrap();
        let origin = v8::ScriptOrigin::new(
            tc_scope,
            v8::String::new(tc_scope, "main.js").unwrap().into(),
            0,
            0,
            false,
            123,
            None,
            false,
            false,
            true, // is_module
            None, // host_defined_options
        );
        let mut source = v8::script_compiler::Source::new(source_str, Some(&origin));

        println!("Compiling module...");
        let module = match v8::script_compiler::compile_module(tc_scope, &mut source) {
            Some(module) => {
                println!("Module compiled successfully");
                module
            }
            None => {
                let exception = tc_scope.exception().unwrap();
                let exception_str = exception.to_string(tc_scope).unwrap();
                return format!("Error: Module compilation failed: {}", exception_str.to_rust_string_lossy(tc_scope));
            }
        };

        let module_hash = module.get_identity_hash();
        let global_module = v8::Global::new(tc_scope, module);
        {
            let loader = crate::modules::FsModuleLoader::global();
            let mut loader_guard = loader.lock().unwrap();
            // Use the current directory as the base path for the main module
            let cwd = std::env::current_dir().unwrap().to_string_lossy().to_string();
            let main_path = format!("{}/main.js", cwd);
            loader_guard.store_module(main_path, global_module, module_hash);
        }

        println!("Instantiating module...");
        let status = module.instantiate_module(tc_scope, Self::module_resolver);
        if status.is_none() {
            let msg = if tc_scope.has_caught() {
                let exception = tc_scope.exception().unwrap();
                let exception_str = exception.to_string(tc_scope).unwrap();
                format!("Exception: {}", exception_str.to_rust_string_lossy(tc_scope))
            } else {
                "Unknown error (no exception caught)".to_string()
            };
            return format!("Error: Module instantiation failed - {}", msg);
        }
        println!("Module instantiated successfully");

        println!("Evaluating module...");
        let result = match module.evaluate(tc_scope) {
            Some(result) => {
                println!("Module evaluated successfully");
                result
            }
            None => {
                let msg = if tc_scope.has_caught() {
                    let exception = tc_scope.exception().unwrap();
                    let exception_str = exception.to_string(tc_scope).unwrap();
                    format!("Exception: {}", exception_str.to_rust_string_lossy(tc_scope))
                } else {
                    "Unknown error (no exception caught)".to_string()
                };
                return format!("Error: Module execution failed - {}", msg);
            }
        };

        let result = result.to_string(tc_scope).unwrap();
        result.to_rust_string_lossy(tc_scope)
    }
}

