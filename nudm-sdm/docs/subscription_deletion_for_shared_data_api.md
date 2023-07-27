# subscription_deletion_for_shared_data_api

All URIs are relative to *https://example.com/nudm-sdm/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
**UnsubscribeForSharedData**](subscription_deletion_for_shared_data_api.md#UnsubscribeForSharedData) | **DELETE** /shared-data-subscriptions/{subscriptionId} | unsubscribe from notifications for shared data


# **UnsubscribeForSharedData**
> UnsubscribeForSharedData(ctx, subscription_id)
unsubscribe from notifications for shared data

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **subscription_id** | **String**| Id of the Shared data Subscription | 

### Return type

 (empty response body)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

