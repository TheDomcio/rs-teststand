//! Example: run every other example in order, reporting what each one did.
//!
//! ```text
//! cargo run --example launch_all_examples
//! ```
//!
//! Each example is run as its own process. That is deliberate rather than
//! convenient: an example owns its engine from construction to teardown, and
//! sharing one engine across all of them would mean rewriting every example to
//! accept a borrowed engine, which would make each one worse as documentation
//! for the sake of this launcher. A separate process also means one example
//! that wedges cannot take the rest down with it.
//!
//! The order is roughly the order someone would learn them in: ask the engine
//! what it is, read the station, build a file, run it, then get results out.
//!
//! The analyzer examples are not included; that domain is out of scope for now.

use std::process::Command;
use std::time::{Duration, Instant};

/// How long any one example may take before it is treated as wedged.
const PER_EXAMPLE_TIMEOUT: Duration = Duration::from_secs(120);

/// The examples, in the order they are run.
///
/// `property_object_serialize` lives in the `rs-teststand-serde` crate, so it
/// carries the package it belongs to.
const EXAMPLES: [(&str, &str); 11] = [
    ("rs-teststand", "version_print"),
    ("rs-teststand", "station_options_update"),
    ("rs-teststand", "search_directory_manage"),
    ("rs-teststand", "users_manage"),
    ("rs-teststand", "sequence_build"),
    ("rs-teststand", "step_insert"),
    ("rs-teststand", "step_insert_from_template"),
    ("rs-teststand", "template_manage_complex"),
    ("rs-teststand", "variables_manage"),
    ("rs-teststand", "data_type_manage"),
    ("rs-teststand-serde", "property_object_serialize"),
];

/// Examples that start an execution. Run last, because an engine that has run a
/// sequence is the one most likely to leave something behind.
const EXECUTION_EXAMPLES: [(&str, &str); 4] = [
    ("rs-teststand", "execution_run_test_headless"),
    ("rs-teststand", "execution_run_subsequence"),
    ("rs-teststand", "result_list_parse"),
    ("rs-teststand", "ui_messages_handle"),
];

/// What happened to one example.
enum Outcome {
    Ok(Duration),
    Failed(Duration, Option<i32>),
    TimedOut,
}

fn run(package: &str, example: &str) -> Outcome {
    let started = Instant::now();
    let Ok(mut child) = Command::new(env!("CARGO"))
        .args(["run", "--quiet", "-p", package, "--example", example])
        .spawn()
    else {
        return Outcome::Failed(started.elapsed(), None);
    };

    // Poll rather than `wait`, so a wedged example is bounded. An example that
    // raises a modal dialog would otherwise hold this launcher for ever, which
    // is the failure the crate exists to prevent.
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let elapsed = started.elapsed();
                return if status.success() {
                    Outcome::Ok(elapsed)
                } else {
                    Outcome::Failed(elapsed, status.code())
                };
            }
            Ok(None) => {
                if started.elapsed() > PER_EXAMPLE_TIMEOUT {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Outcome::TimedOut;
                }
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(_) => return Outcome::Failed(started.elapsed(), None),
        }
    }
}

fn main() {
    let all: Vec<(&str, &str)> = EXAMPLES.into_iter().chain(EXECUTION_EXAMPLES).collect();

    println!("Running {} examples in order.\n", all.len());
    let started = Instant::now();
    let mut failures = Vec::new();

    for (index, (package, example)) in all.iter().enumerate() {
        println!("[{}/{}] {example}", index + 1, all.len());
        match run(package, example) {
            Outcome::Ok(elapsed) => println!("      ok, {:.1}s\n", elapsed.as_secs_f64()),
            Outcome::Failed(elapsed, code) => {
                println!(
                    "      FAILED ({}), {:.1}s\n",
                    code.map_or_else(|| "no exit code".to_owned(), |c| format!("exit {c}")),
                    elapsed.as_secs_f64()
                );
                failures.push(*example);
            }
            Outcome::TimedOut => {
                println!("      TIMED OUT after {PER_EXAMPLE_TIMEOUT:?}\n");
                failures.push(*example);
            }
        }
    }

    println!(
        "{} of {} succeeded in {:.1}s total.",
        all.len() - failures.len(),
        all.len(),
        started.elapsed().as_secs_f64()
    );
    if !failures.is_empty() {
        println!("Failed: {}", failures.join(", "));
        // A non-zero exit so a supervisor or CI notices, rather than a launcher
        // that reports trouble and then claims success.
        std::process::exit(1);
    }
}
