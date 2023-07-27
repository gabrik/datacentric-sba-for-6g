# nf_instances_store_api

All URIs are relative to *https://example.com/nnrf-disc/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
**SearchNFInstances**](nf_instances_store_api.md#SearchNFInstances) | **GET** /nf-instances | Search a collection of NF Instances


# **SearchNFInstances**
> models::SearchResult SearchNFInstances(ctx, target_nf_type, requester_nf_type, optional)
Search a collection of NF Instances

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **target_nf_type** | [****](.md)| Type of the target NF | 
  **requester_nf_type** | [****](.md)| Type of the requester NF | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **target_nf_type** | [****](.md)| Type of the target NF | 
 **requester_nf_type** | [****](.md)| Type of the requester NF | 
 **accept_encoding** | **String**| Accept-Encoding, described in IETF RFC 7231 | 
 **preferred_collocated_nf_types** | [**models::CollocatedNfType**](models::CollocatedNfType.md)| collocated NF types that candidate NFs should preferentially support | 
 **requester_nf_instance_id** | [****](.md)| NfInstanceId of the requester NF | 
 **service_names** | [**models::ServiceName**](models::ServiceName.md)| Names of the services offered by the NF | 
 **requester_nf_instance_fqdn** | **String**| FQDN of the requester NF | 
 **target_plmn_list** | [**models::PlmnId**](models::PlmnId.md)| Id of the PLMN of either the target NF, or in SNPN scenario the Credentials Holder in the PLMN  | 
 **requester_plmn_list** | [**models::PlmnId**](models::PlmnId.md)| Id of the PLMN where the NF issuing the Discovery request is located | 
 **target_nf_instance_id** | [****](.md)| Identity of the NF instance being discovered | 
 **target_nf_fqdn** | **String**| FQDN of the NF instance being discovered | 
 **hnrf_uri** | **String**| Uri of the home NRF | 
 **snssais** | [**models::Snssai**](models::Snssai.md)| Slice info of the target NF | 
 **requester_snssais** | [**models::ExtSnssai**](models::ExtSnssai.md)| Slice info of the requester NF | 
 **plmn_specific_snssai_list** | [**models::PlmnSnssai**](models::PlmnSnssai.md)| PLMN specific Slice info of the target NF | 
 **requester_plmn_specific_snssai_list** | [**models::PlmnSnssai**](models::PlmnSnssai.md)| PLMN-specific slice info of the NF issuing the Discovery request | 
 **dnn** | **String**| Dnn supported by the BSF, SMF or UPF | 
 **ipv4_index** | [****](.md)| The IPv4 Index supported by the candidate UPF. | 
 **ipv6_index** | [****](.md)| The IPv6 Index supported by the candidate UPF. | 
 **nsi_list** | [**String**](String.md)| NSI IDs that are served by the services being discovered | 
 **smf_serving_area** | **String**|  | 
 **mbsmf_serving_area** | **String**|  | 
 **tai** | [****](.md)| Tracking Area Identity | 
 **amf_region_id** | **String**| AMF Region Identity | 
 **amf_set_id** | **String**| AMF Set Identity | 
 **guami** | [****](.md)| Guami used to search for an appropriate AMF | 
 **supi** | **String**| SUPI of the user | 
 **ue_ipv4_address** | **String**| IPv4 address of the UE | 
 **ip_domain** | **String**| IP domain of the UE, which supported by BSF | 
 **ue_ipv6_prefix** | [****](.md)| IPv6 prefix of the UE | 
 **pgw_ind** | **bool**| Combined PGW-C and SMF or a standalone SMF | 
 **preferred_pgw_ind** | **bool**| Indicates combined PGW-C+SMF or standalone SMF are preferred | 
 **pgw** | **String**| PGW FQDN of a combined PGW-C and SMF | 
 **pgw_ip** | [****](.md)| PGW IP Address of a combined PGW-C and SMF | 
 **gpsi** | **String**| GPSI of the user | 
 **external_group_identity** | **String**| external group identifier of the user | 
 **internal_group_identity** | **String**| internal group identifier of the user | 
 **pfd_data** | [****](.md)| PFD data | 
 **data_set** | [****](.md)| data set supported by the NF | 
 **routing_indicator** | **String**| routing indicator in SUCI | 
 **group_id_list** | [**String**](String.md)| Group IDs of the NFs being discovered | 
 **dnai_list** | [**String**](String.md)| Data network access identifiers of the NFs being discovered | 
 **pdu_session_types** | [**models::PduSessionType**](models::PduSessionType.md)| list of PDU Session Type required to be supported by the target NF | 
 **event_id_list** | [**models::EventId**](models::EventId.md)| Analytics event(s) requested to be supported by the Nnwdaf_AnalyticsInfo service  | 
 **nwdaf_event_list** | [**models::NwdafEvent**](models::NwdafEvent.md)| Analytics event(s) requested to be supported by the Nnwdaf_EventsSubscription service.  | 
 **supported_features** | **String**| Features required to be supported by the target NF | 
 **upf_iwk_eps_ind** | **bool**| UPF supporting interworking with EPS or not | 
 **chf_supported_plmn** | [****](.md)| PLMN ID supported by a CHF | 
 **preferred_locality** | **String**| preferred target NF location | 
 **access_type** | [****](.md)| AccessType supported by the target NF | 
 **limit** | **i32**| Maximum number of NFProfiles to return in the response | 
 **required_features** | [**String**](String.md)| Features required to be supported by the target NF | 
 **complex_query** | [****](.md)| the complex query condition expression | 
 **max_payload_size** | **i32**| Maximum payload size of the response expressed in kilo octets | [default to 124]
 **max_payload_size_ext** | **i32**| Extended query for maximum payload size of the response expressed in kilo octets  | [default to 124]
 **atsss_capability** | [****](.md)| ATSSS Capability | 
 **upf_ue_ip_addr_ind** | **bool**| UPF supporting allocating UE IP addresses/prefixes | 
 **client_type** | [****](.md)| Requested client type served by the NF | 
 **lmf_id** | **String**| LMF identification to be discovered | 
 **an_node_type** | [****](.md)| Requested AN node type served by the NF | 
 **rat_type** | [****](.md)| Requested RAT type served by the NF | 
 **preferred_tai** | [****](.md)| preferred Tracking Area Identity | 
 **preferred_nf_instances** | [**uuid::Uuid**](uuid::Uuid.md)| preferred NF Instances | 
 **if_none_match** | **String**| Validator for conditional requests, as described in IETF RFC 7232, 3.2 | 
 **target_snpn** | [****](.md)| Target SNPN Identity, or the Credentials Holder in the SNPN | 
 **requester_snpn_list** | [**models::PlmnIdNid**](models::PlmnIdNid.md)| SNPN ID(s) of the NF instance issuing the Discovery request | 
 **af_ee_data** | [****](.md)| NEF exposured by the AF | 
 **w_agf_info** | [****](.md)| UPF collocated with W-AGF | 
 **tngf_info** | [****](.md)| UPF collocated with TNGF | 
 **twif_info** | [****](.md)| UPF collocated with TWIF | 
 **target_nf_set_id** | **String**| Target NF Set ID | 
 **target_nf_service_set_id** | **String**| Target NF Service Set ID | 
 **nef_id** | **String**| NEF ID | 
 **notification_type** | [****](.md)| Notification Type | 
 **n1_msg_class** | [****](.md)| N1 Message Class | 
 **n2_info_class** | [****](.md)| N2 Information Class | 
 **serving_scope** | [**String**](String.md)| areas that can be served by the target NF | 
 **imsi** | **String**| IMSI of the requester UE to search for an appropriate NF (e.g. HSS) | 
 **ims_private_identity** | **String**| IMPI of the requester UE to search for a target HSS | 
 **ims_public_identity** | **String**| IMS Public Identity of the requester UE to search for a target HSS | 
 **msisdn** | **String**| MSISDN of the requester UE to search for a target HSS | 
 **preferred_api_versions** | [**String**](String.md)| Preferred API version of the services to be discovered | 
 **v2x_support_ind** | **bool**| PCF supports V2X | 
 **redundant_gtpu** | **bool**| UPF supports redundant gtp-u to be discovered | 
 **redundant_transport** | **bool**| UPF supports redundant transport path to be discovered | 
 **ipups** | **bool**| UPF which is configured for IPUPS functionality to be discovered | 
 **scp_domain_list** | [**String**](String.md)| SCP domains the target SCP or SEPP belongs to | 
 **address_domain** | **String**| Address domain reachable through the SCP | 
 **ipv4_addr** | **String**| IPv4 address reachable through the SCP | 
 **ipv6_prefix** | [****](.md)| IPv6 prefix reachable through the SCP | 
 **served_nf_set_id** | **String**| NF Set ID served by the SCP | 
 **remote_plmn_id** | [****](.md)| Id of the PLMN reachable through the SCP or SEPP | 
 **remote_snpn_id** | [****](.md)| Id of the SNPN reachable through the SCP or SEPP | 
 **data_forwarding** | **bool**| UPF Instance(s) configured for data forwarding are requested | 
 **preferred_full_plmn** | **bool**| NF Instance(s) serving the full PLMN are preferred | 
 **requester_features** | **String**| Features supported by the NF Service Consumer that is invoking the Nnrf_NFDiscovery service  | 
 **realm_id** | **String**| realm-id to search for an appropriate UDSF | 
 **storage_id** | **String**| storage-id to search for an appropriate UDSF | 
 **vsmf_support_ind** | **bool**| V-SMF capability supported by the target NF instance(s) | 
 **ismf_support_ind** | **bool**| I-SMF capability supported by the target NF instance(s) | 
 **nrf_disc_uri** | **String**| Uri of the NRF holding the NF profile of a target NF Instance | 
 **preferred_vendor_specific_features** | [**std::collections::HashMap<String, Vec<models::VendorSpecificFeature>>**](std::collections::HashMap<String, Vec<models::VendorSpecificFeature>>.md)| Preferred vendor specific features of the services to be discovered | 
 **preferred_vendor_specific_nf_features** | [**Vec<models::VendorSpecificFeature>**](Vec<models::VendorSpecificFeature>.md)| Preferred vendor specific features of the network function to be discovered | 
 **required_pfcp_features** | **String**| PFCP features required to be supported by the target UPF | 
 **home_pub_key_id** | **i32**| Indicates the Home Network Public Key ID which shall be able to be served by the NF instance  | 
 **prose_support_ind** | **bool**| PCF supports ProSe Capability | 
 **analytics_aggregation_ind** | **bool**| analytics aggregation is supported by NWDAF or not | 
 **serving_nf_set_id** | **String**| NF Set Id served by target NF | 
 **serving_nf_type** | [****](.md)| NF type served by the target NF | 
 **ml_analytics_info_list** | [**models::MlAnalyticsInfo**](models::MlAnalyticsInfo.md)| Lisf of ML Analytics Filter information of Nnwdaf_MLModelProvision service | 
 **analytics_metadata_prov_ind** | **bool**| analytics matadata provisioning is supported by NWDAF or not | 
 **nsacf_capability** | [****](.md)| the service capability supported by the target NSACF | 
 **mbs_session_id_list** | [**models::MbsSessionId**](models::MbsSessionId.md)| List of MBS Session ID(s) | 
 **area_session_id** | **i32**| Area Session ID | 
 **gmlc_number** | **String**| The GMLC Number supported by the GMLC | 
 **upf_n6_ip** | [****](.md)| N6 IP address of PSA UPF supported by the EASDF | 
 **tai_list** | [**models::Tai**](models::Tai.md)| Tracking Area Identifiers of the NFs being discovered | 
 **preferences_precedence** | [**String**](String.md)| Indicates the precedence of the preference query parameters (from higher to lower)  | 
 **support_onboarding_capability** | **bool**| Indicating the support for onboarding. | [default to false]
 **uas_nf_functionality_ind** | **bool**| UAS NF functionality is supported by NEF or not | 
 **v2x_capability** | [****](.md)| indicates the V2X capability that the target PCF needs to support. | 
 **prose_capability** | [****](.md)| indicates the ProSe capability that the target PCF needs to support. | 
 **shared_data_id** | **String**| Identifier of shared data stored in the NF being discovered | 
 **target_hni** | **String**| Home Network Identifier query. | 
 **target_nw_resolution** | **bool**| Resolution of the identity of the target PLMN based on the GPSI of the UE | 
 **exclude_nfinst_list** | [**uuid::Uuid**](uuid::Uuid.md)| NF Instance IDs to be excluded from the NF Discovery procedure | 
 **exclude_nfservinst_list** | [**models::NfServiceInstance**](models::NfServiceInstance.md)| NF service instance IDs to be excluded from the NF Discovery procedure | 
 **exclude_nfserviceset_list** | [**String**](String.md)| NF Service Set IDs to be excluded from the NF Discovery procedure | 
 **exclude_nfset_list** | [**String**](String.md)| NF Set IDs to be excluded from the NF Discovery procedure | 
 **preferred_analytics_delays** | [**i32**](i32.md)| Preferred analytics delays supported by the NWDAF to be discovered | 

### Return type

[**models::SearchResult**](SearchResult.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

