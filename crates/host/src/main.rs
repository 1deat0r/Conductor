use conductor_protocol::{HealthV1, Service};
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [arg] if arg == "doctor" => println!(
            "{}",
            serde_json::to_string(&HealthV1::scaffold(Service::Host))
                .expect("health serialization")
        ),
        [arg] if arg == "--version" => println!("conductor-host {}", env!("CARGO_PKG_VERSION")),
        _ => {
            eprintln!("Usage: conductor-host doctor | --version. Execution is not implemented.");
            std::process::exit(2);
        }
    }
}
