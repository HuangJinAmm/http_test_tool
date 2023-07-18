use log::{debug, error, info, trace, warn};
use once_cell::sync::{Lazy};
use rhai::{Engine};
// use rhai_rand::RandomPackage;
// use rhai_sci::SciPackage;

use crate::app::{TASK_CHANNEL, TOKIO_RT};

pub const SCRIPT_ENGINE: Lazy<Engine> = Lazy::new(|| {
    let mut engine = Engine::new();
    // Create new 'RandomPackage' instance
    // let random = RandomPackage::new();

    // Load the package into the `Engine`
    // random.register_into_engine(&mut engine);

    // Create new 'SciPackage' instance
    // let sci = SciPackage::new();

    // Load the package into the [`Engine`]
    // sci.register_into_engine(&mut engine);

    // Create new 'FilesystemPackage' instance
    // let random = FilesystemPackage::new();

    // Load the package into the `Engine`
    // random.register_into_engine(&mut engine);

    engine.register_fn("call_req", call_req);
    engine.register_fn("log_info", log_info);
    engine.register_fn("log_error", log_error);
    engine.register_fn("log_debug", log_debug);
    engine.register_fn("log_warn", log_warn);
    engine.register_fn("log_trace", log_trace);
    engine.set_max_call_levels(255);
    engine
});

// pub struct ScriptEngine {
//     pub engine: Engine,
// }

fn call_req(id: u64) {
    let task_sender = unsafe { TASK_CHANNEL.0.clone() };
    TOKIO_RT.spawn(async move {
        if let Err(_) = task_sender.send((id, 1, 1)).await {
            log::info!("receiver dropped");
            return;
        }
    });
}

fn log_info(msg: &str) {
    info!("{}", msg);
}
fn log_error(msg: &str) {
    error!("{}", msg);
}
fn log_debug(msg: &str) {
    debug!("{}", msg);
}
fn log_warn(msg: &str) {
    warn!("{}", msg);
}
fn log_trace(msg: &str) {
    trace!("{}", msg);
}

// impl ScriptEngine {
//     pub fn new() -> Self {
//         let mut engine = Engine::new();
//         engine.register_fn("call_req", call_req);
//         engine.register_fn("log_info", log_info);
//         engine.register_fn("log_error", log_error);
//         engine.register_fn("log_debug", log_debug);
//         engine.register_fn("log_warn", log_warn);
//         engine.register_fn("log_trace", log_trace);
//         engine.set_max_call_levels(255);
//         ScriptEngine { engine }
//     }

//     pub fn run(&self, script: &str) {
//         let res = self.engine.run(script);
//     }
// }

#[cfg(test)]
mod tests {
    use rhai::{Dynamic, Engine};

    use super::*;

    #[test]
    fn test_script() {
        let mut engine = Engine::new();
        engine.set_max_call_levels(150);
        let s = r#"
        //! This script calculates the n-th Fibonacci number using a really dumb algorithm
//! to test the speed of the scripting engine.

const TARGET = 28;
const REPEAT = 5;
const ANSWER = 317_811;

fn fib(n) {
    if n < 2 {
        n
    } else {
        fib(n-1) + fib(n-2)
    }
}

print(`Running Fibonacci(${TARGET}) x ${REPEAT} times...`);
print("Ready... Go!");

let result;
let now = timestamp();

for n in 0..REPEAT {
    result = fib(TARGET);
}

print(`Finished. Run time = ${now.elapsed} seconds.`);

print(`Fibonacci number #${TARGET} = ${result}`);

if result != ANSWER {
    print(`The answer is WRONG! Should be ${ANSWER}!`);
}
return result;
        "#;

        let result = engine.eval::<Dynamic>(s).unwrap();

        println!("Answer: {result}"); // prints 42
    }
}
