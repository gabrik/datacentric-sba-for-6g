use anyhow::Result;
use clap::Parser;
use pcf_kafka::{KeyVal, PCF_TOPIC};
use std::time::Duration;

#[derive(Parser)]
pub struct Opts {
    #[clap(long, default_value_t = PCF_TOPIC.to_string())]
    pub pcf_topic: String,
    #[clap(long, value_parser = parse_duration)]
    pub timeout: Option<Duration>,

    // public options
    #[clap(short = 'b', long, default_value = "127.0.0.1")]
    pub brokers: String,

    #[clap(short = 'P', long)]
    pub producer_configs: Option<Vec<KeyVal>>,

    #[clap(short = 'C', long)]
    pub consumer_configs: Option<Vec<KeyVal>>,
}

fn parse_duration(text: &str) -> Result<Duration> {
    let dur = humantime::parse_duration(text)?;
    Ok(dur)
}
