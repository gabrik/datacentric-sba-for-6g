# NefInfo

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**nef_id** | **String** | Identity of the NEF | [optional] [default to None]
**pfd_data** | [***models::PfdData**](PfdData.md) |  | [optional] [default to None]
**af_ee_data** | [***models::AfEventExposureData**](AfEventExposureData.md) |  | [optional] [default to None]
**gpsi_ranges** | [**Vec<models::IdentityRange>**](IdentityRange.md) |  | [optional] [default to None]
**external_group_identifiers_ranges** | [**Vec<models::IdentityRange>**](IdentityRange.md) |  | [optional] [default to None]
**served_fqdn_list** | **Vec<String>** |  | [optional] [default to None]
**tai_list** | [**Vec<models::Tai>**](Tai.md) |  | [optional] [default to None]
**tai_range_list** | [**Vec<models::TaiRange>**](TaiRange.md) |  | [optional] [default to None]
**dnai_list** | **Vec<models::Dnai>** |  | [optional] [default to None]
**un_trust_af_info_list** | [**Vec<models::UnTrustAfInfo>**](UnTrustAfInfo.md) |  | [optional] [default to None]
**uas_nf_functionality_ind** | **bool** |  | [optional] [default to Some(false)]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


