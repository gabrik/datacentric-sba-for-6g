# VsmfUpdateData

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**request_indication** | [***models::RequestIndication**](RequestIndication.md) |  | 
**session_ambr** | [***models::Ambr**](Ambr.md) |  | [optional] [default to None]
**qos_flows_add_mod_request_list** | [**Vec<models::QosFlowAddModifyRequestItem>**](QosFlowAddModifyRequestItem.md) |  | [optional] [default to None]
**qos_flows_rel_request_list** | [**Vec<models::QosFlowReleaseRequestItem>**](QosFlowReleaseRequestItem.md) |  | [optional] [default to None]
**eps_bearer_info** | [**Vec<models::EpsBearerInfo>**](EpsBearerInfo.md) |  | [optional] [default to None]
**assign_ebi_list** | [**Vec<models::Arp>**](Arp.md) |  | [optional] [default to None]
**revoke_ebi_list** | **Vec<models::EpsBearerId>** |  | [optional] [default to None]
**modified_ebi_list** | [**Vec<models::EbiArpMapping>**](EbiArpMapping.md) |  | [optional] [default to None]
**pti** | **u8** | Procedure Transaction Identifier | [optional] [default to None]
**n1_sm_info_to_ue** | [***models::RefToBinaryData**](RefToBinaryData.md) |  | [optional] [default to None]
**always_on_granted** | **bool** |  | [optional] [default to Some(false)]
**hsmf_pdu_session_uri** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]
**new_smf_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | [optional] [default to None]
**new_smf_pdu_session_uri** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]
**supported_features** | **String** | A string used to indicate the features supported by an API that is used as defined in clause  6.6 in 3GPP TS 29.500. The string shall contain a bitmask indicating supported features in  hexadecimal representation Each character in the string shall take a value of \"0\" to \"9\",  \"a\" to \"f\" or \"A\" to \"F\" and shall represent the support of 4 features as described in  table 5.2.2-3. The most significant character representing the highest-numbered features shall  appear first in the string, and the character representing features 1 to 4 shall appear last  in the string. The list of features and their numbering (starting with 1) are defined  separately for each API. If the string contains a lower number of characters than there are  defined features for an API, all features that would be represented by characters that are not  present in the string are not supported.  | [optional] [default to None]
**cause** | [***models::Cause**](Cause.md) |  | [optional] [default to None]
**n1sm_cause** | **String** |  | [optional] [default to None]
**back_off_timer** | **i32** | indicating a time in seconds. | [optional] [default to None]
**ma_release_ind** | [***models::MaReleaseIndication**](MaReleaseIndication.md) |  | [optional] [default to None]
**ma_accepted_ind** | **bool** |  | [optional] [default to Some(false)]
**additional_cn_tunnel_info** | [***models::TunnelInfo**](TunnelInfo.md) |  | [optional] [default to None]
**dnai_list** | **Vec<models::Dnai>** |  | [optional] [default to None]
**n4_info** | [***models::N4Information**](N4Information.md) |  | [optional] [default to None]
**n4_info_ext1** | [***models::N4Information**](N4Information.md) |  | [optional] [default to None]
**n4_info_ext2** | [***models::N4Information**](N4Information.md) |  | [optional] [default to None]
**n4_info_ext3** | [***models::N4Information**](N4Information.md) |  | [optional] [default to None]
**small_data_rate_control_enabled** | **bool** |  | [optional] [default to None]
**qos_monitoring_info** | [***models::QosMonitoringInfo**](QosMonitoringInfo.md) |  | [optional] [default to None]
**eps_pdn_cnx_info** | [***models::EpsPdnCnxInfo**](EpsPdnCnxInfo.md) |  | [optional] [default to None]
**n9_data_forwarding_ind** | **bool** |  | [optional] [default to Some(false)]
**n9_inactivity_timer** | **i32** | indicating a time in seconds. | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


