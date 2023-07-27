# AlternativeQosProfile

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**index** | **u8** |  | 
**gua_fbr_dl** | **String** | String representing a bit rate; the prefixes follow the standard symbols from The International System of Units, and represent x1000 multipliers, with the exception that prefix \"K\" is used to represent the standard symbol \"k\".  | [optional] [default to None]
**gua_fbr_ul** | **String** | String representing a bit rate; the prefixes follow the standard symbols from The International System of Units, and represent x1000 multipliers, with the exception that prefix \"K\" is used to represent the standard symbol \"k\".  | [optional] [default to None]
**packet_delay_budget** | **u32** | Unsigned integer indicating Packet Delay Budget (see clauses 5.7.3.4 and 5.7.4 of 3GPP TS 23.501), expressed in milliseconds.  | [optional] [default to None]
**packet_err_rate** | **String** | String representing Packet Error Rate (see clause 5.7.3.5 and 5.7.4 of 3GPP TS 23.501, expressed as a \"scalar x 10-k\" where the scalar and the exponent k are each encoded as one decimal digit.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


