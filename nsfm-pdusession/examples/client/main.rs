#![allow(missing_docs, unused_variables, trivial_casts)]

use clap::{Parser, ValueEnum};
#[allow(unused_imports)]
use futures::{future, stream, Stream};
#[allow(unused_imports)]
use nsfm_pdusession::{
    models, Api, ApiNoContext, Client, ContextWrapperExt, PostPduSessionsResponse,
    PostSmContextsResponse, ReleasePduSessionResponse, ReleaseSmContextResponse,
    RetrievePduSessionResponse, RetrieveSmContextResponse, SendMoDataResponse,
    TransferMoDataResponse, UpdatePduSessionResponse, UpdateSmContextResponse,
};

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
    #[clap(short = 'c', long, default_value = "http://127.0.0.1:8082")]
    pub connect: String,
    #[clap(short = 'o', long)]
    pub operation: Operation,
}

#[derive(ValueEnum, Clone)]
pub enum Operation {
    ReleasePduSession,
    TransferMoData,
    ReleaseSmContext,
    RetrieveSmContext,
    SendMoData,
    PostSmContexts,
}

// rt may be unused if there are no examples
#[allow(unused_mut)]
fn main() {
    env_logger::init();

    let opts = Opts::parse();

    let base_url = url::Url::parse(&opts.connect).unwrap();

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
        Operation::ReleasePduSession => {
            let result = rt
                .block_on(client.release_pdu_session("pdu_session_ref_example".to_string(), None));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        /* Disabled because there's no example.
         Operation::RetrievePduSession => {
            let result = rt.block_on(client.retrieve_pdu_session(
                  "pdu_session_ref_example".to_string(),
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        Operation::TransferMoData => {
            let result = rt.block_on(client.transfer_mo_data(
                "pdu_session_ref_example".to_string(),
                None,
                Some(swagger::ByteArray(Vec::from("BINARY_DATA_HERE"))),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        /* Disabled because there's no example.
         Operation::UpdatePduSession => {
            let result = rt.block_on(client.update_pdu_session(
                  "pdu_session_ref_example".to_string(),
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        Operation::ReleaseSmContext => {
            let result =
                rt.block_on(client.release_sm_context("sm_context_ref_example".to_string(), None));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::RetrieveSmContext => {
            let result =
                rt.block_on(client.retrieve_sm_context("sm_context_ref_example".to_string(), None));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::SendMoData => {
            let result = rt.block_on(client.send_mo_data(
                "sm_context_ref_example".to_string(),
                None,
                Some(swagger::ByteArray(Vec::from("BINARY_DATA_HERE"))),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        /* Disabled because there's no example.
         Operation::UpdateSmContext => {
            let result = rt.block_on(client.update_sm_context(
                  "sm_context_ref_example".to_string(),
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
         Operation::PostPduSessions => {
            let result = rt.block_on(client.post_pdu_sessions(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        Operation::PostSmContexts => {
            let result = rt.block_on(client.post_sm_contexts(
                None,
                Some(swagger::ByteArray(Vec::from("BINARY_DATA_HERE"))),
                Some(swagger::ByteArray(Vec::from("BINARY_DATA_HERE"))),
                Some(swagger::ByteArray(Vec::from("BINARY_DATA_HERE"))),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
    }
}
