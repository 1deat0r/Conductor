use conductor_protocol::{HealthV1, Service};

#[allow(dead_code)] // S1 journal foundation is not enabled by the S0 CLI.
mod journal;

#[cfg(test)]
mod journal_tests;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [arg] if arg == "doctor" => println!(
            "{}",
            serde_json::to_string(&HealthV1::scaffold(Service::Supervisor))
                .expect("health serialization")
        ),
        [arg] if arg == "--version" => {
            println!("conductor-supervisor {}", env!("CARGO_PKG_VERSION"))
        }
        _ => {
            eprintln!(
                "Usage: conductor-supervisor doctor | --version. Execution is not implemented."
            );
            std::process::exit(2);
        }
    }
}
