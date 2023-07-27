# SmContextStatusNotification

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**status_info** | [***models::StatusInfo**](StatusInfo.md) |  | 
**small_data_rate_status** | [***models::SmallDataRateStatus**](SmallDataRateStatus.md) |  | [optional] [default to None]
**apn_rate_status** | [***models::ApnRateStatus**](ApnRateStatus.md) |  | [optional] [default to None]
**ddn_failure_status** | **bool** |  | [optional] [default to Some(false)]
**notify_correlation_ids_forddn_failure** | **Vec<String>** |  | [optional] [default to None]
**new_intermediate_smf_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | [optional] [default to None]
**new_smf_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | [optional] [default to None]
**new_smf_set_id** | **String** | NF Set Identifier (see clause 28.12 of 3GPP TS 23.003), formatted as the following string \"set<Set ID>.<nftype>set.5gc.mnc<MNC>.mcc<MCC>\", or  \"set<SetID>.<NFType>set.5gc.nid<NID>.mnc<MNC>.mcc<MCC>\" with  <MCC> encoded as defined in clause 5.4.2 (\"Mcc\" data type definition)  <MNC> encoding the Mobile Network Code part of the PLMN, comprising 3 digits.    If there are only 2 significant digits in the MNC, one \"0\" digit shall be inserted    at the left side to fill the 3 digits coding of MNC.  Pattern: '^[0-9]{3}$' <NFType> encoded as a value defined in Table 6.1.6.3.3-1 of 3GPP TS 29.510 but    with lower case characters <Set ID> encoded as a string of characters consisting of    alphabetic characters (A-Z and a-z), digits (0-9) and/or the hyphen (-) and that    shall end with either an alphabetic character or a digit.   | [optional] [default to None]
**old_smf_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | [optional] [default to None]
**old_sm_context_ref** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]
**alt_anchor_smf_uri** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]
**alt_anchor_smf_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | [optional] [default to None]
**target_dnai_info** | [***models::TargetDnaiInfo**](TargetDnaiInfo.md) |  | [optional] [default to None]
**old_pdu_session_ref** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]
**inter_plmn_api_root** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


