# providing_acknowledgement_of_snssais_update_api

All URIs are relative to *https://example.com/nudm-sdm/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
**S-NSSAIs Ack**](providing_acknowledgement_of_snssais_update_api.md#S-NSSAIs Ack) | **PUT** /{supi}/am-data/subscribed-snssais-ack | Nudm_Sdm Info operation for S-NSSAIs acknowledgement


# **S-NSSAIs Ack**
> S-NSSAIs Ack(ctx, supi, optional)
Nudm_Sdm Info operation for S-NSSAIs acknowledgement

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
 **acknowledge_info** | [**AcknowledgeInfo**](AcknowledgeInfo.md)|  | 

### Return type

 (empty response body)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

