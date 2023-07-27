# AmfInfo

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**amf_set_id** | **String** | String identifying the AMF Set ID (10 bits) as specified in clause 2.10.1 of 3GPP TS 23.003.  It is encoded as a string of 3 hexadecimal characters where the first character is limited to  values 0 to 3 (i.e. 10 bits).  | 
**amf_region_id** | **String** | String identifying the AMF Set ID (10 bits) as specified in clause 2.10.1 of 3GPP TS 23.003.  It is encoded as a string of 3 hexadecimal characters where the first character is limited to  values 0 to 3 (i.e. 10 bits)  | 
**guami_list** | [**Vec<models::Guami>**](Guami.md) |  | 
**tai_list** | [**Vec<models::Tai>**](Tai.md) |  | [optional] [default to None]
**tai_range_list** | [**Vec<models::TaiRange>**](TaiRange.md) |  | [optional] [default to None]
**backup_info_amf_failure** | [**Vec<models::Guami>**](Guami.md) |  | [optional] [default to None]
**backup_info_amf_removal** | [**Vec<models::Guami>**](Guami.md) |  | [optional] [default to None]
**n2_interface_amf_info** | [***models::N2InterfaceAmfInfo**](N2InterfaceAmfInfo.md) |  | [optional] [default to None]
**amf_onboarding_capability** | **bool** |  | [optional] [default to Some(false)]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


