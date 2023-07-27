# SdmSubscription

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**nf_instance_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | 
**implicit_unsubscribe** | **bool** |  | [optional] [default to None]
**expires** | [**chrono::DateTime::<chrono::Utc>**](DateTime.md) | string with format 'date-time' as defined in OpenAPI. | [optional] [default to None]
**callback_reference** | **String** | String providing an URI formatted according to RFC 3986. | 
**amf_service_name** | [***models::ServiceName**](ServiceName.md) |  | [optional] [default to None]
**monitored_resource_uris** | **Vec<models::Uri>** |  | 
**single_nssai** | [***models::Snssai**](Snssai.md) |  | [optional] [default to None]
**dnn** | **String** | String representing a Data Network as defined in clause 9A of 3GPP TS 23.003;  it shall contain either a DNN Network Identifier, or a full DNN with both the Network  Identifier and Operator Identifier, as specified in 3GPP TS 23.003 clause 9.1.1 and 9.1.2. It shall be coded as string in which the labels are separated by dots  (e.g. \"Label1.Label2.Label3\").  | [optional] [default to None]
**subscription_id** | **String** |  | [optional] [default to None]
**plmn_id** | [***models::PlmnId**](PlmnId.md) |  | [optional] [default to None]
**immediate_report** | **bool** |  | [optional] [default to Some(false)]
**report** | [***models::ImmediateReport**](ImmediateReport.md) |  | [optional] [default to None]
**supported_features** | **String** | A string used to indicate the features supported by an API that is used as defined in clause  6.6 in 3GPP TS 29.500. The string shall contain a bitmask indicating supported features in  hexadecimal representation Each character in the string shall take a value of \"0\" to \"9\",  \"a\" to \"f\" or \"A\" to \"F\" and shall represent the support of 4 features as described in  table 5.2.2-3. The most significant character representing the highest-numbered features shall  appear first in the string, and the character representing features 1 to 4 shall appear last  in the string. The list of features and their numbering (starting with 1) are defined  separately for each API. If the string contains a lower number of characters than there are  defined features for an API, all features that would be represented by characters that are not  present in the string are not supported.  | [optional] [default to None]
**context_info** | [***models::ContextInfo**](ContextInfo.md) |  | [optional] [default to None]
**nf_change_filter** | **bool** |  | [optional] [default to Some(false)]
**unique_subscription** | **bool** |  | [optional] [default to None]
**reset_ids** | **Vec<String>** |  | [optional] [default to None]
**ue_con_smf_data_sub_filter** | [***models::UeContextInSmfDataSubFilter**](UeContextInSmfDataSubFilter.md) |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


