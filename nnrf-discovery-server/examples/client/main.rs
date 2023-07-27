#![allow(missing_docs, unused_variables, trivial_casts)]

use clap::{Parser, ValueEnum};
#[allow(unused_imports)]
use futures::{future, stream, Stream};
use nnrf_discovery_server::models::NfType;
#[allow(unused_imports)]
use nnrf_discovery_server::{
    models, Api, ApiNoContext, Client, ContextWrapperExt, RetrieveCompleteSearchResponse,
    RetrieveStoredSearchResponse, SCpDomainRoutingInfoGetResponse,
    ScpDomainRoutingInfoSubscribeResponse, ScpDomainRoutingInfoUnsubscribeResponse,
    SearchNfInstancesResponse,
};
use std::str::FromStr;
use url::Url;

#[allow(unused_imports)]
use log::info;

// swagger::Has may be unused if there are no examples
#[allow(unused_imports)]
use swagger::{AuthData, ContextBuilder, EmptyContext, Has, Push, XSpanIdString};

type ClientContext = swagger::make_context_ty!(
    ContextBuilder,
    EmptyContext,
    Option<AuthData>,
    XSpanIdString
);

#[derive(Parser)]
pub struct Opts {
    // public options
    #[clap(short = 'c', long, default_value = "http://127.0.0.1:8080")]
    pub connect: String,
    #[clap(short = 'o', long)]
    pub operation: Operation,
}

#[derive(ValueEnum, Clone)]
pub enum Operation {
    RetrieveCompleteSearch,
    ScpDomainRoutingInfoUnsubscribe,
    SCpDomainRoutingInfoGet,
    RetrieveStoredSearch,
    SearchNfInstances,
}

// rt may be unused if there are no examples
#[allow(unused_mut)]
fn main() {
    env_logger::init();

    let opts = Opts::parse();

    let base_url = Url::parse(&opts.connect).unwrap();

    let context: ClientContext = swagger::make_context!(
        ContextBuilder,
        EmptyContext,
        None as Option<AuthData>,
        XSpanIdString::default()
    );

    let mut client: Box<dyn ApiNoContext<ClientContext>> = if base_url.scheme() == "https" {
        // Using Simple HTTPS
        let client = Box::new(
            Client::try_new_https(base_url.as_str()).expect("Failed to create HTTPS client"),
        );
        Box::new(client.with_context(context))
    } else {
        // Using HTTP
        let client = Box::new(
            Client::try_new_http(base_url.as_str()).expect("Failed to create HTTP client"),
        );
        Box::new(client.with_context(context))
    };

    let mut rt = tokio::runtime::Runtime::new().unwrap();

    match opts.operation {
        Operation::RetrieveCompleteSearch => {
            let result = rt.block_on(client.retrieve_complete_search(
                "search_id_example".to_string(),
                Some("accept_encoding_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::ScpDomainRoutingInfoUnsubscribe => {
            let result = rt.block_on(
                client.scp_domain_routing_info_unsubscribe("subscription_id_example".to_string()),
            );
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::SearchNfInstances => {
            let result = rt.block_on(
                client.search_nf_instances(
                    NfType::SMF,
                    NfType::AMF,
                    Some("accept_encoding_example".to_string()),
                    Some(&Vec::new()),
                    Some(
                        uuid::Uuid::from_str("38400000-8cf0-11bd-b23e-10b96e4ef00d")
                            .expect("Failed to parse UUID example"),
                    ),
                    Some(&Vec::new()),
                    Some("requester_nf_instance_fqdn_example".to_string()),
                    Some(&Vec::new()),
                    Some(&Vec::new()),
                    Some(
                        uuid::Uuid::from_str("38400000-8cf0-11bd-b23e-10b96e4ef00d")
                            .expect("Failed to parse UUID example"),
                    ),
                    Some("target_nf_fqdn_example".to_string()),
                    Some("hnrf_uri_example".to_string()),
                    Some(&Vec::new()),
                    Some(&Vec::new()),
                    Some(&Vec::new()),
                    Some(&Vec::new()),
                    Some("dnn_example".to_string()),
                    None,
                    None,
                    Some(&Vec::new()),
                    Some("smf_serving_area_example".to_string()),
                    Some("mbsmf_serving_area_example".to_string()),
                    None,
                    Some("amf_region_id_example".to_string()),
                    Some("amf_set_id_example".to_string()),
                    None,
                    Some("supi_example".to_string()),
                    Some("ue_ipv4_address_example".to_string()),
                    Some("ip_domain_example".to_string()),
                    None,
                    Some(true),
                    Some(true),
                    Some("pgw_example".to_string()),
                    None,
                    Some("gpsi_example".to_string()),
                    Some("external_group_identity_example".to_string()),
                    Some("internal_group_identity_example".to_string()),
                    None,
                    None,
                    Some("routing_indicator_example".to_string()),
                    Some(&Vec::new()),
                    Some(&Vec::new()),
                    Some(&Vec::new()),
                    Some(&Vec::new()),
                    Some(&Vec::new()),
                    Some("supported_features_example".to_string()),
                    Some(true),
                    None,
                    Some("preferred_locality_example".to_string()),
                    None,
                    Some(56),
                    Some(&Vec::new()),
                    None,
                    Some(56),
                    Some(56),
                    None,
                    Some(true),
                    None,
                    Some("lmf_id_example".to_string()),
                    None,
                    None,
                    None,
                    Some(&Vec::new()),
                    Some("if_none_match_example".to_string()),
                    None,
                    Some(&Vec::new()),
                    None,
                    None,
                    None,
                    None,
                    Some("target_nf_set_id_example".to_string()),
                    Some("target_nf_service_set_id_example".to_string()),
                    Some("nef_id_example".to_string()),
                    None,
                    None,
                    None,
                    Some(&Vec::new()),
                    Some("imsi_example".to_string()),
                    Some("ims_private_identity_example".to_string()),
                    Some("ims_public_identity_example".to_string()),
                    Some("msisdn_example".to_string()),
                    None,
                    Some(true),
                    Some(true),
                    Some(true),
                    Some(true),
                    Some(&Vec::new()),
                    Some("address_domain_example".to_string()),
                    Some("ipv4_addr_example".to_string()),
                    None,
                    Some("served_nf_set_id_example".to_string()),
                    None,
                    None,
                    Some(true),
                    Some(true),
                    Some("requester_features_example".to_string()),
                    Some("realm_id_example".to_string()),
                    Some("storage_id_example".to_string()),
                    Some(true),
                    Some(true),
                    Some("nrf_disc_uri_example".to_string()),
                    None,
                    None,
                    Some("required_pfcp_features_example".to_string()),
                    Some(56),
                    Some(true),
                    Some(true),
                    Some("serving_nf_set_id_example".to_string()),
                    None,
                    Some(&Vec::new()),
                    Some(true),
                    None,
                    Some(&Vec::new()),
                    Some(56),
                    Some("gmlc_number_example".to_string()),
                    None,
                    Some(&Vec::new()),
                    Some(&Vec::new()),
                    Some(true),
                    Some(true),
                    None,
                    None,
                    Some("shared_data_id_example".to_string()),
                    Some("target_hni_example".to_string()),
                    Some(true),
                    Some(&Vec::new()),
                    Some(&Vec::new()),
                    Some(&Vec::new()),
                    Some(&Vec::new()),
                    None,
                ),
            );
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::SCpDomainRoutingInfoGet => {
            let result = rt.block_on(client.scp_domain_routing_info_get(
                Some(true),
                Some("accept_encoding_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        /* Disabled because there's no example.
        Operation::ScpDomainRoutingInfoSubscribe => {
            let result = rt.block_on(client.scp_domain_routing_info_subscribe(
                  ???,
                  Some("content_encoding_example".to_string()),
                  Some("accept_encoding_example".to_string())
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        Operation::RetrieveStoredSearch => {
            let result = rt.block_on(client.retrieve_stored_search(
                "search_id_example".to_string(),
                Some("accept_encoding_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
    }
}
