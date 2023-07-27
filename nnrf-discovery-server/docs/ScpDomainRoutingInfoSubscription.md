# ScpDomainRoutingInfoSubscription

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**callback_uri** | **String** | String providing an URI formatted according to RFC 3986. | 
**validity_time** | [**chrono::DateTime::<chrono::Utc>**](DateTime.md) | string with format 'date-time' as defined in OpenAPI. | [optional] [default to None]
**req_instance_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | [optional] [default to None]
**local_ind** | **bool** |  | [optional] [default to Some(false)]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


