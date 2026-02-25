pub fn main() {
  let platform = v8::new_default_platform(0, false).make_shared();
  v8::V8::initialize_platform(platform);
  v8::V8::initialize();
  {
      let mut isolate = v8::Isolate::new(v8::CreateParams::default());
      let scope = std::pin::pin!(v8::HandleScope::new(&mut isolate));
      let mut handle_scope = scope.init();
      let context = v8::Context::new(&handle_scope, Default::default());
      let mut context_scope = v8::ContextScope::new(&mut handle_scope, context);

      create_add_method(&mut context_scope);

      let code = v8::String::new(&context_scope, "`the answer is ${__add(16, 26)}`").unwrap();
      let script = v8::Script::compile(&context_scope, code, None).unwrap();
      let v8_value = script.run(&context_scope).unwrap();
      let result = v8_value.to_string(&context_scope).unwrap();
      println!("{}", result.to_rust_string_lossy(&context_scope));
  } // destroy resources before calling v8 dispose
  unsafe {
    v8::V8::dispose();
  }
  v8::V8::dispose_platform();
}


pub fn add_cb(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    let a = args.get(0).number_value(scope).unwrap_or(0.0);
    let b = args.get(1).number_value(scope).unwrap_or(0.0);
    retval.set(v8::Number::new(scope, a + b).into());
}

pub fn create_add_method(scope: &mut v8::PinScope) {
    let global = scope.get_current_context().global(scope);
    let name = v8::String::new(scope, "__add").unwrap();
    let func = v8::FunctionTemplate::new(scope, add_cb);
    let func = func.get_function(scope).unwrap();
    global.set(scope, name.into(), func.into());
}
