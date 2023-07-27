# N4Information

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**n4_message_type** | [***models::N4MessageType**](N4MessageType.md) |  | 
**n4_message_payload** | [***models::RefToBinaryData**](RefToBinaryData.md) |  | 
**n4_dnai_info** | [***models::DnaiInformation**](DnaiInformation.md) |  | [optional] [default to None]
**psa_upf_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | [optional] [default to None]
**ul_cl_bp_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | [optional] [default to None]
**n9_ul_pdr_id_list** | **Vec<models::Uint16>** |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


