use clap::Parser;
use sfm_grpc::nsfm_pdusession::smf_server::SmfServer;
use sfm_grpc::MySmf;
use tonic::transport::Server;

#[derive(Parser)]
pub struct Opts {
    // public options
    #[clap(short = 'l', long, default_value = "127.0.0.1:9092")]
    pub listen: String,
    #[clap(short = 'u', long, default_value = "http://127.0.0.1:9091")]
    pub udm: String,
    #[clap(short = 'u', long, default_value = "http://127.0.0.1:9093")]
    pub amf: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::parse();

    let addr = opts.listen.parse()?;
    let smf = MySmf::new(opts.udm.into(), opts.amf).await;

    Server::builder()
        .add_service(SmfServer::new(smf))
        .serve(addr)
        .await?;

    Ok(())
}
