use std::sync::Once;
use v8;

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

        let context = {
            let handle_scope = std::pin::pin!(v8::HandleScope::new(&mut isolate));
            let mut scope = handle_scope.init();
            let context = v8::Context::new(&scope, Default::default());

            let scope = &mut v8::ContextScope::new(&mut scope, context);
            Self::setup_bindings(scope);

            v8::Global::new(scope, context)
        };

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
}

fn print_cb(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut _retval: v8::ReturnValue,
) {
    let val = args.get(0).to_string(scope).unwrap();
    println!("JS: {}", val.to_rust_string_lossy(scope));
}

fn add_cb(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let a = args.get(0).number_value(scope).unwrap_or(0.0);
    let b = args.get(1).number_value(scope).unwrap_or(0.0);
    retval.set(v8::Number::new(scope, a + b).into());
}
