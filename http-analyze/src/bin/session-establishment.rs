#![allow(missing_docs, unused_variables, trivial_casts)]

use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use clap::Parser;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Response, Server};
use nnrf_discovery_server::models::{NfType, ServiceName};
use nnrf_discovery_server::{
    ApiNoContext as NRFApiNoContext, Client as NRFClient, ContextWrapperExt as NRFContextWrapperExt,
};

use nsfm_pdusession::{
    ApiNoContext as SMFApiNoContext, Client as SMFClient, ContextWrapperExt as SMFContextWrapperExt,
};

use log::info;

// swagger::Has may be unused if there are no examples
use swagger::{AuthData, ContextBuilder, EmptyContext, Has, Push, XSpanIdString};
use url::Url;

type ClientContext = swagger::make_context_ty!(
    ContextBuilder,
    EmptyContext,
    Option<AuthData>,
    XSpanIdString
);

struct EstErr(String);

async fn establish_session(
    nrf_client: &Box<dyn NRFApiNoContext<ClientContext>>,
    smf_client: &Box<dyn SMFApiNoContext<ClientContext>>,
    flag: Arc<AtomicBool>,
) -> Result<(), EstErr> {
    // Mocking the session establishment.
    // 0. Us NRF in order to discover SMF
    // 1. Send a create SM context to the SMF
    // 2. Wait for the SM Context create response
    // Done!

    // println!("Calling NRF");
    // Discover SMF
    let sfm_search_result = nrf_client
        .search_nf_instances(
            NfType::SMF,
            NfType::AMF,
            Some("applicaiton/json,application/problem+json".to_string()),
            None,
            None,
            Some(&vec![ServiceName::new("nsmf-pdusession".to_string())]),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some("20".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .map_err(|e| EstErr(format!("NRF {e:?}")))?;

    info!(
        "{:?} (X-Span-ID: {:?})",
        sfm_search_result,
        (nrf_client.context() as &dyn Has<XSpanIdString>)
            .get()
            .clone()
    );
    // Send create context request

    // println!("Calling SMF");

    let nas_data = [
        0x2E, 0x01, 0x01, 0xC1, 0xFF, 0xFF, 0x91, 0xA1, 0x28, 0x01, 0x00, 0x7B, 0x00, 0x07, 0x80,
        0x00, 0x0A, 0x00, 0x00, 0x0D, 0x00,
    ];
    let json_request = r#"{
             "supi":	"imsi-001011234567895",
             "pei":	"imeisv-4370816125816151",
             "pduSessionId":	1,
             "dnn":	"internet",
             "sNssai":	{
                 "sst":	1
             },
             "servingNfId":	"66bf4df8-b832-41ed-aa12-4df3ea315a7c",
             "guami":	{
                 "plmnId":	{
                     "mcc":	"001",
                     "mnc":	"01"
                 },
                 "amfId":	"020040"
             },
             "servingNetwork":	{
                 "mcc":	"001",
                 "mnc":	"01"
             },
             "n1SmMsg":	{
                 "contentId":	"5gnas-sm"
             },
             "anType":	"3GPP_ACCESS",
             "ratType":	"NR",
             "ueLocation":	{
                 "nrLocation":	{
                     "tai":	{
                         "plmnId":	{
                             "mcc":	"001",
                             "mnc":	"01"
                         },
                         "tac":	"000001"
                     },
                     "ncgi":	{
                         "plmnId":	{
                             "mcc":	"001",
                             "mnc":	"01"
                         },
                         "nrCellId":	"000000010"
                     },
                     "ueLocationTimestamp":	"2023-03-01T13:42:11.144288Z"
                 }
             },
             "ueTimeZone":	"+00:00",
             "smContextStatusUri":	"http://172.22.0.10:7777/namf-callback/v1/imsi-001011234567895/sm-context-status/1",
             "pcfId":	"6c05c1d4-b832-41ed-9698-8dec5d3774de"
         }"#;

    let sm_session_creation_result = smf_client
        .post_sm_contexts(
            serde_json::from_str(json_request).map_err(|e| EstErr(format!("parse {e:?}")))?,
            Some(swagger::ByteArray(Vec::from(nas_data))),
            None,
            None,
        )
        .await
        .map_err(|e| EstErr(format!("SMF {e:?}")))?;

    info!(
        "{:?} (X-Span-ID: {:?})",
        sm_session_creation_result,
        (smf_client.context() as &dyn Has<XSpanIdString>)
            .get()
            .clone()
    );

    // waits for the callback from AMF
    while let Err(_) = flag.compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed) {}

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
    // public options
    #[clap(short = 'l', long, default_value = "http://127.0.0.1:8083")]
    pub listen: String,
    #[clap(short = 'n', long, default_value = "http://127.0.0.1:8080")]
    pub nrf: String,
    #[clap(short = 's', long, default_value = "http://127.0.0.1:8082")]
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

    let nrf_base_url = opts.nrf.clone();

    let smf_base_url = opts.smf.clone();

    let runs = opts.runs;

    let context: ClientContext = swagger::make_context!(
        ContextBuilder,
        EmptyContext,
        None as Option<AuthData>,
        XSpanIdString::default()
    );

    // NRF client
    let mut nrf_client: Box<dyn NRFApiNoContext<ClientContext>> = {
        let client =
            Box::new(NRFClient::try_new_http(&nrf_base_url).expect("Failed to create HTTP client"));
        Box::new(client.with_context(context.clone()))
    };

    // SFM client

    let mut smf_client: Box<dyn SMFApiNoContext<ClientContext>> = {
        // Using HTTP
        let client =
            Box::new(SMFClient::try_new_http(&smf_base_url).expect("Failed to create HTTP client"));
        Box::new(client.with_context(context.clone()))
    };

    let flag = Arc::new(AtomicBool::new(false));

    let c_flag = flag.clone();
    // Spawn callback server
    tokio::task::spawn(
        async move { spawn_server(c_flag, Url::parse(&opts.listen).unwrap()).await },
    );

    let mut i = 0;

    while i < runs {
        let now = Instant::now();
        match establish_session(&nrf_client, &smf_client, flag.clone()).await {
            Ok(_) => {
                let delta = now.elapsed();
                println!("establishment,http,{},ns", delta.as_nanos());
                i += 1;
            }
            Err(_) => (),
        }
    }
}
