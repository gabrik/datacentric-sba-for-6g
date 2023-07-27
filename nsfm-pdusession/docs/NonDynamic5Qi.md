# NonDynamic5Qi

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**priority_level** | **u8** | Unsigned integer indicating the 5QI Priority Level (see clauses 5.7.3.3 and 5.7.4 of 3GPP TS 23.501, within the range 1 to 127.Values are ordered in decreasing order of priority,  i.e. with 1 as the highest priority and 127 as the lowest priority.   | [optional] [default to None]
**aver_window** | **u16** | Unsigned integer indicating Averaging Window (see clause 5.7.3.6 and 5.7.4 of 3GPP TS 23.501), expressed in milliseconds. | [optional] [default to Some(2000)]
**max_data_burst_vol** | **u16** | Unsigned integer indicating Maximum Data Burst Volume (see clauses 5.7.3.7 and 5.7.4 of 3GPP TS 23.501), expressed in Bytes.   | [optional] [default to None]
**ext_max_data_burst_vol** | **u32** | Unsigned integer indicating Maximum Data Burst Volume (see clauses 5.7.3.7 and 5.7.4 of 3GPP TS 23.501), expressed in Bytes.   | [optional] [default to None]
**cn_packet_delay_budget_dl** | **u32** | Unsigned integer indicating Packet Delay Budget (see clauses 5.7.3.4 and 5.7.4 of 3GPP TS 23.501 [8])), expressed in 0.01 milliseconds.  | [optional] [default to None]
**cn_packet_delay_budget_ul** | **u32** | Unsigned integer indicating Packet Delay Budget (see clauses 5.7.3.4 and 5.7.4 of 3GPP TS 23.501 [8])), expressed in 0.01 milliseconds.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


