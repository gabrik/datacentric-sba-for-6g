# scp_domain_routing_information_subscriptions_collection_api

All URIs are relative to *https://example.com/nnrf-disc/v1*

Method | HTTP request | Description
------------- | ------------- | -------------
**ScpDomainRoutingInfoSubscribe**](scp_domain_routing_information_subscriptions_collection_api.md#ScpDomainRoutingInfoSubscribe) | **POST** /scp-domain-routing-info-subs | Create a new subscription


# **ScpDomainRoutingInfoSubscribe**
> models::ScpDomainRoutingInfoSubscription ScpDomainRoutingInfoSubscribe(ctx, scp_domain_routing_info_subscription, optional)
Create a new subscription

### Required Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **ctx** | **context.Context** | context containing the authentication | nil if no authentication
  **scp_domain_routing_info_subscription** | [**ScpDomainRoutingInfoSubscription**](ScpDomainRoutingInfoSubscription.md)|  | 
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters
Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **scp_domain_routing_info_subscription** | [**ScpDomainRoutingInfoSubscription**](ScpDomainRoutingInfoSubscription.md)|  | 
 **content_encoding** | **String**| Content-Encoding, described in IETF RFC 7231 | 
 **accept_encoding** | **String**| Accept-Encoding, described in IETF RFC 7231 | 

### Return type

[**models::ScpDomainRoutingInfoSubscription**](ScpDomainRoutingInfoSubscription.md)

### Authorization

[oAuth2ClientCredentials](../README.md#oAuth2ClientCredentials)

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json, application/problem+json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

