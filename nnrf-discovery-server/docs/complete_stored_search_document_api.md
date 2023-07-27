# complete_stored_search_document_api

All URIs are relative to *https://example.com/nnrf-disc/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
**RetrieveCompleteSearch**](complete_stored_search_document_api.md#RetrieveCompleteSearch) | **GET** /searches/{searchId}/complete | 


# **RetrieveCompleteSearch**
> models::StoredSearchResult RetrieveCompleteSearch(ctx, search_id, optional)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **search_id** | **String**| Id of a stored search | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **search_id** | **String**| Id of a stored search | 
 **accept_encoding** | **String**| Accept-Encoding, described in IETF RFC 7231 | 

### Return type

[**models::StoredSearchResult**](StoredSearchResult.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

