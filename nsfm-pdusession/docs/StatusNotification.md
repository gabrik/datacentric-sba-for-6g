# StatusNotification

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**status_info** | [***models::StatusInfo**](StatusInfo.md) |  | 
**small_data_rate_status** | [***models::SmallDataRateStatus**](SmallDataRateStatus.md) |  | [optional] [default to None]
**apn_rate_status** | [***models::ApnRateStatus**](ApnRateStatus.md) |  | [optional] [default to None]
**target_dnai_info** | [***models::TargetDnaiInfo**](TargetDnaiInfo.md) |  | [optional] [default to None]
**old_pdu_session_ref** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]
**new_smf_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | [optional] [default to None]
**eps_pdn_cnx_info** | [***models::EpsPdnCnxInfo**](EpsPdnCnxInfo.md) |  | [optional] [default to None]
**inter_plmn_api_root** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]
**intra_plmn_api_root** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


