use tonic::{Request, Response, Status};

use nnrf_disc::nrf_discovery_server::NrfDiscovery;
use nnrf_disc::{IpEndpoints, NfProfile, NfService, NfVersion, SearchRequest, SearchResult};

use std::collections::HashMap;

pub mod nnrf_disc {
    tonic::include_proto!("fiveg_proto"); // The string specified here must match the proto package name
}

#[derive(Debug, Default)]
pub struct MyNRF {}

#[tonic::async_trait]
impl NrfDiscovery for MyNRF {
    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResult>, Status> {
        let req: SearchRequest = request.into_inner();

        let reply = if req.target_nf_type == "UDM" {
            SearchResult {
                validity_period: 3600,
                nf_instances: vec![NfProfile {
                    nf_instance_id: "65a52dac-b832-41ed-ba6e-53c1c0b3ed51".into(),
                    nf_type: "UDM".into(),
                    nf_status: "REGISTERED".into(),
                    heartbeat_timer: 10,
                    ipv4_addresses: vec!["172.22.0.13".into()],
                    allowed_nf_types: vec!["AMF".into(), "SMF".into(), "AUSF".into(), "SCP".into()],
                    priority: 0,
                    capacity: 100,
                    load: 0,
                    nf_service_list: HashMap::from([(
                        "65a54148-b832-41ed-ba6e-53c1c0b3ed51".into(),
                        NfService {
                            service_instance_id: "65a54148-b832-41ed-ba6e-53c1c0b3ed51".into(),
                            service_name: "nudm-sdm".into(),
                            versions: vec![NfVersion {
                                api_version_in_url: "v2".into(),
                                api_full_version: "2.0.0".into(),
                            }],
                            scheme: "http".into(),
                            nf_service_status: "REGISTERED".into(),
                            ip_endpoints: vec![IpEndpoints {
                                ipv4_address: "172.22.0.13".into(),
                                port: 7777,
                            }],
                            allowed_nf_types: vec!["AMF".into(), "SMF".into()],
                            priority: 0,
                            capacity: 100,
                            load: 0,
                        },
                    )]),
                    nf_profile_changes_support_ind: true,
                }],
            }
        } else if req.target_nf_type == "SMF" {
            SearchResult {
                validity_period: 3600,
                nf_instances: vec![NfProfile {
                    nf_instance_id: "b3a71a80-b8d5-41ed-b2cc-8bbc6f173b7d".into(),
                    nf_type: "SMF".into(),
                    nf_status: "REGISTERED".into(),
                    heartbeat_timer: 10,
                    ipv4_addresses: vec!["172.22.0.7".into()],
                    allowed_nf_types: vec!["AMF".into(), "SCP".into()],
                    priority: 0,
                    capacity: 100,
                    load: 0,
                    nf_service_list: HashMap::from([(
                        "b3a71a80-b8d5-41ed-b2cc-8bbc6f173b7d".into(),
                        NfService {
                            service_instance_id: "b3a71a80-b8d5-41ed-b2cc-8bbc6f173b7d".into(),
                            service_name: "nsmf-pdusession".into(),
                            versions: vec![NfVersion {
                                api_version_in_url: "v1".into(),
                                api_full_version: "1.0.0".into(),
                            }],
                            scheme: "http".into(),
                            nf_service_status: "REGISTERED".into(),
                            ip_endpoints: vec![IpEndpoints {
                                ipv4_address: "172.22.0.7".into(),
                                port: 7777,
                            }],
                            allowed_nf_types: vec!["AMF".into()],
                            priority: 0,
                            capacity: 100,
                            load: 0,
                        },
                    )]),
                    nf_profile_changes_support_ind: true,
                }],
            }
        } else {
            panic!("Neither UDM nor SMF, other are not mockup");
        };

        Ok(Response::new(reply))
    }
}
