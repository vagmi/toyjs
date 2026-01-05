mod runtime;
mod modules;
use runtime::JsRuntime;

fn main() {
    let mut runtime = JsRuntime::new();
    let code = r#"
        import {sum} from './math.js';

        print("Hello from JS Module!");
        let value = add(10, 20);
        print("10 + 20 = " + value);
        print(`Using imported sum: 20 + 22 = ${sum(20, 22)}`);

        export default "Finished"
    "#;
    println!("Running JS...");
    let result = runtime.execute_script_module(code);
    println!("Result: {}", result);
}
