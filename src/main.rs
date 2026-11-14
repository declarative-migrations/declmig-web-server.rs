#![forbid(unsafe_code)]

use declmig_web_server::{config::WebConfig, flags, server};

fn main() {
    let environment = flags::resolve().unwrap_or_else(|error| panic!("{error}"));
    let cfg = WebConfig::from_map(&environment);
    if let Err(error) = cfg.validate() {
        eprintln!("invalid web service configuration: {error}");
        std::process::exit(2);
    }
    server::run(&cfg);
}
