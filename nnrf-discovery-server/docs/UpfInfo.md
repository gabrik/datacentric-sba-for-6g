# UpfInfo

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**s_nssai_upf_info_list** | [**Vec<models::SnssaiUpfInfoItem>**](SnssaiUpfInfoItem.md) |  | 
**smf_serving_area** | **Vec<String>** |  | [optional] [default to None]
**interface_upf_info_list** | [**Vec<models::InterfaceUpfInfoItem>**](InterfaceUpfInfoItem.md) |  | [optional] [default to None]
**iwk_eps_ind** | **bool** |  | [optional] [default to Some(false)]
**pdu_session_types** | [**Vec<models::PduSessionType>**](PduSessionType.md) |  | [optional] [default to None]
**atsss_capability** | [***models::AtsssCapability**](AtsssCapability.md) |  | [optional] [default to None]
**ue_ip_addr_ind** | **bool** |  | [optional] [default to Some(false)]
**tai_list** | [**Vec<models::Tai>**](Tai.md) |  | [optional] [default to None]
**tai_range_list** | [**Vec<models::TaiRange>**](TaiRange.md) |  | [optional] [default to None]
**w_agf_info** | [***models::WAgfInfo**](WAgfInfo.md) |  | [optional] [default to None]
**tngf_info** | [***models::TngfInfo**](TngfInfo.md) |  | [optional] [default to None]
**twif_info** | [***models::TwifInfo**](TwifInfo.md) |  | [optional] [default to None]
**priority** | **u16** |  | [optional] [default to None]
**redundant_gtpu** | **bool** |  | [optional] [default to Some(false)]
**ipups** | **bool** |  | [optional] [default to Some(false)]
**data_forwarding** | **bool** |  | [optional] [default to Some(false)]
**supported_pfcp_features** | **String** |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


