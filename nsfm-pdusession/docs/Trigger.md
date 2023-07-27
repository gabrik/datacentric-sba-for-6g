# Trigger

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**trigger_type** | [***models::TriggerType**](TriggerType.md) |  | 
**trigger_category** | [***models::TriggerCategory**](TriggerCategory.md) |  | 
**time_limit** | **i32** | indicating a time in seconds. | [optional] [default to None]
**volume_limit** | **u32** | Integer where the allowed values correspond to the value range of an unsigned 32-bit integer.  | [optional] [default to None]
**volume_limit64** | **u8** | Integer where the allowed values correspond to the value range of an unsigned 64-bit integer.  | [optional] [default to None]
**event_limit** | **u32** | Integer where the allowed values correspond to the value range of an unsigned 32-bit integer.  | [optional] [default to None]
**max_number_ofccc** | **u32** | Integer where the allowed values correspond to the value range of an unsigned 32-bit integer.  | [optional] [default to None]
**tariff_time_change** | [**chrono::DateTime::<chrono::Utc>**](DateTime.md) | string with format 'date-time' as defined in OpenAPI. | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


