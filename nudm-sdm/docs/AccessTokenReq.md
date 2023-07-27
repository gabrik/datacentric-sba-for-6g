# AccessTokenReq

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**grant_type** | **String** |  | 
**nf_instance_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | 
**nf_type** | [***models::NfType**](NFType.md) |  | [optional] [default to None]
**target_nf_type** | [***models::NfType**](NFType.md) |  | [optional] [default to None]
**scope** | **String** |  | 
**target_nf_instance_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | [optional] [default to None]
**requester_plmn** | [***models::PlmnId**](PlmnId.md) |  | [optional] [default to None]
**requester_plmn_list** | [**Vec<models::PlmnId>**](PlmnId.md) |  | [optional] [default to None]
**requester_snssai_list** | [**Vec<models::Snssai>**](Snssai.md) |  | [optional] [default to None]
**requester_fqdn** | **String** | Fully Qualified Domain Name | [optional] [default to None]
**requester_snpn_list** | [**Vec<models::PlmnIdNid>**](PlmnIdNid.md) |  | [optional] [default to None]
**target_plmn** | [***models::PlmnId**](PlmnId.md) |  | [optional] [default to None]
**target_snpn** | [***models::PlmnIdNid**](PlmnIdNid.md) |  | [optional] [default to None]
**target_snssai_list** | [**Vec<models::Snssai>**](Snssai.md) |  | [optional] [default to None]
**target_nsi_list** | **Vec<String>** |  | [optional] [default to None]
**target_nf_set_id** | **String** | NF Set Identifier (see clause 28.12 of 3GPP TS 23.003), formatted as the following string \"set<Set ID>.<nftype>set.5gc.mnc<MNC>.mcc<MCC>\", or  \"set<SetID>.<NFType>set.5gc.nid<NID>.mnc<MNC>.mcc<MCC>\" with  <MCC> encoded as defined in clause 5.4.2 (\"Mcc\" data type definition)  <MNC> encoding the Mobile Network Code part of the PLMN, comprising 3 digits.    If there are only 2 significant digits in the MNC, one \"0\" digit shall be inserted    at the left side to fill the 3 digits coding of MNC.  Pattern: '^[0-9]{3}$' <NFType> encoded as a value defined in Table 6.1.6.3.3-1 of 3GPP TS 29.510 but    with lower case characters <Set ID> encoded as a string of characters consisting of    alphabetic characters (A-Z and a-z), digits (0-9) and/or the hyphen (-) and that    shall end with either an alphabetic character or a digit.   | [optional] [default to None]
**target_nf_service_set_id** | **String** | NF Service Set Identifier (see clause 28.12 of 3GPP TS 23.003) formatted as the following  string \"set<Set ID>.sn<Service Name>.nfi<NF Instance ID>.5gc.mnc<MNC>.mcc<MCC>\", or  \"set<SetID>.sn<ServiceName>.nfi<NFInstanceID>.5gc.nid<NID>.mnc<MNC>.mcc<MCC>\" with  <MCC> encoded as defined in clause 5.4.2 (\"Mcc\" data type definition)   <MNC> encoding the Mobile Network Code part of the PLMN, comprising 3 digits.    If there are only 2 significant digits in the MNC, one \"0\" digit shall be inserted    at the left side to fill the 3 digits coding of MNC.  Pattern: '^[0-9]{3}$' <NID> encoded as defined in clause 5.4.2 (\"Nid\" data type definition)  <NFInstanceId> encoded as defined in clause 5.3.2  <ServiceName> encoded as defined in 3GPP TS 29.510  <Set ID> encoded as a string of characters consisting of alphabetic    characters (A-Z and a-z), digits (0-9) and/or the hyphen (-) and that shall end    with either an alphabetic character or a digit.  | [optional] [default to None]
**hnrf_access_token_uri** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]
**source_nf_instance_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


