use mqtt_async_client::client::{Client, Publish, QoS, Subscribe, SubscribeTopic};
use pcf_zenoh::{get_epoch_ns, TerminationNotification};

use clap::Parser;

#[derive(Parser)]
pub struct Opts {
    // public options
    #[clap(short = 'b', long, default_value = "mqtt://127.0.0.1:1883")]
    pub broker: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let opts = Opts::parse();
    let mut client = Client::builder()
        .set_url_string(&opts.broker)
        .unwrap()
        .set_client_id(Some("pcf".into()))
        .build()
        .unwrap();

    client.connect().await.unwrap();

    let subs = client
        .subscribe(Subscribe::new(vec![SubscribeTopic {
            topic_path: "npcf-smpolicycontrol/v1/sm-policies".into(),
            qos: QoS::AtMostOnce,
        }]))
        .await
        .unwrap();
    subs.any_failures().unwrap();

    if let Ok(r) = client.read_subscriptions().await {
        let cb = std::str::from_utf8(r.payload()).unwrap();
        log::info!("Received data from {}  - {cb}", r.topic());
        loop {
            let tn = TerminationNotification {
                supi: "imsi-001011234567895".into(),
                pdu_session_id: 1,
                ts: get_epoch_ns(),
            };
            log::info!("Sending {tn:?} to {cb}");
            client
                .publish(&Publish::new(cb.into(), tn.ser()))
                .await
                .unwrap();
        }
    }
}
