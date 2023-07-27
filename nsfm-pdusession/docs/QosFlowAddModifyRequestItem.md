# QosFlowAddModifyRequestItem

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**qfi** | **u8** | Unsigned integer identifying a QoS flow, within the range 0 to 63. | 
**ebi** | **u8** | EPS Bearer Identifier | [optional] [default to None]
**qos_rules** | [***swagger::ByteArray**](ByteArray.md) | string with format 'bytes' as defined in OpenAPI | [optional] [default to None]
**qos_flow_description** | [***swagger::ByteArray**](ByteArray.md) | string with format 'bytes' as defined in OpenAPI | [optional] [default to None]
**qos_flow_profile** | [***models::QosFlowProfile**](QosFlowProfile.md) |  | [optional] [default to None]
**associated_an_type** | [***models::QosFlowAccessType**](QosFlowAccessType.md) |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


