# GeraLocation

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**location_number** | **String** | Location number within the PLMN. See 3GPP TS 23.003, clause 4.5. | [optional] [default to None]
**cgi** | [***models::CellGlobalId**](CellGlobalId.md) |  | [optional] [default to None]
**rai** | [***models::RoutingAreaId**](RoutingAreaId.md) |  | [optional] [default to None]
**sai** | [***models::ServiceAreaId**](ServiceAreaId.md) |  | [optional] [default to None]
**lai** | [***models::LocationAreaId**](LocationAreaId.md) |  | [optional] [default to None]
**vlr_number** | **String** | VLR number. See 3GPP TS 23.003 clause 5.1. | [optional] [default to None]
**msc_number** | **String** | MSC number. See 3GPP TS 23.003 clause 5.1. | [optional] [default to None]
**age_of_location_information** | **u16** | The value represents the elapsed time in minutes since the last network contact of the mobile station. Value \"0\" indicates that the location information was obtained after a successful paging procedure for  Active Location Retrieval when the UE is in idle mode or after a successful location reporting procedure the UE is in connected mode. Any other value than \"0\" indicates that the location information is the last known one. See 3GPP TS 29.002 clause 17.7.8.  | [optional] [default to None]
**ue_location_timestamp** | [**chrono::DateTime::<chrono::Utc>**](DateTime.md) | string with format 'date-time' as defined in OpenAPI. | [optional] [default to None]
**geographical_information** | **String** | Refer to geographical Information.See 3GPP TS 23.032 clause 7.3.2. Only the description of an ellipsoid point with uncertainty circle is allowed to be used.  | [optional] [default to None]
**geodetic_information** | **String** | Refers to Calling Geodetic Location.See ITU-T Recommendation Q.763 (1999) clause 3.88.2.  Only the description of an ellipsoid point with uncertainty circle is allowed to be used.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


