# SharedData

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**shared_data_id** | **String** |  | 
**shared_am_data** | [***models::AccessAndMobilitySubscriptionData**](AccessAndMobilitySubscriptionData.md) |  | [optional] [default to None]
**shared_sms_subs_data** | [***models::SmsSubscriptionData**](SmsSubscriptionData.md) |  | [optional] [default to None]
**shared_sms_mng_subs_data** | [***models::SmsManagementSubscriptionData**](SmsManagementSubscriptionData.md) |  | [optional] [default to None]
**shared_dnn_configurations** | [**std::collections::HashMap<String, models::DnnConfiguration>**](DnnConfiguration.md) | A map(list of key-value pairs) where Dnn, or optionally the Wildcard DNN, serves as key of DnnConfiguration | [optional] [default to None]
**shared_trace_data** | [***models::TraceData**](TraceData.md) |  | [optional] [default to None]
**shared_snssai_infos** | [**std::collections::HashMap<String, models::SnssaiInfo>**](SnssaiInfo.md) | A map(list of key-value pairs) where singleNssai serves as key of SnssaiInfo | [optional] [default to None]
**shared_vn_group_datas** | [**std::collections::HashMap<String, models::VnGroupData>**](VnGroupData.md) | A map(list of key-value pairs) where GroupId serves as key of VnGroupData | [optional] [default to None]
**treatment_instructions** | [**std::collections::HashMap<String, models::SharedDataTreatmentInstruction>**](SharedDataTreatmentInstruction.md) | A map(list of key-value pairs) where JSON pointer pointing to an attribute within the SharedData serves as key of SharedDataTreatmentInstruction | [optional] [default to None]
**shared_sm_subs_data** | [***models::SessionManagementSubscriptionData**](SessionManagementSubscriptionData.md) |  | [optional] [default to None]
**shared_ecs_addr_config_info** | [***models::EcsAddrConfigInfo**](EcsAddrConfigInfo.md) |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


