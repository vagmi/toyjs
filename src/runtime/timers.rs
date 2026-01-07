use super::SchedulerMessage;
use tokio::sync::mpsc;
use v8;

struct TimerState {
    scheduler_tx: mpsc::UnboundedSender<SchedulerMessage>,
}

fn get_timer_state<'a>(scope: &mut v8::PinScope) -> Option<&'a TimerState> {
    let global = scope.get_current_context().global(scope);
    let state_key = v8::String::new(scope, "__timerState").unwrap();
    let state_val = global.get(scope, state_key.into())?;

    if !state_val.is_external() {
        return None;
    }

    let external: v8::Local<v8::External> = state_val.try_into().ok()?;
    let state_ptr = external.value() as *const TimerState;
    Some(unsafe { &*state_ptr })
}

pub fn setup_timers(
    scope: &mut v8::PinScope,
    scheduler_tx: mpsc::UnboundedSender<SchedulerMessage>,
) {
    let global = scope.get_current_context().global(scope);

    let state = TimerState { scheduler_tx };
    let state_ptr = Box::into_raw(Box::new(state)) as *mut std::ffi::c_void;
    let external = v8::External::new(scope, state_ptr);
    let state_key = v8::String::new(scope, "__timerState").unwrap();
    global.set(scope, state_key.into(), external.into());

    let native_schedule_timeout = v8::Function::new(
        scope,
        |scope: &mut v8::PinScope,
         args: v8::FunctionCallbackArguments,
         mut _retval: v8::ReturnValue| {
            if args.length() < 2 {
                return;
            }

            let id = args.get(0).number_value(scope).unwrap_or(0.0) as u64;
            let delay = args.get(1).number_value(scope).unwrap_or(0.0) as u64;

            if let Some(state) = get_timer_state(scope) {
                let _ = state.scheduler_tx.send(SchedulerMessage::ScheduleTimeout(id, delay));
            }
        },
    )
    .unwrap();

    let name = v8::String::new(scope, "__nativeScheduleTimeout").unwrap();
    global.set(scope, name.into(), native_schedule_timeout.into());

    let native_schedule_interval = v8::Function::new(
        scope,
        |scope: &mut v8::PinScope,
         args: v8::FunctionCallbackArguments,
         mut _retval: v8::ReturnValue| {
            if args.length() < 2 {
                return;
            }

            let id = args.get(0).number_value(scope).unwrap_or(0.0) as u64;
            let interval = args.get(1).number_value(scope).unwrap_or(0.0) as u64;

            if let Some(state) = get_timer_state(scope) {
                let _ = state.scheduler_tx.send(SchedulerMessage::ScheduleInterval(id, interval));
            }
        },
    )
    .unwrap();

    let name = v8::String::new(scope, "__nativeScheduleInterval").unwrap();
    global.set(scope, name.into(), native_schedule_interval.into());

    let native_clear_timer = v8::Function::new(
        scope,
        |scope: &mut v8::PinScope,
         args: v8::FunctionCallbackArguments,
         mut _retval: v8::ReturnValue| {
            if args.length() < 1 {
                return;
            }

            let id = args.get(0).number_value(scope).unwrap_or(0.0) as u64;

            if let Some(state) = get_timer_state(scope) {
                let _ = state.scheduler_tx.send(SchedulerMessage::ClearTimer(id));
            }
        },
    )
    .unwrap();

    let name = v8::String::new(scope, "__nativeClearTimer").unwrap();
    global.set(scope, name.into(), native_clear_timer.into());

    let js_code = r#"
        // Timer state
        globalThis.__timerCallbacks = new Map();
        globalThis.__intervalIds = new Set();
        globalThis.__nextTimerId = 1;

        // setTimeout implementation
        globalThis.setTimeout = function(callback, delay) {
            const id = globalThis.__nextTimerId++;
            globalThis.__timerCallbacks.set(id, callback);
            __nativeScheduleTimeout(id, delay || 0);
            return id;
        };

        // setInterval implementation
        globalThis.setInterval = function(callback, interval) {
            const id = globalThis.__nextTimerId++;
            globalThis.__timerCallbacks.set(id, callback);
            globalThis.__intervalIds.add(id);
            __nativeScheduleInterval(id, interval || 0);
            return id;
        };

        // clearTimeout and clearInterval
        globalThis.clearTimeout = function(id) {
            globalThis.__timerCallbacks.delete(id);
            globalThis.__intervalIds.delete(id);
            __nativeClearTimer(id);
        };

        globalThis.clearInterval = function(id) {
            globalThis.__timerCallbacks.delete(id);
            globalThis.__intervalIds.delete(id);
            __nativeClearTimer(id);
        };

        // Callback executor called from Rust
        globalThis.__executeTimer = function(id) {
            const callback = globalThis.__timerCallbacks.get(id);
            if (callback) {
                callback();
                // For setTimeout (not interval), remove the callback after execution
                if (!globalThis.__intervalIds.has(id)) {
                    globalThis.__timerCallbacks.delete(id);
                }
            }
        };
    "#;

    let code_str = v8::String::new(scope, js_code).unwrap();
    let script = v8::Script::compile(scope, code_str, None).unwrap();
    script.run(scope).unwrap();
}
