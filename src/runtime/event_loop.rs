use super::{CallbackId, CallbackMessage, SchedulerMessage};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc;

/// Event loop that runs in tokio async context
/// Processes scheduler messages and spawns async tasks
pub async fn run_event_loop(
    mut scheduler_rx: mpsc::UnboundedReceiver<SchedulerMessage>,
    callback_tx: mpsc::UnboundedSender<CallbackMessage>,
) {
    // Track running tasks so we can cancel them
    let mut running_tasks: HashMap<CallbackId, tokio::task::JoinHandle<()>> = HashMap::new();

    println!("Event loop started");

    while let Some(msg) = scheduler_rx.recv().await {
        match msg {
            SchedulerMessage::ScheduleTimeout(id, delay_ms) => {
                println!("Scheduling timeout: id={}, delay={}ms", id, delay_ms);
                let tx = callback_tx.clone();
                let handle = tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    let _ = tx.send(CallbackMessage::ExecuteTimeout(id));
                });
                running_tasks.insert(id, handle);
            }
            SchedulerMessage::ScheduleInterval(id, interval_ms) => {
                println!("Scheduling interval: id={}, interval={}ms", id, interval_ms);
                let tx = callback_tx.clone();
                let handle = tokio::spawn(async move {
                    let mut interval = tokio::time::interval(Duration::from_millis(interval_ms));
                    interval.tick().await; // Skip first tick

                    loop {
                        interval.tick().await;
                        if tx.send(CallbackMessage::ExecuteInterval(id)).is_err() {
                            break;
                        }
                    }
                });
                running_tasks.insert(id, handle);
            }
            SchedulerMessage::ClearTimer(id) => {
                println!("Clearing timer: id={}", id);
                if let Some(handle) = running_tasks.remove(&id) {
                    handle.abort();
                }
            }
            SchedulerMessage::Fetch(id, url) => {
                println!("Fetching: id={}, url={}", id, url);
                let tx = callback_tx.clone();
                tokio::spawn(async move {
                    match reqwest::get(&url).await {
                        Ok(response) => {
                            match response.text().await {
                                Ok(body) => {
                                    println!("Fetch success: id={}", id);
                                    let _ = tx.send(CallbackMessage::FetchSuccess(id, body));
                                }
                                Err(e) => {
                                    println!("Fetch error (text): id={}, error={}", id, e);
                                    let _ = tx.send(CallbackMessage::FetchError(id, e.to_string()));
                                }
                            }
                        }
                        Err(e) => {
                            println!("Fetch error (request): id={}, error={}", id, e);
                            let _ = tx.send(CallbackMessage::FetchError(id, e.to_string()));
                        }
                    }
                });
            }
            SchedulerMessage::Shutdown => {
                println!("Event loop shutting down");
                // Abort all running tasks
                for (_, handle) in running_tasks.drain() {
                    handle.abort();
                }
                break;
            }
        }
    }

    println!("Event loop stopped");
}
