# pdu_sessions_collection_api

All URIs are relative to *https://example.com/nsmf-pdusession/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
**PostPduSessions**](pdu_sessions_collection_api.md#PostPduSessions) | **POST** /pdu-sessions | Create


# **PostPduSessions**
> models::PduSessionCreatedData PostPduSessions(ctx, pdu_session_create_data)
Create

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **pdu_session_create_data** | [**PduSessionCreateData**](PduSessionCreateData.md)| representation of the PDU session to be created in the H-SMF or SMF | 

### Return type

[**models::PduSessionCreatedData**](PduSessionCreatedData.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: application/json, multipart/related
 - **Accept**: application/json, application/problem+json, multipart/related

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

