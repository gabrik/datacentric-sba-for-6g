# gpsito_supi_translation_or_supito_gpsi_translation_api

All URIs are relative to *https://example.com/nudm-sdm/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
**GetSupiOrGpsi**](gpsito_supi_translation_or_supito_gpsi_translation_api.md#GetSupiOrGpsi) | **GET** /{ueId}/id-translation-result | retrieve a UE's SUPI or GPSI


# **GetSupiOrGpsi**
> models::IdTranslationResult GetSupiOrGpsi(ctx, ue_id, optional)
retrieve a UE's SUPI or GPSI

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
 **af_id** | **String**| AF identifier | 
 **app_port_id** | [****](.md)| Application port identifier | 
 **af_service_id** | **String**| AF Service Identifier | 
 **mtc_provider_info** | **String**| MTC Provider Information | 
 **requested_gpsi_type** | [****](.md)| Requested GPSI Type | 
 **if_none_match** | **String**| Validator for conditional requests, as described in RFC 7232, 3.2 | 
 **if_modified_since** | **String**| Validator for conditional requests, as described in RFC 7232, 3.3 | 

### Return type

[**models::IdTranslationResult**](IdTranslationResult.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

