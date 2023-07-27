# group_identifiers_api

All URIs are relative to *https://example.com/nudm-sdm/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
**GetGroupIdentifiers**](group_identifiers_api.md#GetGroupIdentifiers) | **GET** /group-data/group-identifiers | Mapping of Group Identifiers


# **GetGroupIdentifiers**
> models::GroupIdentifiers GetGroupIdentifiers(ctx, optional)
Mapping of Group Identifiers

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ext_group_id** | **String**| External Group Identifier | 
 **int_group_id** | **String**| Internal Group Identifier | 
 **ue_id_ind** | **bool**| Indication whether UE identifiers are required or not | [default to false]
 **supported_features** | **String**| Supported Features | 
 **af_id** | **String**| AF identifier | 
 **if_none_match** | **String**| Validator for conditional requests, as described in RFC 7232, 3.2 | 
 **if_modified_since** | **String**| Validator for conditional requests, as described in RFC 7232, 3.3 | 

### Return type

[**models::GroupIdentifiers**](GroupIdentifiers.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

