use toyjs::runtime::JsRuntime;
use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("=== ToyJS Runtime with Event Loop ===\n");

    let mut runtime = JsRuntime::new();
    let event_loop = runtime.run_event_loop();

    println!("--- Test 1: setTimeout ---");
    let timer_code = r#"
        print("Setting timeout for 1000ms...");
        setTimeout(() => {
            print("Timer fired!");
        }, 1000);
        print("Timer scheduled");
    "#;
    runtime.execute_script_module(timer_code);

    for _ in 0..20 {
        runtime.process_callbacks();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("\n--- Test 2: Simple fetch ---");
    let fetch_code = r#"
        print("Fetching from httpbin.org...");

        fetch("https://httpbin.org/get")
            .then(response => response.text())
            .then(body => {
                print("Fetch successful! Response length: " + body.length);
            })
            .catch(err => {
                print("Fetch error: " + err);
            });

        print("Fetch initiated, waiting for response...");
    "#;
    runtime.execute_script_module(fetch_code);

    for _ in 0..50 {
        runtime.process_callbacks();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("\n--- Shutting down ---");
    runtime.shutdown();
    event_loop.await.unwrap();
    println!("Done!");
}
