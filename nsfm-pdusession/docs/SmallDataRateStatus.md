# SmallDataRateStatus

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**remain_packets_ul** | **u32** | When present, it shall contain the number of packets the UE is allowed to send uplink in the given time unit for the given PDU session (see clause 5.31.14.3 of 3GPP TS 23.501.  | [optional] [default to None]
**remain_packets_dl** | **u32** | When present it shall contain the number of packets the AF is allowed to send downlink in the given time unit for the given PDU session (see clause 5.31.14.3 of 3GPP TS 23.501.  | [optional] [default to None]
**validity_time** | [**chrono::DateTime::<chrono::Utc>**](DateTime.md) | string with format 'date-time' as defined in OpenAPI. | [optional] [default to None]
**remain_ex_reports_ul** | **u32** | When present, it shall indicate number of additional exception reports the UE is allowed to send uplink in the given time  unit for  the given PDU session (see clause 5.31.14.3 of 3GPP TS 23.501.  | [optional] [default to None]
**remain_ex_reports_dl** | **u32** | When present, it shall indicate number of additional exception reports the AF is allowed to send downlink  in the given time unit for the given PDU session (see clause 5.31.14.3 in 3GPP TS 23.501  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


