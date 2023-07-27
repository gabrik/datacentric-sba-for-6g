#![allow(missing_docs, unused_variables, trivial_casts)]

use clap::{Parser, ValueEnum};
#[allow(unused_imports)]
use futures::{future, stream, Stream};
#[allow(unused_imports)]
use nudm_sdm::{
    models, Api, ApiNoContext, CAgAckResponse, Client, ContextWrapperExt, GetAmDataResponse,
    GetDataSetsResponse, GetEcrDataResponse, GetGroupIdentifiersResponse,
    GetIndividualSharedDataResponse, GetLcsBcaDataResponse, GetLcsMoDataResponse,
    GetLcsPrivacyDataResponse, GetMbsDataResponse, GetMultipleIdentifiersResponse,
    GetNssaiResponse, GetProseDataResponse, GetSharedDataResponse, GetSmDataResponse,
    GetSmfSelDataResponse, GetSmsDataResponse, GetSmsMngtDataResponse, GetSupiOrGpsiResponse,
    GetTraceConfigDataResponse, GetUcDataResponse, GetUeCtxInAmfDataResponse,
    GetUeCtxInSmfDataResponse, GetUeCtxInSmsfDataResponse, GetV2xDataResponse, ModifyResponse,
    ModifySharedDataSubsResponse, SNssaisAckResponse, SorAckInfoResponse, SubscribeResponse,
    SubscribeToSharedDataResponse, UnsubscribeForSharedDataResponse, UnsubscribeResponse,
    UpdateSorInfoResponse, UpuAckResponse,
};

#[allow(unused_imports)]
use log::info;

// swagger::Has may be unused if there are no examples
#[allow(unused_imports)]
use swagger::{AuthData, ContextBuilder, EmptyContext, Has, Push, XSpanIdString};
use url::Url;

type ClientContext = swagger::make_context_ty!(
    ContextBuilder,
    EmptyContext,
    Option<AuthData>,
    XSpanIdString
);

#[derive(Parser)]
pub struct Opts {
    // public options
    #[clap(short = 'c', long, default_value = "http://127.0.0.1:8081")]
    pub connect: String,
    #[clap(short = 'o', long)]
    pub operation: Operation,
}

#[derive(ValueEnum, Clone)]
pub enum Operation {
    GetAmData,
    GetMbsData,
    GetEcrData,
    GetSupiOrGpsi,
    GetGroupIdentifiers,
    GetLcsBcaData,
    GetLcsMoData,
    GetLcsPrivacyData,
    GetMultipleIdentifiers,
    GetProseData,
    CAgAck,
    SNssaisAck,
    SorAckInfo,
    UpuAck,
    GetDataSets,
    GetSharedData,
    GetIndividualSharedData,
    GetSmfSelData,
    GetSmsMngtData,
    GetSmsData,
    GetSmData,
    GetNssai,
    Unsubscribe,
    UnsubscribeForSharedData,
    GetTraceConfigData,
    UpdateSorInfo,
    GetUeCtxInAmfData,
    GetUeCtxInSmfData,
    GetUeCtxInSmsfData,
    GetUcData,
    GetV2xData,
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
        Operation::GetAmData => {
            let result = rt.block_on(client.get_am_data(
                "supi_example".to_string(),
                Some("supported_features_example".to_string()),
                None,
                Some(&Vec::new()),
                Some(true),
                Some("if_none_match_example".to_string()),
                Some("if_modified_since_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetMbsData => {
            let result = rt.block_on(client.get_mbs_data(
                "supi_example".to_string(),
                Some("supported_features_example".to_string()),
                Some("if_none_match_example".to_string()),
                Some("if_modified_since_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetEcrData => {
            let result = rt.block_on(client.get_ecr_data(
                "supi_example".to_string(),
                Some("supported_features_example".to_string()),
                Some("if_none_match_example".to_string()),
                Some("if_modified_since_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetSupiOrGpsi => {
            let result = rt.block_on(client.get_supi_or_gpsi(
                "ue_id_example".to_string(),
                Some("supported_features_example".to_string()),
                Some("af_id_example".to_string()),
                None,
                Some("af_service_id_example".to_string()),
                Some("mtc_provider_info_example".to_string()),
                None,
                Some("if_none_match_example".to_string()),
                Some("if_modified_since_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetGroupIdentifiers => {
            let result = rt.block_on(client.get_group_identifiers(
                Some("ext_group_id_example".to_string()),
                Some("int_group_id_example".to_string()),
                Some(true),
                Some("supported_features_example".to_string()),
                Some("af_id_example".to_string()),
                Some("if_none_match_example".to_string()),
                Some("if_modified_since_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetLcsBcaData => {
            let result = rt.block_on(client.get_lcs_bca_data(
                "supi_example".to_string(),
                Some("supported_features_example".to_string()),
                None,
                Some("if_none_match_example".to_string()),
                Some("if_modified_since_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetLcsMoData => {
            let result = rt.block_on(client.get_lcs_mo_data(
                "supi_example".to_string(),
                Some("supported_features_example".to_string()),
                Some("if_none_match_example".to_string()),
                Some("if_modified_since_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetLcsPrivacyData => {
            let result = rt.block_on(client.get_lcs_privacy_data(
                "ue_id_example".to_string(),
                Some("supported_features_example".to_string()),
                Some("if_none_match_example".to_string()),
                Some("if_modified_since_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetMultipleIdentifiers => {
            let result = rt.block_on(client.get_multiple_identifiers(
                &Vec::new(),
                Some("supported_features_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetProseData => {
            let result = rt.block_on(client.get_prose_data(
                "supi_example".to_string(),
                Some("supported_features_example".to_string()),
                Some("if_none_match_example".to_string()),
                Some("if_modified_since_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::CAgAck => {
            let result = rt.block_on(client.cag_ack("supi_example".to_string(), None));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::SNssaisAck => {
            let result = rt.block_on(client.s_nssais_ack("supi_example".to_string(), None));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::SorAckInfo => {
            let result = rt.block_on(client.sor_ack_info("supi_example".to_string(), None));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::UpuAck => {
            let result = rt.block_on(client.upu_ack("supi_example".to_string(), None));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetDataSets => {
            let result = rt.block_on(client.get_data_sets(
                "supi_example".to_string(),
                &Vec::new(),
                None,
                Some(true),
                Some("supported_features_example".to_string()),
                Some("if_none_match_example".to_string()),
                Some("if_modified_since_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetSharedData => {
            let result = rt.block_on(client.get_shared_data(
                &Vec::new(),
                Some("supported_features_example".to_string()),
                Some("supported_features_example".to_string()),
                Some("if_none_match_example".to_string()),
                Some("if_modified_since_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetIndividualSharedData => {
            let result = rt.block_on(client.get_individual_shared_data(
                &Vec::new(),
                Some("supported_features_example".to_string()),
                Some("if_none_match_example".to_string()),
                Some("if_modified_since_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetSmfSelData => {
            let result = rt.block_on(client.get_smf_sel_data(
                "supi_example".to_string(),
                Some("supported_features_example".to_string()),
                None,
                Some(true),
                Some("if_none_match_example".to_string()),
                Some("if_modified_since_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetSmsMngtData => {
            let result = rt.block_on(client.get_sms_mngt_data(
                "supi_example".to_string(),
                Some("supported_features_example".to_string()),
                None,
                Some("if_none_match_example".to_string()),
                Some("if_modified_since_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetSmsData => {
            let result = rt.block_on(client.get_sms_data(
                "supi_example".to_string(),
                Some("supported_features_example".to_string()),
                None,
                Some("if_none_match_example".to_string()),
                Some("if_modified_since_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetSmData => {
            let result = rt.block_on(client.get_sm_data(
                "supi_example".to_string(),
                Some("supported_features_example".to_string()),
                None,
                Some("dnn_example".to_string()),
                None,
                Some("if_none_match_example".to_string()),
                Some("if_modified_since_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetNssai => {
            let result = rt.block_on(client.get_nssai(
                "supi_example".to_string(),
                Some("supported_features_example".to_string()),
                None,
                Some(true),
                Some("if_none_match_example".to_string()),
                Some("if_modified_since_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        /* Disabled because there's no example.
        Some("Subscribe => {
            let result = rt.block_on(client.subscribe(
                  "ue_id_example".to_string(),
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("SubscribeToSharedData => {
            let result = rt.block_on(client.subscribe_to_shared_data(
                  ???
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        Operation::Unsubscribe => {
            let result = rt.block_on(client.unsubscribe(
                "ue_id_example".to_string(),
                "subscription_id_example".to_string(),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::UnsubscribeForSharedData => {
            let result = rt.block_on(
                client.unsubscribe_for_shared_data("subscription_id_example".to_string()),
            );
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        /* Disabled because there's no example.
        Some("Modify => {
            let result = rt.block_on(client.modify(
                  "ue_id_example".to_string(),
                  "subscription_id_example".to_string(),
                  ???,
                  Some("supported_features_example".to_string())
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        /* Disabled because there's no example.
        Some("ModifySharedDataSubs => {
            let result = rt.block_on(client.modify_shared_data_subs(
                  "subscription_id_example".to_string(),
                  ???,
                  Some("supported_features_example".to_string())
            ));
            info!("{:?} (X-Span-ID: {:?})", result, (client.context() as &dyn Has<XSpanIdString>).get().clone());
        },
        */
        Operation::GetTraceConfigData => {
            let result = rt.block_on(client.get_trace_config_data(
                "supi_example".to_string(),
                Some("supported_features_example".to_string()),
                None,
                Some("if_none_match_example".to_string()),
                Some("if_modified_since_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::UpdateSorInfo => {
            let result = rt.block_on(client.update_sor_info("supi_example".to_string(), None));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetUeCtxInAmfData => {
            let result = rt.block_on(client.get_ue_ctx_in_amf_data(
                "supi_example".to_string(),
                Some("supported_features_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetUeCtxInSmfData => {
            let result = rt.block_on(client.get_ue_ctx_in_smf_data(
                "supi_example".to_string(),
                Some("supported_features_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetUeCtxInSmsfData => {
            let result = rt.block_on(client.get_ue_ctx_in_smsf_data(
                "supi_example".to_string(),
                Some("supported_features_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetUcData => {
            let result = rt.block_on(client.get_uc_data(
                "supi_example".to_string(),
                Some("supported_features_example".to_string()),
                None,
                Some("if_none_match_example".to_string()),
                Some("if_modified_since_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
        Operation::GetV2xData => {
            let result = rt.block_on(client.get_v2x_data(
                "supi_example".to_string(),
                Some("supported_features_example".to_string()),
                Some("if_none_match_example".to_string()),
                Some("if_modified_since_example".to_string()),
            ));
            info!(
                "{:?} (X-Span-ID: {:?})",
                result,
                (client.context() as &dyn Has<XSpanIdString>).get().clone()
            );
        }
    }
}
