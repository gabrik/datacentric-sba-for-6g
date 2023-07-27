# ChangeItem

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**op** | [***models::ChangeType**](ChangeType.md) |  | 
**path** | **String** | contains a JSON pointer value (as defined in IETF RFC 6901) that references a target  location within the resource on which the change has been applied.  | 
**from** | **String** | indicates the path of the source JSON element (according to JSON Pointer syntax)  being moved or copied to the location indicated by the \"path\" attribute. It shall  be present if the \"op\" attribute is of value \"MOVE\".  | [optional] [default to None]
**orig_value** | [***serde_json::Value**](.md) |  | [optional] [default to None]
**new_value** | [***serde_json::Value**](.md) |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


