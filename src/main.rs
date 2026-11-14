#![forbid(unsafe_code)]

use declmig_web_server::{config::WebConfig, flags, server};

fn main() {
    let environment = match flags::resolve() {
        Ok(environment) => environment,
        Err(_) => {
            eprintln!("invalid web flags or environment configuration");
            std::process::exit(2);
        }
    };
    let cfg = WebConfig::from_map(&environment);
    if let Err(error) = cfg.validate() {
        eprintln!("invalid web service configuration: {error}");
        std::process::exit(2);
    }
    server::run(&cfg);
}
