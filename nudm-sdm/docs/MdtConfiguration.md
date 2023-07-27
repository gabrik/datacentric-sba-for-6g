# MdtConfiguration

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**job_type** | [***models::JobType**](JobType.md) |  | 
**report_type** | [***models::ReportTypeMdt**](ReportTypeMdt.md) |  | [optional] [default to None]
**area_scope** | [***models::AreaScope**](AreaScope.md) |  | [optional] [default to None]
**measurement_lte_list** | [**Vec<models::MeasurementLteForMdt>**](MeasurementLteForMdt.md) |  | [optional] [default to None]
**measurement_nr_list** | [**Vec<models::MeasurementNrForMdt>**](MeasurementNrForMdt.md) |  | [optional] [default to None]
**sensor_measurement_list** | [**Vec<models::SensorMeasurement>**](SensorMeasurement.md) |  | [optional] [default to None]
**reporting_trigger_list** | [**Vec<models::ReportingTrigger>**](ReportingTrigger.md) |  | [optional] [default to None]
**report_interval** | [***models::ReportIntervalMdt**](ReportIntervalMdt.md) |  | [optional] [default to None]
**report_interval_nr** | [***models::ReportIntervalNrMdt**](ReportIntervalNrMdt.md) |  | [optional] [default to None]
**report_amount** | [***models::ReportAmountMdt**](ReportAmountMdt.md) |  | [optional] [default to None]
**event_threshold_rsrp** | **u8** | This IE shall be present if the report trigger parameter is configured for A2 event reporting or A2 event triggered periodic reporting and the job type parameter is configured for Immediate MDT or combined Immediate MDT and Trace in LTE. When present, this IE shall indicate the Event Threshold for RSRP, and the value shall be between 0-97.  | [optional] [default to None]
**event_threshold_rsrp_nr** | **u8** | This IE shall be present if the report trigger parameter is configured for A2 event reporting or A2 event triggered periodic reporting and the job type parameter is configured for Immediate MDT or combined Immediate MDT and Trace in NR. When present, this IE shall indicate the Event Threshold for RSRP, and the value shall be between 0-127.  | [optional] [default to None]
**event_threshold_rsrq** | **u8** | This IE shall be present if the report trigger parameter is configured for A2 event reporting or A2 event triggered periodic reporting and the job type parameter is configured for Immediate MDT or combined Immediate MDT and Trace in LTE.When present, this IE shall indicate the Event Threshold for RSRQ, and the value shall be between 0-34.  | [optional] [default to None]
**event_threshold_rsrq_nr** | **u8** | This IE shall be present if the report trigger parameter is configured for A2 event reporting or A2 event triggered periodic reporting and the job type parameter is configured for Immediate MDT or combined Immediate MDT and Trace in NR.When present, this IE shall indicate the Event Threshold for RSRQ, and the value shall be between 0-127.  | [optional] [default to None]
**event_list** | [**Vec<models::EventForMdt>**](EventForMdt.md) |  | [optional] [default to None]
**logging_interval** | [***models::LoggingIntervalMdt**](LoggingIntervalMdt.md) |  | [optional] [default to None]
**logging_interval_nr** | [***models::LoggingIntervalNrMdt**](LoggingIntervalNrMdt.md) |  | [optional] [default to None]
**logging_duration** | [***models::LoggingDurationMdt**](LoggingDurationMdt.md) |  | [optional] [default to None]
**logging_duration_nr** | [***models::LoggingDurationNrMdt**](LoggingDurationNrMdt.md) |  | [optional] [default to None]
**positioning_method** | [***models::PositioningMethodMdt**](PositioningMethodMdt.md) |  | [optional] [default to None]
**add_positioning_method_list** | [**Vec<models::PositioningMethodMdt>**](PositioningMethodMdt.md) |  | [optional] [default to None]
**collection_period_rmm_lte** | [***models::CollectionPeriodRmmLteMdt**](CollectionPeriodRmmLteMdt.md) |  | [optional] [default to None]
**collection_period_rmm_nr** | [***models::CollectionPeriodRmmNrMdt**](CollectionPeriodRmmNrMdt.md) |  | [optional] [default to None]
**measurement_period_lte** | [***models::MeasurementPeriodLteMdt**](MeasurementPeriodLteMdt.md) |  | [optional] [default to None]
**mdt_allowed_plmn_id_list** | [**Vec<models::PlmnId>**](PlmnId.md) |  | [optional] [default to None]
**mbsfn_area_list** | [**Vec<models::MbsfnArea>**](MbsfnArea.md) |  | [optional] [default to None]
**inter_freq_target_list** | [**Vec<models::InterFreqTargetInfo>**](InterFreqTargetInfo.md) |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


