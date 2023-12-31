//! Main binary entry point for openapi_client implementation.

#![allow(missing_docs)]

use clap::Parser;
use url::Url;

mod server;

#[derive(Parser)]
pub struct Opts {
    // public options
    #[clap(short = 'l', long, default_value = "http://127.0.0.1:8080")]
    pub listen: String,
}

/// Create custom server, wire it to the autogenerated router,
/// and pass it to the web server.
#[tokio::main]
async fn main() {
    env_logger::init();
    let opts = Opts::parse();

    let base_url = Url::parse(&opts.listen).unwrap();
    let is_https = if base_url.scheme() == "https" {
        true
    } else {
        false
    };

    server::create(
        &format!(
            "{}:{}",
            base_url.host_str().unwrap(),
            base_url.port().unwrap()
        ),
        is_https,
    )
    .await;
}
