# UdsfInfo

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**group_id** | **String** | Identifier of a group of NFs. | [optional] [default to None]
**supi_ranges** | [**Vec<models::SupiRange>**](SupiRange.md) |  | [optional] [default to None]
**storage_id_ranges** | [**std::collections::HashMap<String, Vec<models::IdentityRange>>**](array.md) | A map (list of key-value pairs) where realmId serves as key and each value in the map is an array of IdentityRanges. Each IdentityRange is a range of storageIds.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


