# trigger_sor_info_update_api

All URIs are relative to *https://example.com/nudm-sdm/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
**Update SOR Info**](trigger_sor_info_update_api.md#Update SOR Info) | **POST** /{supi}/am-data/update-sor | Nudm_Sdm custom operation to trigger SOR info update


# **Update SOR Info**
> models::SorInfo Update SOR Info(ctx, supi, optional)
Nudm_Sdm custom operation to trigger SOR info update

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
 **sor_update_info** | [**SorUpdateInfo**](SorUpdateInfo.md)|  | 

### Return type

[**models::SorInfo**](SorInfo.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

