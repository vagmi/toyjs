use super::SchedulerMessage;
use tokio::sync::mpsc;
use v8;

struct FetchState {
    scheduler_tx: mpsc::UnboundedSender<SchedulerMessage>,
}

fn get_fetch_state<'a>(scope: &mut v8::PinScope) -> Option<&'a FetchState> {
    let global = scope.get_current_context().global(scope);
    let state_key = v8::String::new(scope, "__fetchState").unwrap();
    let state_val = global.get(scope, state_key.into())?;

    if !state_val.is_external() {
        return None;
    }

    let external: v8::Local<v8::External> = state_val.try_into().ok()?;
    let state_ptr = external.value() as *const FetchState;
    Some(unsafe { &*state_ptr })
}

pub fn setup_fetch(
    scope: &mut v8::PinScope,
    scheduler_tx: mpsc::UnboundedSender<SchedulerMessage>,
) {
    let global = scope.get_current_context().global(scope);

    let state = FetchState { scheduler_tx };
    let state_ptr = Box::into_raw(Box::new(state)) as *mut std::ffi::c_void;
    let external = v8::External::new(scope, state_ptr);
    let state_key = v8::String::new(scope, "__fetchState").unwrap();
    global.set(scope, state_key.into(), external.into());

    let native_fetch = v8::Function::new(
        scope,
        |scope: &mut v8::PinScope,
         args: v8::FunctionCallbackArguments,
         mut _retval: v8::ReturnValue| {
            if args.length() < 2 {
                return;
            }

            let id = args.get(0).number_value(scope).unwrap_or(0.0) as u64;
            let url = args
                .get(1)
                .to_string(scope)
                .map(|s| s.to_rust_string_lossy(scope))
                .unwrap_or_default();

            if let Some(state) = get_fetch_state(scope) {
                let _ = state.scheduler_tx.send(SchedulerMessage::Fetch(id, url));
            }
        },
    )
    .unwrap();

    let name = v8::String::new(scope, "__nativeFetch").unwrap();
    global.set(scope, name.into(), native_fetch.into());

    let js_code = r#"
        // Fetch state
        globalThis.__fetchCallbacks = new Map();
        globalThis.__nextFetchId = 1;

        // Simple fetch implementation that returns a Promise
        globalThis.fetch = function(url) {
            return new Promise((resolve, reject) => {
                const id = globalThis.__nextFetchId++;
                globalThis.__fetchCallbacks.set(id, { resolve, reject });
                __nativeFetch(id, url);
            });
        };

        // Success callback called from Rust
        globalThis.__executeFetchSuccess = function(id, body) {
            const callbacks = globalThis.__fetchCallbacks.get(id);
            if (callbacks) {
                globalThis.__fetchCallbacks.delete(id);
                // Create a minimal Response-like object
                callbacks.resolve({
                    text: () => Promise.resolve(body),
                    json: () => Promise.resolve(JSON.parse(body)),
                    ok: true,
                    status: 200
                });
            }
        };

        // Error callback called from Rust
        globalThis.__executeFetchError = function(id, error) {
            const callbacks = globalThis.__fetchCallbacks.get(id);
            if (callbacks) {
                globalThis.__fetchCallbacks.delete(id);
                callbacks.reject(new Error(error));
            }
        };
    "#;

    let code_str = v8::String::new(scope, js_code).unwrap();
    let script = v8::Script::compile(scope, code_str, None).unwrap();
    script.run(scope).unwrap();
}
