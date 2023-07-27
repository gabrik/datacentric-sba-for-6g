# SmContextReleaseData

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**cause** | [***models::Cause**](Cause.md) |  | [optional] [default to None]
**ng_ap_cause** | [***models::NgApCause**](NgApCause.md) |  | [optional] [default to None]
**param_5g_mm_cause_value** | **u32** | Unsigned Integer, i.e. only value 0 and integers above 0 are permissible. | [optional] [default to None]
**ue_location** | [***models::UserLocation**](UserLocation.md) |  | [optional] [default to None]
**ue_time_zone** | **String** | String with format \"time-numoffset\" optionally appended by \"daylightSavingTime\", where  - \"time-numoffset\" shall represent the time zone adjusted for daylight saving time and be    encoded as time-numoffset as defined in clause 5.6 of IETF RFC 3339;  - \"daylightSavingTime\" shall represent the adjustment that has been made and shall be    encoded as \"+1\" or \"+2\" for a +1 or +2 hours adjustment.   The example is for 8 hours behind UTC, +1 hour adjustment for Daylight Saving Time.  | [optional] [default to None]
**add_ue_location** | [***models::UserLocation**](UserLocation.md) |  | [optional] [default to None]
**vsmf_release_only** | **bool** |  | [optional] [default to Some(false)]
**n2_sm_info** | [***models::RefToBinaryData**](RefToBinaryData.md) |  | [optional] [default to None]
**n2_sm_info_type** | [***models::N2SmInfoType**](N2SmInfoType.md) |  | [optional] [default to None]
**ismf_release_only** | **bool** |  | [optional] [default to Some(false)]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


