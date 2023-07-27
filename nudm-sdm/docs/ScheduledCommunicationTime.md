# ScheduledCommunicationTime

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**days_of_week** | **Vec<models::DayOfWeek>** | Identifies the day(s) of the week. If absent, it indicates every day of the week.  | [optional] [default to None]
**time_of_day_start** | **String** | String with format partial-time or full-time as defined in clause 5.6 of IETF RFC 3339. Examples, 20:15:00, 20:15:00-08:00 (for 8 hours behind UTC).   | [optional] [default to None]
**time_of_day_end** | **String** | String with format partial-time or full-time as defined in clause 5.6 of IETF RFC 3339. Examples, 20:15:00, 20:15:00-08:00 (for 8 hours behind UTC).   | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


