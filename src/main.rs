mod runtime;
use runtime::JsRuntime;

fn main() {
    let mut runtime = JsRuntime::new();
    let code = r#"
        print("Hello from JS!");
        let sum = add(10, 20);
        print("10 + 20 = " + sum);
        "Finished"
    "#;
    println!("Running JS...");
    let result = runtime.execute_script(code);
    println!("Result: {}", result);
}
