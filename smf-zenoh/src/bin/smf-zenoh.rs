use smf_zenoh::server::Server;
use smf_zenoh::SmfApi;
use std::{str::FromStr, sync::Arc};
use zenoh::prelude::r#async::*;
use zenoh_config::{EndPoint, ListenConfig};
use zrpc::ZServe;
#[async_std::main]
async fn main() {
    env_logger::init();

    let mut config = zenoh::config::Config::default();
    config
        .set_mode(Some(zenoh::config::whatami::WhatAmI::Peer))
        .unwrap();
    config
        .set_listen(
            ListenConfig::new(vec![EndPoint {
                locator: Locator::from_str("tcp/127.0.0.1:7072").unwrap(),
                config: None,
            }])
            .unwrap(),
        )
        .unwrap();

    let session = Arc::new(zenoh::open(config).res().await.unwrap());

    async_std::task::sleep(std::time::Duration::from_secs(5)).await;

    let server = Server::new(session.clone()).await;

    let api_server = server.get_smf_api_server(session, None);

    let (_stopper, _h) = api_server.connect().await.unwrap();
    api_server.initialize().await.unwrap();
    api_server.register().await.unwrap();

    let (_s, handle) = api_server.start().await.unwrap();
    println!("Ready!");
    let _ = handle.await;
}
