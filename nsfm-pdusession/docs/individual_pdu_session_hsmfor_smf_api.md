# individual_pdu_session_hsmfor_smf_api

All URIs are relative to *https://example.com/nsmf-pdusession/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
**ReleasePduSession**](individual_pdu_session_hsmfor_smf_api.md#ReleasePduSession) | **POST** /pdu-sessions/{pduSessionRef}/release | Release
**RetrievePduSession**](individual_pdu_session_hsmfor_smf_api.md#RetrievePduSession) | **POST** /pdu-sessions/{pduSessionRef}/retrieve | Retrieve
**TransferMoData**](individual_pdu_session_hsmfor_smf_api.md#TransferMoData) | **POST** /pdu-sessions/{pduSessionRef}/transfer-mo-data | Transfer MO Data
**UpdatePduSession**](individual_pdu_session_hsmfor_smf_api.md#UpdatePduSession) | **POST** /pdu-sessions/{pduSessionRef}/modify | Update (initiated by V-SMF or I-SMF)


# **ReleasePduSession**
> models::ReleasedData ReleasePduSession(ctx, pdu_session_ref, optional)
Release

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **pdu_session_ref** | **String**| PDU session reference | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **pdu_session_ref** | **String**| PDU session reference | 
 **release_data** | [**ReleaseData**](ReleaseData.md)| data sent to H-SMF or SMF when releasing the PDU session | 

### Return type

[**models::ReleasedData**](ReleasedData.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: application/json, multipart/related
 - **Accept**: application/json, application/problem+json, multipart/related

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **RetrievePduSession**
> models::RetrievedData RetrievePduSession(ctx, pdu_session_ref, retrieve_data)
Retrieve

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **pdu_session_ref** | **String**| PDU session reference | 
  **retrieve_data** | [**RetrieveData**](RetrieveData.md)| representation of the payload of the Retrieve Request | 

### Return type

[**models::RetrievedData**](RetrievedData.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **TransferMoData**
> TransferMoData(ctx, pdu_session_ref, optional)
Transfer MO Data

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **pdu_session_ref** | **String**| PDU session reference | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **pdu_session_ref** | **String**| PDU session reference | 
 **json_data** | [**TransferMoDataReqData**](TransferMoDataReqData.md)|  | 
 **binary_mo_data** | **swagger::ByteArray**|  | 

### Return type

 (empty response body)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: multipart/related
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

# **UpdatePduSession**
> models::HsmfUpdatedData UpdatePduSession(ctx, pdu_session_ref, hsmf_update_data)
Update (initiated by V-SMF or I-SMF)

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **pdu_session_ref** | **String**| PDU session reference | 
  **hsmf_update_data** | [**HsmfUpdateData**](HsmfUpdateData.md)| representation of the updates to apply to the PDU session | 

### Return type

[**models::HsmfUpdatedData**](HsmfUpdatedData.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: application/json, multipart/related
 - **Accept**: application/json, application/problem+json, multipart/related

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

