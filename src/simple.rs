fn main() -> anyhow::Result<()> {
  let platform = v8::new_default_platform(0, false).make_shared();
  v8::V8::initialize_platform(platform);
  v8::V8::initialize();
  {
      let mut isolate = v8::Isolate::new(v8::CreateParams::default());
      let scope = std::pin::pin!(v8::HandleScope::new(&mut isolate));
      let mut handle_scope = scope.init();
      let context = v8::Context::new(&handle_scope, Default::default());
      let mut context_scope = v8::ContextScope::new(&mut handle_scope, context);
      let src= "`the answer is ${6*7}`";
      let code = v8::String::new(&context_scope, src).unwrap();
      let script = v8::Script::compile(&context_scope, code, None).unwrap();
      let v8_value = script.run(&context_scope).unwrap();
      let result = v8_value.to_string(&context_scope).unwrap();
      println!("{}", result.to_rust_string_lossy(&context_scope));
  } // destroy resources before calling v8 dispose
  unsafe {
    v8::V8::dispose();
  }
  v8::V8::dispose_platform();
  Ok(())
}
