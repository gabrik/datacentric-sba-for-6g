#![allow(missing_docs, unused_variables, trivial_casts)]

use clap::Parser;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Response, Server};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use url::Url;

use nrf_grpc::nnrf_disc::nrf_discovery_client::NrfDiscoveryClient;
use nrf_grpc::nnrf_disc::SearchRequest;

use sfm_grpc::nsfm_pdusession::smf_client::SmfClient;
use sfm_grpc::nsfm_pdusession::{
    CreateSmContextRequest, Guami, Ncgi, NrLocation, Nssai, PlmnId, Tai, UeLocation,
};
use tonic::transport::Channel;

use log::info;

struct EstErr(String);

async fn establish_session(
    smf_client: &mut SmfClient<Channel>,
    flag: Arc<AtomicBool>,
) -> Result<(), EstErr> {
    // Mocking the session establishment.
    // 1. Send a create SM context to the SMF
    // 2. Wait for the SM Context create response
    // Done!

    // Send create context request

    // println!("Calling SMF");

    let nas_data = [
        0x2E, 0x01, 0x01, 0xC1, 0xFF, 0xFF, 0x91, 0xA1, 0x28, 0x01, 0x00, 0x7B, 0x00, 0x07, 0x80,
        0x00, 0x0A, 0x00, 0x00, 0x0D, 0x00,
    ];

    let smf_req = CreateSmContextRequest {
        supi: "imsi-001011234567895".into(),
        pei: "imeisv-4370816125816151".into(),
        pdu_session_id: 1,
        dnn: "internet".into(),
        s_nnssai: Some(Nssai {
            sst: 1,
            sd: "".into(),
        }),
        serving_nf_id: "66bf4df8-b832-41ed-aa12-4df3ea315a7c".into(),
        guami: Some(Guami {
            plmn_id: Some(PlmnId {
                mcc: "001".into(),
                mnc: "01".into(),
            }),
            amf_id: "020040".into(),
        }),
        serving_network: Some(PlmnId {
            mcc: "001".into(),
            mnc: "01".into(),
        }),
        an_type: "3GPP_ACCESS".into(),
        rat_type: "NR".into(),
        ue_location: Some(UeLocation {
            nr_location: Some(NrLocation {
                tai: Some(Tai {
                    plmn_id: Some(PlmnId {
                        mcc: "001".into(),
                        mnc: "01".into(),
                    }),
                    tac: "000001".into(),
                }),
                ncgi: Some(Ncgi {
                    plmn_id: Some(PlmnId {
                        mcc: "001".into(),
                        mnc: "01".into(),
                    }),
                    nr_cell_id: "000000010".into(),
                }),
                ue_location_timestamp: "2023-03-01T13:42:11.144288Z".into(),
            }),
        }),
        ue_timezone: "+00:00".into(),
        sm_context_status_uri:
            "http://172.22.0.10:7777/namf-callback/v1/imsi-001011234567895/sm-context-status/1"
                .into(),
        pcf_id: "6c05c1d4-b832-41ed-9698-8dec5d3774de".into(),
        n1_sm_msg: nas_data.into(),
    };

    let sm_session_creation_result = smf_client
        .sm_context(smf_req)
        .await
        .map_err(|e| EstErr(format!("SMF {e:?}")))?;

    info!("{:?}", sm_session_creation_result,);

    // waits for the callback from AMF
    let _ = flag.compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed);

    Ok(())
}

async fn spawn_server(flag: Arc<AtomicBool>, listen_url: Url) {
    let url_str = format!(
        "{}:{}",
        listen_url.host_str().unwrap(),
        listen_url.port().unwrap()
    );

    let addr: SocketAddr = url_str.parse().expect("unable to parse listen url");

    // For the most basic of state, we just share a counter, that increments
    // with each request, and we send its value back in the response.
    // The closure inside `make_service_fn` is run for each connection,
    // creating a 'service' to handle requests for that specific connection.
    let make_service = make_service_fn(move |_| {
        // While the state was moved into the make_service closure,
        // we need to clone it here because this closure is called
        // once for every connection.
        //
        // Each connection could send multiple requests, so
        // the `Service` needs a clone to handle later requests.
        let flag = flag.clone();

        async move {
            // This is the `Service` that will handle the connection.
            // `service_fn` is a helper to convert a function that
            // returns a Response into a `Service`.
            Ok::<_, Error>(service_fn(move |_req| {
                // Get the current count, and also increment by 1, in a single
                // atomic operation.
                flag.store(true, Ordering::Relaxed);
                async move { Ok::<_, Error>(Response::new(Body::from(format!("Hello")))) }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_service);

    // println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

#[derive(Parser)]
pub struct Opts {
    #[clap(short = 'l', long, default_value = "http://127.0.0.1:9093")]
    pub listen: String,
    #[clap(short = 's', long, default_value = "http://127.0.0.1:9092")]
    pub smf: String,
    #[clap(short = 's', long, default_value = "10")]
    pub runs: usize,
}

// rt may be unused if there are no examples
#[allow(unused_mut)]
#[tokio::main]
async fn main() {
    env_logger::init();

    let opts = Opts::parse();

    let smf_base_url = opts.smf.clone();

    let runs = opts.runs;
    let mut smf_client = SmfClient::connect(smf_base_url).await.unwrap();

    let flag = Arc::new(AtomicBool::new(false));

    let c_flag = flag.clone();
    // Spawn callback server
    tokio::task::spawn(
        async move { spawn_server(c_flag, Url::parse(&opts.listen).unwrap()).await },
    );

    let mut i = 0;

    while i < runs {
        let now = Instant::now();
        match establish_session(&mut smf_client, flag.clone()).await {
            Ok(_) => {
                let delta = now.elapsed();
                println!("establishment,grpc,{},ns", delta.as_nanos());
                i += 1;
            }
            Err(_) => (),
        }
    }
}
