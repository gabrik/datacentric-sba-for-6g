use mqtt_async_client::client::{Client, Publish, QoS, Subscribe, SubscribeTopic};

use pcf_zenoh::{get_epoch_ns, TerminationNotification};

use clap::Parser;

#[derive(Parser)]
pub struct Opts {
    // public options
    #[clap(short = 'l', long, default_value = "mqtt://127.0.0.1:1883")]
    pub listen: String,
    #[clap(short = 'r', long, default_value = "10")]
    pub runs: usize,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let opts = Opts::parse();
    let runs = opts.runs;

    let mut client = Client::builder()
        .set_url_string(&opts.listen)
        .unwrap()
        .set_client_id(Some("client".into()))
        .build()
        .unwrap();

    client.connect().await.unwrap();

    let subs = client
        .subscribe(Subscribe::new(vec![SubscribeTopic {
            topic_path: "smf-callback/v1/sm-policy-notify/1".into(),
            qos: QoS::AtMostOnce,
        }]))
        .await
        .unwrap();
    subs.any_failures().unwrap();

    let cb = "smf-callback/v1/sm-policy-notify/1".as_bytes();
    client
        .publish(&Publish::new(
            "npcf-smpolicycontrol/v1/sm-policies".into(),
            cb.to_vec(),
        ))
        .await
        .unwrap();

    let mut count = 0;
    while count <= runs {
        if let Ok(r) = client.read_subscriptions().await {
            log::info!("Received data from {}", r.topic());
            let tn = TerminationNotification::de(r.payload());
            let delta = get_epoch_ns() - tn.ts;
            println!("notification,mqtt,{},ns", delta);
            count += 1;
        }
    }
}
