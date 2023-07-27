# SmContextUpdatedData

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**up_cnx_state** | [***models::UpCnxState**](UpCnxState.md) |  | [optional] [default to None]
**ho_state** | [***models::HoState**](HoState.md) |  | [optional] [default to None]
**release_ebi_list** | **Vec<models::EpsBearerId>** |  | [optional] [default to None]
**allocated_ebi_list** | [**Vec<models::EbiArpMapping>**](EbiArpMapping.md) |  | [optional] [default to None]
**modified_ebi_list** | [**Vec<models::EbiArpMapping>**](EbiArpMapping.md) |  | [optional] [default to None]
**n1_sm_msg** | [***models::RefToBinaryData**](RefToBinaryData.md) |  | [optional] [default to None]
**n2_sm_info** | [***models::RefToBinaryData**](RefToBinaryData.md) |  | [optional] [default to None]
**n2_sm_info_type** | [***models::N2SmInfoType**](N2SmInfoType.md) |  | [optional] [default to None]
**eps_bearer_setup** | **Vec<models::EpsBearerContainer>** |  | [optional] [default to None]
**data_forwarding** | **bool** |  | [optional] [default to None]
**n3_dl_forwarding_tnl_list** | [**Vec<models::IndirectDataForwardingTunnelInfo>**](IndirectDataForwardingTunnelInfo.md) |  | [optional] [default to None]
**n3_ul_forwarding_tnl_list** | [**Vec<models::IndirectDataForwardingTunnelInfo>**](IndirectDataForwardingTunnelInfo.md) |  | [optional] [default to None]
**n9_ul_forwarding_tunnel** | [***models::TunnelInfo**](TunnelInfo.md) |  | [optional] [default to None]
**cause** | [***models::Cause**](Cause.md) |  | [optional] [default to None]
**ma_accepted_ind** | **bool** |  | [optional] [default to Some(false)]
**supported_features** | **String** | A string used to indicate the features supported by an API that is used as defined in clause  6.6 in 3GPP TS 29.500. The string shall contain a bitmask indicating supported features in  hexadecimal representation Each character in the string shall take a value of \"0\" to \"9\",  \"a\" to \"f\" or \"A\" to \"F\" and shall represent the support of 4 features as described in  table 5.2.2-3. The most significant character representing the highest-numbered features shall  appear first in the string, and the character representing features 1 to 4 shall appear last  in the string. The list of features and their numbering (starting with 1) are defined  separately for each API. If the string contains a lower number of characters than there are  defined features for an API, all features that would be represented by characters that are not  present in the string are not supported.  | [optional] [default to None]
**forwarding_f_teid** | [***swagger::ByteArray**](ByteArray.md) | string with format 'bytes' as defined in OpenAPI | [optional] [default to None]
**forwarding_bearer_contexts** | **Vec<models::ForwardingBearerContainer>** |  | [optional] [default to None]
**selected_smf_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | [optional] [default to None]
**selected_old_smf_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | [optional] [default to None]
**inter_plmn_api_root** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]
**anchor_smf_features** | [***models::AnchorSmfFeatures**](AnchorSmfFeatures.md) |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


