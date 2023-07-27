# lcs_privacy_data_retrieval_api

All URIs are relative to *https://example.com/nudm-sdm/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
**GetLcsPrivacyData**](lcs_privacy_data_retrieval_api.md#GetLcsPrivacyData) | **GET** /{ueId}/lcs-privacy-data | retrieve a UE's LCS Privacy Subscription Data


# **GetLcsPrivacyData**
> models::LcsPrivacyData GetLcsPrivacyData(ctx, ue_id, optional)
retrieve a UE's LCS Privacy Subscription Data

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **ue_id** | **String**| Identifier of the UE | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ue_id** | **String**| Identifier of the UE | 
 **supported_features** | **String**| Supported Features | 
 **if_none_match** | **String**| Validator for conditional requests, as described in RFC 7232, 3.2 | 
 **if_modified_since** | **String**| Validator for conditional requests, as described in RFC 7232, 3.3 | 

### Return type

[**models::LcsPrivacyData**](LcsPrivacyData.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

