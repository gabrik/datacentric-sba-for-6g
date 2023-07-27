# retrieval_of_shared_data_api

All URIs are relative to *https://example.com/nudm-sdm/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
**GetSharedData**](retrieval_of_shared_data_api.md#GetSharedData) | **GET** /shared-data | retrieve shared data


# **GetSharedData**
> Vec<models::SharedData> GetSharedData(ctx, shared_data_ids, optional)
retrieve shared data

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **shared_data_ids** | [**String**](String.md)| List of shared data ids | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **shared_data_ids** | [**String**](String.md)| List of shared data ids | 
 **supported_features** | **String**| Supported Features; this query parameter should not be used | 
 **supported_features2** | **String**| Supported Features | 
 **if_none_match** | **String**| Validator for conditional requests, as described in RFC 7232, 3.2 | 
 **if_modified_since** | **String**| Validator for conditional requests, as described in RFC 7232, 3.3 | 

### Return type

[**Vec<models::SharedData>**](SharedData.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

