# individual_scp_domain_routing_information_subscription_document_api

All URIs are relative to *https://example.com/nnrf-disc/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
**ScpDomainRoutingInfoUnsubscribe**](individual_scp_domain_routing_information_subscription_document_api.md#ScpDomainRoutingInfoUnsubscribe) | **DELETE** /scp-domain-routing-info-subs/{subscriptionID} | Deletes a subscription


# **ScpDomainRoutingInfoUnsubscribe**
> ScpDomainRoutingInfoUnsubscribe(ctx, subscription_id)
Deletes a subscription

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **subscription_id** | **String**| Unique ID of the subscription to remove | 

### Return type

 (empty response body)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

