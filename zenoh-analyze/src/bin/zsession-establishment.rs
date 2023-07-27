#![allow(missing_docs, unused_variables, trivial_casts)]

use clap::Parser;
use log::info;
use nnrf_discovery_server::models::{NfType, ServiceName};
use nnrf_zenoh::NRFApiClient;
use nsfm_pdusession::models::{
    AccessType, Guami, Ncgi, NrLocation, PlmnId, PlmnIdNid, RatType, SmContextCreateData, Snssai,
    Tai, UserLocation,
};
use smf_zenoh::SmfApiClient;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;
use zenoh::prelude::r#async::*;

struct EstErr(String);
#[derive(Parser)]
pub struct Opts {
    // public options
    #[clap(short = 'r', long, default_value = "10")]
    pub runs: usize,
}

async fn establish_session<'a>(
    nrf_client: &Arc<NRFApiClient>,
    smf_client: &Arc<SmfApiClient>,
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
            Some(vec![ServiceName::new("nsmf-pdusession".to_string())]),
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

    info!("{:?}", sfm_search_result,);
    // Send create context request

    // println!("Calling SMF");

    let nas_data = [
        0x2E, 0x01, 0x01, 0xC1, 0xFF, 0xFF, 0x91, 0xA1, 0x28, 0x01, 0x00, 0x7B, 0x00, 0x07, 0x80,
        0x00, 0x0A, 0x00, 0x00, 0x0D, 0x00,
    ];

    let data = SmContextCreateData {
        supi: Some("imsi-001011234567895".into()),
        pei: Some("imeisv-4370816125816151".into()),
        pdu_session_id: Some(1),
        dnn: Some("internet".into()),
        s_nssai: Some(Snssai { sst: 1, sd: None }),
        serving_nf_id: Uuid::from_str("66bf4df8-b832-41ed-aa12-4df3ea315a7c").unwrap(),
        guami: Some(Guami {
            plmn_id: PlmnIdNid {
                mcc: "001".into(),
                mnc: "01".into(),
                nid: None,
            },
            amf_id: "020040".into(),
        }),
        serving_network: PlmnIdNid {
            mcc: "001".into(),
            mnc: "01".into(),
            nid: None,
        },
        an_type: AccessType::Variant3GppAccess,
        rat_type: Some(RatType::new("NR".into())),
        ue_location: Some(UserLocation {
            eutra_location: None,
            n3ga_location: None,
            utra_location: None,
            gera_location: None,
            nr_location: Some(NrLocation {
                tai: Tai {
                    plmn_id: PlmnId {
                        mcc: "001".into(),
                        mnc: "01".into(),
                    },
                    tac: "000001".into(),
                    nid: None,
                },
                ncgi: Ncgi {
                    plmn_id: PlmnId {
                        mcc: "001".into(),
                        mnc: "01".into(),
                    },
                    nr_cell_id: "000000010".into(),
                    nid: None,
                },
                ue_location_timestamp: Some(
                    chrono::DateTime::parse_from_rfc3339("2023-03-01T13:42:11.144288Z")
                        .unwrap()
                        .into(),
                ),
                ignore_ncgi: None,
                age_of_location_information: None,
                geographical_information: None,
                geodetic_information: None,
                global_gnb_id: None,
            }),
        }),
        ue_time_zone: Some("+00:00".into()),
        sm_context_status_uri:
            "http://172.22.0.10:7777/namf-callback/v1/imsi-001011234567895/sm-context-status/1"
                .into(),
        pcf_id: Some(Uuid::from_str("6c05c1d4-b832-41ed-9698-8dec5d3774de").unwrap()),
        unauthenticated_supi: None,
        gpsi: None,
        selected_dnn: None,
        hplmn_snssai: None,
        service_name: None,
        request_type: None,
        n1_sm_msg: None,
        additional_an_type: None,
        presence_in_ladn: None,
        add_ue_location: None,
        h_smf_uri: None,
        h_smf_id: None,
        smf_uri: None,
        smf_id: None,
        additional_hsmf_uri: None,
        additional_hsmf_id: None,
        additional_smf_uri: None,
        additional_smf_id: None,
        old_pdu_session_id: None,
        pdu_sessions_activate_list: None,
        ue_eps_pdn_connection: None,
        ho_state: None,
        pcf_group_id: None,
        pcf_set_id: None,
        nrf_uri: None,
        supported_features: None,
        sel_mode: None,
        backup_amf_info: None,
        trace_data: None,
        udm_group_id: None,
        routing_indicator: None,
        h_nw_pub_key_id: None,
        eps_interworking_ind: None,
        indirect_forwarding_flag: None,
        direct_forwarding_flag: None,
        target_id: None,
        eps_bearer_ctx_status: None,
        cp_ciot_enabled: None,
        cp_only_ind: None,
        invoke_nef: None,
        ma_request_ind: None,
        ma_nw_upgrade_ind: None,
        n2_sm_info: None,
        n2_sm_info_type: None,
        n2_sm_info_ext1: None,
        n2_sm_info_type_ext1: None,
        sm_context_ref: None,
        sm_context_smf_plmn_id: None,
        sm_context_smf_id: None,
        sm_context_smf_set_id: None,
        sm_context_smf_service_set_id: None,
        sm_context_smf_binding: None,
        up_cnx_state: None,
        small_data_rate_status: None,
        apn_rate_status: None,
        extended_nas_sm_timer_ind: None,
        dl_data_waiting_ind: None,
        ddn_failure_subs: None,
        smf_transfer_ind: None,
        old_smf_id: None,
        old_sm_context_ref: None,
        w_agf_info: None,
        tngf_info: None,
        twif_info: None,
        ran_unchanged_ind: None,
        same_pcf_selection_ind: None,
        target_dnai: None,
        nrf_management_uri: None,
        nrf_discovery_uri: None,
        nrf_access_token_uri: None,
        nrf_oauth2_required: None,
        smf_binding_info: None,
        pvs_info: None,
        onboarding_ind: None,
        old_pdu_session_ref: None,
        sm_policy_notify_ind: None,
        pcf_ue_callback_info: None,
        satellite_backhaul_cat: None,
        upip_supported: None,
        uav_authenticated: None,
        disaster_roaming_ind: None,
        anchor_smf_oauth2_required: None,
        sm_context_smf_oauth2_required: None,
    };

    let sm_session_creation_result = smf_client
        .post_sm_contexts(Some(data), Some(Vec::from(nas_data)), None, None)
        .await
        .map_err(|e| EstErr(format!("SMF {e:?}")))?;

    info!("{:?}", sm_session_creation_result,);

    // waits for the callback from AMF
    while let Err(_) = flag.compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed) {}

    Ok(())
}

// rt may be unused if there are no examples
#[allow(unused_mut)]
#[async_std::main]
async fn main() {
    env_logger::init();

    let opts = Opts::parse();
    let runs = opts.runs;
    let flag = Arc::new(AtomicBool::new(false));

    let mut config = zenoh::config::Config::default();
    config
        .set_mode(Some(zenoh::config::whatami::WhatAmI::Peer))
        .unwrap();
    let session = Arc::new(zenoh::open(config).res().await.unwrap());

    let c_flag = flag.clone();
    let c_session = session.clone();
    async_std::task::spawn(async move {
        let amf_ke = "namf-comm/v1/ue-contexts/imsi-001011234567895/n1-n2-messages";
        let sub = c_session.declare_queryable(amf_ke).res().await.unwrap();
        loop {
            sub.recv_async().await.unwrap();
            c_flag.store(true, Ordering::Relaxed);
        }
    });

    async_std::task::sleep(std::time::Duration::from_secs(5)).await;

    let nrfs = NRFApiClient::find_servers(session.clone()).await.unwrap();
    let smfs = SmfApiClient::find_servers(session.clone()).await.unwrap();

    let nrf_client = Arc::new(NRFApiClient::new(session.clone(), nrfs[0]));
    let smf_client = Arc::new(SmfApiClient::new(session.clone(), smfs[0]));

    let mut i = 0;

    while i < runs {
        let now = Instant::now();
        match establish_session(&nrf_client, &smf_client, flag.clone()).await {
            Ok(_) => {
                let delta = now.elapsed();
                println!("establishment,zenoh-rpc,{},ns", delta.as_nanos());
                i += 1;
            }
            Err(_) => (),
        }
    }
}
