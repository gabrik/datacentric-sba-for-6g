# ue_context_in_amf_data_retrieval_api

All URIs are relative to *https://example.com/nudm-sdm/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
**GetUeCtxInAmfData**](ue_context_in_amf_data_retrieval_api.md#GetUeCtxInAmfData) | **GET** /{supi}/ue-context-in-amf-data | retrieve a UE's UE Context In AMF Data


# **GetUeCtxInAmfData**
> models::UeContextInAmfData GetUeCtxInAmfData(ctx, supi, optional)
retrieve a UE's UE Context In AMF Data

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
 **supported_features** | **String**| Supported Features | 

### Return type

[**models::UeContextInAmfData**](UeContextInAmfData.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

