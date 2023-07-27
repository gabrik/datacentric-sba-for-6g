use clap::Parser;
use nrf_grpc::nnrf_disc::nrf_discovery_server::NrfDiscoveryServer;
use nrf_grpc::MyNRF;
use tonic::transport::Server;

#[derive(Parser)]
pub struct Opts {
    // public options
    #[clap(short = 'l', long, default_value = "127.0.0.1:9090")]
    pub listen: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::parse();
    let addr = opts.listen.parse()?;
    let nrf = MyNRF::default();

    Server::builder()
        .add_service(NrfDiscoveryServer::new(nrf))
        .serve(addr)
        .await?;

    Ok(())
}
