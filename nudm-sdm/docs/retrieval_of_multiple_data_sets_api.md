# retrieval_of_multiple_data_sets_api

All URIs are relative to *https://example.com/nudm-sdm/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
**GetDataSets**](retrieval_of_multiple_data_sets_api.md#GetDataSets) | **GET** /{supi} | retrieve multiple data sets


# **GetDataSets**
> models::SubscriptionDataSets GetDataSets(ctx, supi, dataset_names, optional)
retrieve multiple data sets

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **supi** | **String**| Identifier of the UE | 
  **dataset_names** | [**models::DataSetName**](models::DataSetName.md)| List of dataset names | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **supi** | **String**| Identifier of the UE | 
 **dataset_names** | [**models::DataSetName**](models::DataSetName.md)| List of dataset names | 
 **plmn_id** | [****](.md)| serving PLMN ID | 
 **disaster_roaming_ind** | **bool**| Indication whether Disaster Roaming service is applied or not | [default to false]
 **supported_features** | **String**| Supported Features | 
 **if_none_match** | **String**| Validator for conditional requests, as described in RFC 7232, 3.2 | 
 **if_modified_since** | **String**| Validator for conditional requests, as described in RFC 7232, 3.3 | 

### Return type

[**models::SubscriptionDataSets**](SubscriptionDataSets.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

