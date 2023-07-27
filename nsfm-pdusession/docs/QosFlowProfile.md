# QosFlowProfile

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**param_5qi** | **u8** | Unsigned integer representing a 5G QoS Identifier (see clause 5.7.2.1 of 3GPP TS 23.501, within the range 0 to 255.  | 
**non_dynamic5_qi** | [***models::NonDynamic5Qi**](NonDynamic5Qi.md) |  | [optional] [default to None]
**dynamic5_qi** | [***models::Dynamic5Qi**](Dynamic5Qi.md) |  | [optional] [default to None]
**arp** | [***models::Arp**](Arp.md) |  | [optional] [default to None]
**gbr_qos_flow_info** | [***models::GbrQosFlowInformation**](GbrQosFlowInformation.md) |  | [optional] [default to None]
**rqa** | [***models::ReflectiveQoSAttribute**](ReflectiveQoSAttribute.md) |  | [optional] [default to None]
**additional_qos_flow_info** | [***models::AdditionalQosFlowInfo**](AdditionalQosFlowInfo.md) |  | [optional] [default to None]
**qos_monitoring_req** | [***models::QosMonitoringReq**](QosMonitoringReq.md) |  | [optional] [default to None]
**qos_rep_period** | **i32** | indicating a time in seconds. | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


