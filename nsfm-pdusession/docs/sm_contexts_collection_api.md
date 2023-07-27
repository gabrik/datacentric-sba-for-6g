# sm_contexts_collection_api

All URIs are relative to *https://example.com/nsmf-pdusession/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
**PostSmContexts**](sm_contexts_collection_api.md#PostSmContexts) | **POST** /sm-contexts | Create SM Context


# **PostSmContexts**
> models::SmContextCreatedData PostSmContexts(ctx, optional)
Create SM Context

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **json_data** | [**SmContextCreateData**](SmContextCreateData.md)|  | 
 **binary_data_n1_sm_message** | **swagger::ByteArray**|  | 
 **binary_data_n2_sm_information** | **swagger::ByteArray**|  | 
 **binary_data_n2_sm_information_ext1** | **swagger::ByteArray**|  | 

### Return type

[**models::SmContextCreatedData**](SmContextCreatedData.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: multipart/related
 - **Accept**: application/json, application/problem+json, multipart/related

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

