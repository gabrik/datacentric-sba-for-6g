# subscription_modification_api

All URIs are relative to *https://example.com/nudm-sdm/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
**Modify**](subscription_modification_api.md#Modify) | **PATCH** /{ueId}/sdm-subscriptions/{subscriptionId} | modify the subscription
**ModifySharedDataSubs**](subscription_modification_api.md#ModifySharedDataSubs) | **PATCH** /shared-data-subscriptions/{subscriptionId} | modify the subscription


# **Modify**
> models::Modify200Response Modify(ctx, ue_id, subscription_id, sdm_subs_modification, optional)
modify the subscription

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **ue_id** | **String**| Identity of the user | 
  **subscription_id** | **String**| Id of the SDM Subscription | 
  **sdm_subs_modification** | [**SdmSubsModification**](SdmSubsModification.md)|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ue_id** | **String**| Identity of the user | 
 **subscription_id** | **String**| Id of the SDM Subscription | 
 **sdm_subs_modification** | [**SdmSubsModification**](SdmSubsModification.md)|  | 
 **supported_features** | **String**| Features required to be supported by the target NF | 

### Return type

[**models::Modify200Response**](Modify_200_response.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: application/merge-patch+json
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **ModifySharedDataSubs**
> models::Modify200Response ModifySharedDataSubs(ctx, subscription_id, sdm_subs_modification, optional)
modify the subscription

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **subscription_id** | **String**| Id of the SDM Subscription | 
  **sdm_subs_modification** | [**SdmSubsModification**](SdmSubsModification.md)|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **subscription_id** | **String**| Id of the SDM Subscription | 
 **sdm_subs_modification** | [**SdmSubsModification**](SdmSubsModification.md)|  | 
 **supported_features** | **String**| Features required to be supported by the target NF | 

### Return type

[**models::Modify200Response**](Modify_200_response.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: application/merge-patch+json
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

