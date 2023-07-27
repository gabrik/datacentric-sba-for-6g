use clap::Parser;
use tonic::transport::Server;
use udm_grpc::nudm_sdm::udm_server::UdmServer;
use udm_grpc::MyUDM;

#[derive(Parser)]
pub struct Opts {
    // public options
    #[clap(short = 'l', long, default_value = "127.0.0.1:9091")]
    pub listen: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::parse();

    let addr = opts.listen.parse()?;
    let udm = MyUDM::default();

    Server::builder()
        .add_service(UdmServer::new(udm))
        .serve(addr)
        .await?;

    Ok(())
}
