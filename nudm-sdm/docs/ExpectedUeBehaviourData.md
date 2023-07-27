# ExpectedUeBehaviourData

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**stationary_indication** | [***models::StationaryIndication**](StationaryIndication.md) |  | [optional] [default to None]
**communication_duration_time** | **i32** | indicating a time in seconds. | [optional] [default to None]
**periodic_time** | **i32** | indicating a time in seconds. | [optional] [default to None]
**scheduled_communication_time** | [***models::ScheduledCommunicationTime**](ScheduledCommunicationTime.md) |  | [optional] [default to None]
**scheduled_communication_type** | [***models::ScheduledCommunicationType**](ScheduledCommunicationType.md) |  | [optional] [default to None]
**expected_umts** | [**Vec<models::LocationArea>**](LocationArea.md) | Identifies the UE's expected geographical movement. The attribute is only applicable in 5G. | [optional] [default to None]
**traffic_profile** | [***models::TrafficProfile**](TrafficProfile.md) |  | [optional] [default to None]
**battery_indication** | [***models::BatteryIndication**](BatteryIndication.md) |  | [optional] [default to None]
**validity_time** | [**chrono::DateTime::<chrono::Utc>**](DateTime.md) | string with format 'date-time' as defined in OpenAPI. | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


