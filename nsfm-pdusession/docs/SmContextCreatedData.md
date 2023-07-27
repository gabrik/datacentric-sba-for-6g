# SmContextCreatedData

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**h_smf_uri** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]
**smf_uri** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]
**pdu_session_id** | **u8** | Unsigned integer identifying a PDU session, within the range 0 to 255, as specified in  clause 11.2.3.1b, bits 1 to 8, of 3GPP TS 24.007. If the PDU Session ID is allocated by the  Core Network for UEs not supporting N1 mode, reserved range 64 to 95 is used. PDU Session ID  within the reserved range is only visible in the Core Network.   | [optional] [default to None]
**s_nssai** | [***models::Snssai**](Snssai.md) |  | [optional] [default to None]
**up_cnx_state** | [***models::UpCnxState**](UpCnxState.md) |  | [optional] [default to None]
**n2_sm_info** | [***models::RefToBinaryData**](RefToBinaryData.md) |  | [optional] [default to None]
**n2_sm_info_type** | [***models::N2SmInfoType**](N2SmInfoType.md) |  | [optional] [default to None]
**allocated_ebi_list** | [**Vec<models::EbiArpMapping>**](EbiArpMapping.md) |  | [optional] [default to None]
**ho_state** | [***models::HoState**](HoState.md) |  | [optional] [default to None]
**gpsi** | **String** | String identifying a Gpsi shall contain either an External Id or an MSISDN.  It shall be formatted as follows -External Identifier= \"extid-'extid', where 'extid'  shall be formatted according to clause 19.7.2 of 3GPP TS 23.003 that describes an  External Identifier.   | [optional] [default to None]
**smf_service_instance_id** | **String** |  | [optional] [default to None]
**recovery_time** | [**chrono::DateTime::<chrono::Utc>**](DateTime.md) | string with format 'date-time' as defined in OpenAPI. | [optional] [default to None]
**supported_features** | **String** | A string used to indicate the features supported by an API that is used as defined in clause  6.6 in 3GPP TS 29.500. The string shall contain a bitmask indicating supported features in  hexadecimal representation Each character in the string shall take a value of \"0\" to \"9\",  \"a\" to \"f\" or \"A\" to \"F\" and shall represent the support of 4 features as described in  table 5.2.2-3. The most significant character representing the highest-numbered features shall  appear first in the string, and the character representing features 1 to 4 shall appear last  in the string. The list of features and their numbering (starting with 1) are defined  separately for each API. If the string contains a lower number of characters than there are  defined features for an API, all features that would be represented by characters that are not  present in the string are not supported.  | [optional] [default to None]
**selected_smf_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | [optional] [default to None]
**selected_old_smf_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | [optional] [default to None]
**inter_plmn_api_root** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


