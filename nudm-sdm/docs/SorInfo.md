# SorInfo

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**steering_container** | [***models::SteeringContainer**](SteeringContainer.md) |  | [optional] [default to None]
**ack_ind** | **bool** | Contains indication whether the acknowledgement from UE is needed. | 
**sor_mac_iausf** | **String** | MAC value for protecting SOR procedure (SoR-MAC-IAUSF and SoR-XMAC-IUE). | [optional] [default to None]
**countersor** | **String** | CounterSoR. | [optional] [default to None]
**provisioning_time** | [**chrono::DateTime::<chrono::Utc>**](DateTime.md) | string with format 'date-time' as defined in OpenAPI. | 
**sor_transparent_container** | [***swagger::ByteArray**](ByteArray.md) | string with format 'bytes' as defined in OpenAPI | [optional] [default to None]
**sor_cmci** | [***swagger::ByteArray**](ByteArray.md) | string with format 'bytes' as defined in OpenAPI | [optional] [default to None]
**store_sor_cmci_in_me** | **bool** |  | [optional] [default to None]
**usim_support_of_sor_cmci** | **bool** |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


