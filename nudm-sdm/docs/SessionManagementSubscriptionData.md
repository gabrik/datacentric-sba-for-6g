# SessionManagementSubscriptionData

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**single_nssai** | [***models::Snssai**](Snssai.md) |  | 
**dnn_configurations** | [**std::collections::HashMap<String, models::DnnConfiguration>**](DnnConfiguration.md) | A map (list of key-value pairs where Dnn, or optionally the Wildcard DNN, serves as key) of DnnConfigurations | [optional] [default to None]
**internal_group_ids** | **Vec<models::GroupId>** |  | [optional] [default to None]
**shared_vn_group_data_ids** | **std::collections::HashMap<String, models::SharedDataId>** | A map(list of key-value pairs) where GroupId serves as key of SharedDataId | [optional] [default to None]
**shared_dnn_configurations_id** | **String** |  | [optional] [default to None]
**odb_packet_services** | [***models::OdbPacketServices**](OdbPacketServices.md) |  | [optional] [default to None]
**trace_data** | [***models::TraceData**](TraceData.md) |  | [optional] [default to None]
**shared_trace_data_id** | **String** |  | [optional] [default to None]
**expected_ue_behaviours_list** | [**std::collections::HashMap<String, models::ExpectedUeBehaviourData>**](ExpectedUeBehaviourData.md) | A map(list of key-value pairs) where Dnn serves as key of ExpectedUeBehaviourData | [optional] [default to None]
**suggested_packet_num_dl_list** | [**std::collections::HashMap<String, models::SuggestedPacketNumDl>**](SuggestedPacketNumDl.md) | A map(list of key-value pairs) where Dnn serves as key of SuggestedPacketNumDl | [optional] [default to None]
**param_3gpp_charging_characteristics** | **String** |  | [optional] [default to None]
**supported_features** | **String** | A string used to indicate the features supported by an API that is used as defined in clause  6.6 in 3GPP TS 29.500. The string shall contain a bitmask indicating supported features in  hexadecimal representation Each character in the string shall take a value of \"0\" to \"9\",  \"a\" to \"f\" or \"A\" to \"F\" and shall represent the support of 4 features as described in  table 5.2.2-3. The most significant character representing the highest-numbered features shall  appear first in the string, and the character representing features 1 to 4 shall appear last  in the string. The list of features and their numbering (starting with 1) are defined  separately for each API. If the string contains a lower number of characters than there are  defined features for an API, all features that would be represented by characters that are not  present in the string are not supported.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


