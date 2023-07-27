# GbrQosFlowInformation

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**max_fbr_dl** | **String** | String representing a bit rate; the prefixes follow the standard symbols from The International System of Units, and represent x1000 multipliers, with the exception that prefix \"K\" is used to represent the standard symbol \"k\".  | 
**max_fbr_ul** | **String** | String representing a bit rate; the prefixes follow the standard symbols from The International System of Units, and represent x1000 multipliers, with the exception that prefix \"K\" is used to represent the standard symbol \"k\".  | 
**gua_fbr_dl** | **String** | String representing a bit rate; the prefixes follow the standard symbols from The International System of Units, and represent x1000 multipliers, with the exception that prefix \"K\" is used to represent the standard symbol \"k\".  | 
**gua_fbr_ul** | **String** | String representing a bit rate; the prefixes follow the standard symbols from The International System of Units, and represent x1000 multipliers, with the exception that prefix \"K\" is used to represent the standard symbol \"k\".  | 
**notif_control** | [***models::NotificationControl**](NotificationControl.md) |  | [optional] [default to None]
**max_packet_loss_rate_dl** | **u16** | Unsigned integer indicating Packet Loss Rate (see clauses 5.7.2.8 and 5.7.4 of 3GPP TS 23.501), expressed in tenth of percent.  | [optional] [default to None]
**max_packet_loss_rate_ul** | **u16** | Unsigned integer indicating Packet Loss Rate (see clauses 5.7.2.8 and 5.7.4 of 3GPP TS 23.501), expressed in tenth of percent.  | [optional] [default to None]
**alternative_qos_profile_list** | [**Vec<models::AlternativeQosProfile>**](AlternativeQosProfile.md) |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


