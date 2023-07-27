# VsmfUpdatedData

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**qos_flows_add_mod_list** | [**Vec<models::QosFlowItem>**](QosFlowItem.md) |  | [optional] [default to None]
**qos_flows_rel_list** | [**Vec<models::QosFlowItem>**](QosFlowItem.md) |  | [optional] [default to None]
**qos_flows_failedto_add_mod_list** | [**Vec<models::QosFlowItem>**](QosFlowItem.md) |  | [optional] [default to None]
**qos_flows_failedto_rel_list** | [**Vec<models::QosFlowItem>**](QosFlowItem.md) |  | [optional] [default to None]
**n1_sm_info_from_ue** | [***models::RefToBinaryData**](RefToBinaryData.md) |  | [optional] [default to None]
**unknown_n1_sm_info** | [***models::RefToBinaryData**](RefToBinaryData.md) |  | [optional] [default to None]
**ue_location** | [***models::UserLocation**](UserLocation.md) |  | [optional] [default to None]
**ue_time_zone** | **String** | String with format \"time-numoffset\" optionally appended by \"daylightSavingTime\", where  - \"time-numoffset\" shall represent the time zone adjusted for daylight saving time and be    encoded as time-numoffset as defined in clause 5.6 of IETF RFC 3339;  - \"daylightSavingTime\" shall represent the adjustment that has been made and shall be    encoded as \"+1\" or \"+2\" for a +1 or +2 hours adjustment.   The example is for 8 hours behind UTC, +1 hour adjustment for Daylight Saving Time.  | [optional] [default to None]
**add_ue_location** | [***models::UserLocation**](UserLocation.md) |  | [optional] [default to None]
**assigned_ebi_list** | [**Vec<models::EbiArpMapping>**](EbiArpMapping.md) |  | [optional] [default to None]
**failed_to_assign_ebi_list** | [**Vec<models::Arp>**](Arp.md) |  | [optional] [default to None]
**released_ebi_list** | **Vec<models::EpsBearerId>** |  | [optional] [default to None]
**secondary_rat_usage_report** | [**Vec<models::SecondaryRatUsageReport>**](SecondaryRatUsageReport.md) |  | [optional] [default to None]
**secondary_rat_usage_info** | [**Vec<models::SecondaryRatUsageInfo>**](SecondaryRatUsageInfo.md) |  | [optional] [default to None]
**n4_info** | [***models::N4Information**](N4Information.md) |  | [optional] [default to None]
**n4_info_ext1** | [***models::N4Information**](N4Information.md) |  | [optional] [default to None]
**n4_info_ext2** | [***models::N4Information**](N4Information.md) |  | [optional] [default to None]
**n4_info_ext3** | [***models::N4Information**](N4Information.md) |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


