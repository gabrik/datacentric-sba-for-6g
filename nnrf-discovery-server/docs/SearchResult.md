# SearchResult

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**validity_period** | **i32** |  | [optional] [default to None]
**nf_instances** | [**Vec<models::NfProfile>**](NFProfile.md) |  | 
**search_id** | **String** |  | [optional] [default to None]
**num_nf_inst_complete** | **u32** | Integer where the allowed values correspond to the value range of an unsigned 32-bit integer.  | [optional] [default to None]
**preferred_search** | [***models::PreferredSearch**](PreferredSearch.md) |  | [optional] [default to None]
**nrf_supported_features** | **String** | A string used to indicate the features supported by an API that is used as defined in clause  6.6 in 3GPP TS 29.500. The string shall contain a bitmask indicating supported features in  hexadecimal representation Each character in the string shall take a value of \"0\" to \"9\",  \"a\" to \"f\" or \"A\" to \"F\" and shall represent the support of 4 features as described in  table 5.2.2-3. The most significant character representing the highest-numbered features shall  appear first in the string, and the character representing features 1 to 4 shall appear last  in the string. The list of features and their numbering (starting with 1) are defined  separately for each API. If the string contains a lower number of characters than there are  defined features for an API, all features that would be represented by characters that are not  present in the string are not supported.  | [optional] [default to None]
**nf_instance_list** | [**std::collections::HashMap<String, models::NfInstanceInfo>**](NfInstanceInfo.md) | List of matching NF instances. The key of the map is the NF instance ID. | [optional] [default to None]
**altered_priority_ind** | **bool** |  | [optional] [default to None]
**no_profile_match_info** | [***models::NoProfileMatchInfo**](NoProfileMatchInfo.md) |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


