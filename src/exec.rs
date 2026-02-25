use toyjs::runtime::JsRuntime;
use std::path::Path;
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: exec <path_to_js>");
        std::process::exit(1);
    }

    let js_path = Path::new(&args[1]);
    if !js_path.exists() {
        eprintln!("Error: File not found: {}", args[1]);
        std::process::exit(1);
    }

    let mut runtime = JsRuntime::new();
    let event_loop = runtime.run_event_loop();

    match runtime.execute_module(js_path) {
        Ok(result) => {
            if result != "undefined" && !result.is_empty() {
                println!("Result: {}", result);
            }
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    // Give some time for pending callbacks to process if any
    // In a real runtime, we'd wait until the event loop is actually empty
    // For this toy runtime, we'll just poll for a bit
    for _ in 0..50 {
        runtime.process_callbacks();
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    runtime.shutdown();
    let _ = event_loop.await;

    Ok(())
}
