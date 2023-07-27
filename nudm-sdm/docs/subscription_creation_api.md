# subscription_creation_api

All URIs are relative to *https://example.com/nudm-sdm/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
**Subscribe**](subscription_creation_api.md#Subscribe) | **POST** /{ueId}/sdm-subscriptions | subscribe to notifications


# **Subscribe**
> models::SdmSubscription Subscribe(ctx, ue_id, sdm_subscription)
subscribe to notifications

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **ue_id** | **String**| Identity of the user | 
  **sdm_subscription** | [**SdmSubscription**](SdmSubscription.md)|  | 

### Return type

[**models::SdmSubscription**](SdmSubscription.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

