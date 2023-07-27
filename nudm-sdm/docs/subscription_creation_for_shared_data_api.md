# subscription_creation_for_shared_data_api

All URIs are relative to *https://example.com/nudm-sdm/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
**SubscribeToSharedData**](subscription_creation_for_shared_data_api.md#SubscribeToSharedData) | **POST** /shared-data-subscriptions | subscribe to notifications for shared data


# **SubscribeToSharedData**
> models::SdmSubscription SubscribeToSharedData(ctx, sdm_subscription)
subscribe to notifications for shared data

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **sdm_subscription** | [**SdmSubscription**](SdmSubscription.md)|  | 

### Return type

[**models::SdmSubscription**](SdmSubscription.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

