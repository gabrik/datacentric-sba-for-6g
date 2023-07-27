# ProblemDetails

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**r#type** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]
**title** | **String** |  | [optional] [default to None]
**status** | **i32** |  | [optional] [default to None]
**detail** | **String** | A human-readable explanation specific to this occurrence of the problem. | [optional] [default to None]
**instance** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]
**cause** | **String** | A machine-readable application error cause specific to this occurrence of the problem.  This IE should be present and provide application-related error information, if available.  | [optional] [default to None]
**invalid_params** | [**Vec<models::InvalidParam>**](InvalidParam.md) |  | [optional] [default to None]
**supported_features** | **String** | A string used to indicate the features supported by an API that is used as defined in clause  6.6 in 3GPP TS 29.500. The string shall contain a bitmask indicating supported features in  hexadecimal representation Each character in the string shall take a value of \"0\" to \"9\",  \"a\" to \"f\" or \"A\" to \"F\" and shall represent the support of 4 features as described in  table 5.2.2-3. The most significant character representing the highest-numbered features shall  appear first in the string, and the character representing features 1 to 4 shall appear last  in the string. The list of features and their numbering (starting with 1) are defined  separately for each API. If the string contains a lower number of characters than there are  defined features for an API, all features that would be represented by characters that are not  present in the string are not supported.  | [optional] [default to None]
**access_token_error** | [***models::AccessTokenErr**](AccessTokenErr.md) |  | [optional] [default to None]
**access_token_request** | [***models::AccessTokenReq**](AccessTokenReq.md) |  | [optional] [default to None]
**nrf_id** | **String** | Fully Qualified Domain Name | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


