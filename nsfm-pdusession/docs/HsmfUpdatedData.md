# HsmfUpdatedData

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**n1_sm_info_to_ue** | [***models::RefToBinaryData**](RefToBinaryData.md) |  | [optional] [default to None]
**n4_info** | [***models::N4Information**](N4Information.md) |  | [optional] [default to None]
**n4_info_ext1** | [***models::N4Information**](N4Information.md) |  | [optional] [default to None]
**n4_info_ext2** | [***models::N4Information**](N4Information.md) |  | [optional] [default to None]
**dnai_list** | **Vec<models::Dnai>** |  | [optional] [default to None]
**supported_features** | **String** | A string used to indicate the features supported by an API that is used as defined in clause  6.6 in 3GPP TS 29.500. The string shall contain a bitmask indicating supported features in  hexadecimal representation Each character in the string shall take a value of \"0\" to \"9\",  \"a\" to \"f\" or \"A\" to \"F\" and shall represent the support of 4 features as described in  table 5.2.2-3. The most significant character representing the highest-numbered features shall  appear first in the string, and the character representing features 1 to 4 shall appear last  in the string. The list of features and their numbering (starting with 1) are defined  separately for each API. If the string contains a lower number of characters than there are  defined features for an API, all features that would be represented by characters that are not  present in the string are not supported.  | [optional] [default to None]
**roaming_charging_profile** | [***models::RoamingChargingProfile**](RoamingChargingProfile.md) |  | [optional] [default to None]
**home_provided_charging_id** | **String** |  | [optional] [default to None]
**up_security** | [***models::UpSecurity**](UpSecurity.md) |  | [optional] [default to None]
**max_integrity_protected_data_rate_ul** | [***models::MaxIntegrityProtectedDataRate**](MaxIntegrityProtectedDataRate.md) |  | [optional] [default to None]
**max_integrity_protected_data_rate_dl** | [***models::MaxIntegrityProtectedDataRate**](MaxIntegrityProtectedDataRate.md) |  | [optional] [default to None]
**ipv6_multi_homing_ind** | **bool** |  | [optional] [default to Some(false)]
**qos_flows_setup_list** | [**Vec<models::QosFlowSetupItem>**](QosFlowSetupItem.md) |  | [optional] [default to None]
**session_ambr** | [***models::Ambr**](Ambr.md) |  | [optional] [default to None]
**eps_pdn_cnx_info** | [***models::EpsPdnCnxInfo**](EpsPdnCnxInfo.md) |  | [optional] [default to None]
**eps_bearer_info** | [**Vec<models::EpsBearerInfo>**](EpsBearerInfo.md) |  | [optional] [default to None]
**pti** | **u8** | Procedure Transaction Identifier | [optional] [default to None]
**inter_plmn_api_root** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]
**intra_plmn_api_root** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


