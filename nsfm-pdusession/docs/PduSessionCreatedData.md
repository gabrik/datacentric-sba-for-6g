# PduSessionCreatedData

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**pdu_session_type** | [***models::PduSessionType**](PduSessionType.md) |  | 
**ssc_mode** | **String** |  | 
**hcn_tunnel_info** | [***models::TunnelInfo**](TunnelInfo.md) |  | [optional] [default to None]
**cn_tunnel_info** | [***models::TunnelInfo**](TunnelInfo.md) |  | [optional] [default to None]
**additional_cn_tunnel_info** | [***models::TunnelInfo**](TunnelInfo.md) |  | [optional] [default to None]
**session_ambr** | [***models::Ambr**](Ambr.md) |  | [optional] [default to None]
**qos_flows_setup_list** | [**Vec<models::QosFlowSetupItem>**](QosFlowSetupItem.md) |  | [optional] [default to None]
**h_smf_instance_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | [optional] [default to None]
**smf_instance_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | [optional] [default to None]
**pdu_session_id** | **u8** | Unsigned integer identifying a PDU session, within the range 0 to 255, as specified in  clause 11.2.3.1b, bits 1 to 8, of 3GPP TS 24.007. If the PDU Session ID is allocated by the  Core Network for UEs not supporting N1 mode, reserved range 64 to 95 is used. PDU Session ID  within the reserved range is only visible in the Core Network.   | [optional] [default to None]
**s_nssai** | [***models::Snssai**](Snssai.md) |  | [optional] [default to None]
**enable_pause_charging** | **bool** |  | [optional] [default to Some(false)]
**ue_ipv4_address** | **String** | String identifying a IPv4 address formatted in the 'dotted decimal' notation as defined in RFC 1166.  | [optional] [default to None]
**ue_ipv6_prefix** | [***models::Ipv6Prefix**](Ipv6Prefix.md) |  | [optional] [default to None]
**n1_sm_info_to_ue** | [***models::RefToBinaryData**](RefToBinaryData.md) |  | [optional] [default to None]
**eps_pdn_cnx_info** | [***models::EpsPdnCnxInfo**](EpsPdnCnxInfo.md) |  | [optional] [default to None]
**eps_bearer_info** | [**Vec<models::EpsBearerInfo>**](EpsBearerInfo.md) |  | [optional] [default to None]
**supported_features** | **String** | A string used to indicate the features supported by an API that is used as defined in clause  6.6 in 3GPP TS 29.500. The string shall contain a bitmask indicating supported features in  hexadecimal representation Each character in the string shall take a value of \"0\" to \"9\",  \"a\" to \"f\" or \"A\" to \"F\" and shall represent the support of 4 features as described in  table 5.2.2-3. The most significant character representing the highest-numbered features shall  appear first in the string, and the character representing features 1 to 4 shall appear last  in the string. The list of features and their numbering (starting with 1) are defined  separately for each API. If the string contains a lower number of characters than there are  defined features for an API, all features that would be represented by characters that are not  present in the string are not supported.  | [optional] [default to None]
**max_integrity_protected_data_rate** | [***models::MaxIntegrityProtectedDataRate**](MaxIntegrityProtectedDataRate.md) |  | [optional] [default to None]
**max_integrity_protected_data_rate_dl** | [***models::MaxIntegrityProtectedDataRate**](MaxIntegrityProtectedDataRate.md) |  | [optional] [default to None]
**always_on_granted** | **bool** |  | [optional] [default to Some(false)]
**gpsi** | **String** | String identifying a Gpsi shall contain either an External Id or an MSISDN.  It shall be formatted as follows -External Identifier= \"extid-'extid', where 'extid'  shall be formatted according to clause 19.7.2 of 3GPP TS 23.003 that describes an  External Identifier.   | [optional] [default to None]
**up_security** | [***models::UpSecurity**](UpSecurity.md) |  | [optional] [default to None]
**roaming_charging_profile** | [***models::RoamingChargingProfile**](RoamingChargingProfile.md) |  | [optional] [default to None]
**h_smf_service_instance_id** | **String** |  | [optional] [default to None]
**smf_service_instance_id** | **String** |  | [optional] [default to None]
**recovery_time** | [**chrono::DateTime::<chrono::Utc>**](DateTime.md) | string with format 'date-time' as defined in OpenAPI. | [optional] [default to None]
**dnai_list** | **Vec<models::Dnai>** |  | [optional] [default to None]
**ipv6_multi_homing_ind** | **bool** |  | [optional] [default to Some(false)]
**ma_accepted_ind** | **bool** |  | [optional] [default to Some(false)]
**home_provided_charging_id** | **String** |  | [optional] [default to None]
**nef_ext_buf_support_ind** | **bool** |  | [optional] [default to Some(false)]
**small_data_rate_control_enabled** | **bool** |  | [optional] [default to Some(false)]
**ue_ipv6_interface_id** | **String** |  | [optional] [default to None]
**ipv6_index** | **i32** | Represents information that identifies which IP pool or external server is used to allocate the IP address.  | [optional] [default to None]
**dn_aaa_address** | [***models::IpAddress**](IpAddress.md) |  | [optional] [default to None]
**redundant_pdu_session_info** | [***models::RedundantPduSessionInformation**](RedundantPduSessionInformation.md) |  | [optional] [default to None]
**nspu_support_ind** | **bool** |  | [optional] [default to None]
**inter_plmn_api_root** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]
**intra_plmn_api_root** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


