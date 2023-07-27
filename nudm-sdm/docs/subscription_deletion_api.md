# subscription_deletion_api

All URIs are relative to *https://example.com/nudm-sdm/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
**Unsubscribe**](subscription_deletion_api.md#Unsubscribe) | **DELETE** /{ueId}/sdm-subscriptions/{subscriptionId} | unsubscribe from notifications


# **Unsubscribe**
> Unsubscribe(ctx, ue_id, subscription_id)
unsubscribe from notifications

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **ue_id** | **String**| Identity of the user | 
  **subscription_id** | **String**| Id of the SDM Subscription | 

### Return type

 (empty response body)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

