# individual_sm_context_api

All URIs are relative to *https://example.com/nsmf-pdusession/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
**ReleaseSmContext**](individual_sm_context_api.md#ReleaseSmContext) | **POST** /sm-contexts/{smContextRef}/release | Release SM Context
**RetrieveSmContext**](individual_sm_context_api.md#RetrieveSmContext) | **POST** /sm-contexts/{smContextRef}/retrieve | Retrieve SM Context
**SendMoData**](individual_sm_context_api.md#SendMoData) | **POST** /sm-contexts/{smContextRef}/send-mo-data | Send MO Data
**UpdateSmContext**](individual_sm_context_api.md#UpdateSmContext) | **POST** /sm-contexts/{smContextRef}/modify | Update SM Context


# **ReleaseSmContext**
> models::SmContextReleasedData ReleaseSmContext(ctx, sm_context_ref, optional)
Release SM Context

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **sm_context_ref** | **String**| SM context reference | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **sm_context_ref** | **String**| SM context reference | 
 **sm_context_release_data** | [**SmContextReleaseData**](SmContextReleaseData.md)| representation of the data to be sent to the SMF when releasing the SM context | 

### Return type

[**models::SmContextReleasedData**](SmContextReleasedData.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: application/json, multipart/related
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **RetrieveSmContext**
> models::SmContextRetrievedData RetrieveSmContext(ctx, sm_context_ref, optional)
Retrieve SM Context

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **sm_context_ref** | **String**| SM context reference | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **sm_context_ref** | **String**| SM context reference | 
 **sm_context_retrieve_data** | [**SmContextRetrieveData**](SmContextRetrieveData.md)| parameters used to retrieve the SM context | 

### Return type

[**models::SmContextRetrievedData**](SmContextRetrievedData.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **SendMoData**
> SendMoData(ctx, sm_context_ref, optional)
Send MO Data

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **sm_context_ref** | **String**| SM context reference | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **sm_context_ref** | **String**| SM context reference | 
 **json_data** | [**SendMoDataReqData**](SendMoDataReqData.md)|  | 
 **binary_mo_data** | **swagger::ByteArray**|  | 

### Return type

 (empty response body)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: multipart/related
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **UpdateSmContext**
> models::SmContextUpdatedData UpdateSmContext(ctx, sm_context_ref, sm_context_update_data)
Update SM Context

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **sm_context_ref** | **String**| SM context reference | 
  **sm_context_update_data** | [**SmContextUpdateData**](SmContextUpdateData.md)| representation of the updates to apply to the SM context | 

### Return type

[**models::SmContextUpdatedData**](SmContextUpdatedData.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: application/json, multipart/related
 - **Accept**: application/json, application/problem+json, multipart/related

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

