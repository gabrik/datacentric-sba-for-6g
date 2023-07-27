# VsmfUpdateError

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**error** | [***models::ExtProblemDetails**](ExtProblemDetails.md) |  | 
**pti** | **u8** | Procedure Transaction Identifier | [optional] [default to None]
**n1sm_cause** | **String** |  | [optional] [default to None]
**n1_sm_info_from_ue** | [***models::RefToBinaryData**](RefToBinaryData.md) |  | [optional] [default to None]
**unknown_n1_sm_info** | [***models::RefToBinaryData**](RefToBinaryData.md) |  | [optional] [default to None]
**failed_to_assign_ebi_list** | [**Vec<models::Arp>**](Arp.md) |  | [optional] [default to None]
**ng_ap_cause** | [***models::NgApCause**](NgApCause.md) |  | [optional] [default to None]
**param_5g_mm_cause_value** | **u32** | Unsigned Integer, i.e. only value 0 and integers above 0 are permissible. | [optional] [default to None]
**recovery_time** | [**chrono::DateTime::<chrono::Utc>**](DateTime.md) | string with format 'date-time' as defined in OpenAPI. | [optional] [default to None]
**n4_info** | [***models::N4Information**](N4Information.md) |  | [optional] [default to None]
**n4_info_ext1** | [***models::N4Information**](N4Information.md) |  | [optional] [default to None]
**n4_info_ext2** | [***models::N4Information**](N4Information.md) |  | [optional] [default to None]
**n4_info_ext3** | [***models::N4Information**](N4Information.md) |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


