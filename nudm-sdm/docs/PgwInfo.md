# PgwInfo

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dnn** | **String** | String representing a Data Network as defined in clause 9A of 3GPP TS 23.003;  it shall contain either a DNN Network Identifier, or a full DNN with both the Network  Identifier and Operator Identifier, as specified in 3GPP TS 23.003 clause 9.1.1 and 9.1.2. It shall be coded as string in which the labels are separated by dots  (e.g. \"Label1.Label2.Label3\").  | 
**pgw_fqdn** | **String** | Fully Qualified Domain Name | 
**pgw_ip_addr** | [***models::IpAddress**](IpAddress.md) |  | [optional] [default to None]
**plmn_id** | [***models::PlmnId**](PlmnId.md) |  | [optional] [default to None]
**epdg_ind** | **bool** |  | [optional] [default to Some(false)]
**pcf_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | [optional] [default to None]
**registration_time** | [**chrono::DateTime::<chrono::Utc>**](DateTime.md) | string with format 'date-time' as defined in OpenAPI. | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


