#![forbid(unsafe_code)]

use declmig_web_server::{config::WebConfig, server};

fn main() {
    let cfg = WebConfig::from_env();
    server::run(&cfg);
}

