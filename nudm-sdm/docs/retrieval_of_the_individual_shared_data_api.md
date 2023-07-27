# retrieval_of_the_individual_shared_data_api

All URIs are relative to *https://example.com/nudm-sdm/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
**GetIndividualSharedData**](retrieval_of_the_individual_shared_data_api.md#GetIndividualSharedData) | **GET** /shared-data/{sharedDataId} | retrieve the individual shared data


# **GetIndividualSharedData**
> models::SharedData GetIndividualSharedData(ctx, shared_data_id, optional)
retrieve the individual shared data

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **shared_data_id** | [**String**](String.md)| Id of the Shared data | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **shared_data_id** | [**String**](String.md)| Id of the Shared data | 
 **supported_features** | **String**| Supported Features | 
 **if_none_match** | **String**| Validator for conditional requests, as described in RFC 7232, 3.2 | 
 **if_modified_since** | **String**| Validator for conditional requests, as described in RFC 7232, 3.3 | 

### Return type

[**models::SharedData**](SharedData.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

