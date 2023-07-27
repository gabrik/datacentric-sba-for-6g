# v2_x_subscription_data_retrieval_api

All URIs are relative to *https://example.com/nudm-sdm/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
**GetV2xData**](v2_x_subscription_data_retrieval_api.md#GetV2xData) | **GET** /{supi}/v2x-data | retrieve a UE's V2X Subscription Data


# **GetV2xData**
> models::V2xSubscriptionData GetV2xData(ctx, supi, optional)
retrieve a UE's V2X Subscription Data

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **supi** | **String**| Identifier of the UE | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **supi** | **String**| Identifier of the UE | 
 **supported_features** | **String**| Supported Features | 
 **if_none_match** | **String**| Validator for conditional requests, as described in RFC 7232, 3.2 | 
 **if_modified_since** | **String**| Validator for conditional requests, as described in RFC 7232, 3.3 | 

### Return type

[**models::V2xSubscriptionData**](V2xSubscriptionData.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

