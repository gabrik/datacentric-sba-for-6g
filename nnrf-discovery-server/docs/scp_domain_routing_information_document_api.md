# scp_domain_routing_information_document_api

All URIs are relative to *https://example.com/nnrf-disc/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
**SCPDomainRoutingInfoGet**](scp_domain_routing_information_document_api.md#SCPDomainRoutingInfoGet) | **GET** /scp-domain-routing-info | 


# **SCPDomainRoutingInfoGet**
> models::ScpDomainRoutingInformation SCPDomainRoutingInfoGet(ctx, optional)


### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **local** | **bool**| Indication of local SCP Domain Routing Information | [default to false]
 **accept_encoding** | **String**| Accept-Encoding, described in IETF RFC 7231 | 

### Return type

[**models::ScpDomainRoutingInformation**](ScpDomainRoutingInformation.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

