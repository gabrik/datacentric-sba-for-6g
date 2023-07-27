mod opts;

use anyhow::Result;
use clap::Parser;
use log::{info, trace, warn};
use opts::Opts;
use pcf_kafka::{
    AsyncStdFutureProducer, AsyncStdStreamConsumer, DEFAULT_GROUP_ID, PCF_TOPIC, SMF_TOPIC,
};
use pcf_zenoh::{get_epoch_ns, TerminationNotification};
use rdkafka::{
    consumer::Consumer, error::KafkaError, producer::FutureRecord, types::RDKafkaErrorCode,
    ClientConfig, Message as _,
};
use std::{process, time::Duration};

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

    let runs = opts.runs;
    let smf_id = process::id();
    info!("Start smf {}", smf_id);
    let record_key = smf_id.to_le_bytes();

    let client_config = {
        let mut conf = ClientConfig::new();
        conf.set("bootstrap.servers", &opts.brokers);
        conf
    };

    // Configure the consumer
    let mut consumer: AsyncStdStreamConsumer =
        create_consumer(&opts, client_config.clone(), &opts.smf_topic).unwrap();

    let mut client_config = client_config.clone();
    if let Some(cfgs) = &opts.producer_configs {
        cfgs.iter().for_each(|kv| {
            client_config.set(&kv.key, &kv.val);
        });
    }

    let producer: AsyncStdFutureProducer = client_config.create().unwrap();
    let payload = SMF_TOPIC.as_bytes();
    let record = FutureRecord::to(&PCF_TOPIC)
        .payload(payload)
        .key(&record_key);
    trace!("send registrationn");
    producer
        .send(record, Duration::ZERO)
        .await
        .map_err(|(err, _msg)| err)
        .unwrap();

    use KafkaError as E;
    use RDKafkaErrorCode as C;
    let mut i = 0;
    while i < runs {
        let msg = loop {
            let result = consumer.recv().await;
            match result {
                Ok(msg) => break msg.detach(),
                Err(E::MessageConsumption(C::UnknownTopicOrPartition)) => {
                    // retry
                    trace!(
                        "The topic {} is not created yet, retry again",
                        &opts.smf_topic
                    );
                    async_std::task::sleep(Duration::from_secs(1)).await;
                    consumer =
                        create_consumer(&opts, client_config.clone(), &opts.smf_topic).unwrap();
                    continue;
                }
                Err(err) => panic!("{err}"),
            }
        };

        trace!("received a notification");

        let payload = match msg.payload() {
            Some(payload) => payload,
            None => {
                warn!("the payload does not present in the received message");
                continue;
            }
        };

        log::info!("Received data from {}", msg.topic());
        let tn = TerminationNotification::de(payload);
        let delta = get_epoch_ns() - tn.ts;
        println!("notification,kafka,{},ns", delta);
        i += 1;
    }
}
