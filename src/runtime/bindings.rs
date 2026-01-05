
pub fn host_import_module_dynamically_callback<'s>(
    scope: &mut v8::PinScope,
    _host_defined_options: v8::Local<'s, v8::Data>,
    resource_name: v8::Local<'s, v8::Value>,
    specifier: v8::Local<'s, v8::String>,
    _import_attributes: v8::Local<'s, v8::FixedArray>,
) -> Option<v8::Local<'s, v8::Promise>> {
    println!(
        "Dynamic import requested: resource_name = {:?}, specifier = {:?}",
        resource_name.to_rust_string_lossy(scope),
        specifier.to_rust_string_lossy(scope)
    );
    None
}

pub fn print_cb(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    mut _retval: v8::ReturnValue,
) {
    let val = args.get(0).to_string(scope).unwrap();
    println!("JS: {}", val.to_rust_string_lossy(scope));
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
