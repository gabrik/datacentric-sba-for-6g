# multiple_identifiers_api

All URIs are relative to *https://example.com/nudm-sdm/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
**GetMultipleIdentifiers**](multiple_identifiers_api.md#GetMultipleIdentifiers) | **GET** /multiple-identifiers | Mapping of UE Identifiers


# **GetMultipleIdentifiers**
> std::collections::HashMap<String, models::SupiInfo> GetMultipleIdentifiers(ctx, gpsi_list, optional)
Mapping of UE Identifiers

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **gpsi_list** | [**String**](String.md)| list of the GPSIs | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **gpsi_list** | [**String**](String.md)| list of the GPSIs | 
 **supported_features** | **String**| Supported Features | 

### Return type

[**std::collections::HashMap<String, models::SupiInfo>**](SupiInfo.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

