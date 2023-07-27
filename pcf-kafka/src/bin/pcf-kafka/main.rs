mod opts;

use anyhow::Result;
use clap::Parser;
use log::{info, trace, warn};
use opts::Opts;
use pcf_kafka::{AsyncStdFutureProducer, AsyncStdStreamConsumer, DEFAULT_GROUP_ID};
use pcf_zenoh::{get_epoch_ns, TerminationNotification};
use rdkafka::{
    consumer::Consumer, error::KafkaError, producer::FutureRecord, types::RDKafkaErrorCode,
    ClientConfig, Message as _,
};
use std::{process, time::Duration};

async fn notify(cb: String, producer: AsyncStdFutureProducer, id: u32) {
    let record_key = id.to_le_bytes();
    loop {
        let tn = TerminationNotification {
            supi: "imsi-001011234567895".into(),
            pdu_session_id: 1,
            ts: get_epoch_ns(),
        };
        let payload = tn.ser();
        let record = FutureRecord::to(&cb).payload(&payload).key(&record_key);
        trace!("send a notification");
        producer
            .send(record, Duration::ZERO)
            .await
            .map_err(|(err, _msg)| err)
            .unwrap();
    }
}

fn create_consumer(
    opts: &Opts,
    mut config: ClientConfig,
    topic: &str,
) -> Result<AsyncStdStreamConsumer> {
    config
        // .set("enable.partition.eof", "false")
        // .set("enable.auto.commit", "false")
        .set("group.id", DEFAULT_GROUP_ID)
        .set("session.timeout.ms", "6000");

    if let Some(configs) = &opts.consumer_configs {
        for kv in configs {
            config.set(&kv.key, &kv.val);
        }
    }

    let consumer: AsyncStdStreamConsumer = config.create()?;
    consumer.subscribe(&[topic])?;
    Ok(consumer)
}

#[async_std::main]
async fn main() {
    pretty_env_logger::init();

    let opts = Opts::parse();

    let pcf_id = process::id();
    info!("Start pcf {}", pcf_id);

    let client_config = {
        let mut conf = ClientConfig::new();
        conf.set("bootstrap.servers", &opts.brokers);
        conf
    };

    // Configure the consumer
    let mut consumer: AsyncStdStreamConsumer =
        create_consumer(&opts, client_config.clone(), &opts.pcf_topic).unwrap();

    let mut client_config = client_config.clone();
    if let Some(cfgs) = &opts.producer_configs {
        cfgs.iter().for_each(|kv| {
            client_config.set(&kv.key, &kv.val);
        });
    }

    use KafkaError as E;
    use RDKafkaErrorCode as C;

    loop {
        let msg = loop {
            let result = consumer.recv().await;
            match result {
                Ok(msg) => break msg.detach(),
                Err(E::MessageConsumption(C::UnknownTopicOrPartition)) => {
                    // retry
                    trace!(
                        "The topic {} is not created yet, retry again",
                        &opts.pcf_topic
                    );
                    async_std::task::sleep(Duration::from_secs(1)).await;
                    consumer =
                        create_consumer(&opts, client_config.clone(), &opts.pcf_topic).unwrap();
                    continue;
                }
                Err(err) => panic!("{err}"),
            }
        };

        trace!("received a registration to notification");

        let payload = match msg.payload() {
            Some(payload) => payload,
            None => {
                warn!("the payload does not present in the received message");
                continue;
            }
        };

        let producer: AsyncStdFutureProducer = client_config.create().unwrap();
        let cb = std::str::from_utf8(payload).unwrap().to_string();

        async_std::task::spawn(notify(cb, producer, pcf_id));
    }
}
