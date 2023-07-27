# SmContext

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**pdu_session_id** | **u8** | Unsigned integer identifying a PDU session, within the range 0 to 255, as specified in  clause 11.2.3.1b, bits 1 to 8, of 3GPP TS 24.007. If the PDU Session ID is allocated by the  Core Network for UEs not supporting N1 mode, reserved range 64 to 95 is used. PDU Session ID  within the reserved range is only visible in the Core Network.   | 
**dnn** | **String** | String representing a Data Network as defined in clause 9A of 3GPP TS 23.003;  it shall contain either a DNN Network Identifier, or a full DNN with both the Network  Identifier and Operator Identifier, as specified in 3GPP TS 23.003 clause 9.1.1 and 9.1.2. It shall be coded as string in which the labels are separated by dots  (e.g. \"Label1.Label2.Label3\").  | 
**selected_dnn** | **String** | String representing a Data Network as defined in clause 9A of 3GPP TS 23.003;  it shall contain either a DNN Network Identifier, or a full DNN with both the Network  Identifier and Operator Identifier, as specified in 3GPP TS 23.003 clause 9.1.1 and 9.1.2. It shall be coded as string in which the labels are separated by dots  (e.g. \"Label1.Label2.Label3\").  | [optional] [default to None]
**s_nssai** | [***models::Snssai**](Snssai.md) |  | 
**hplmn_snssai** | [***models::Snssai**](Snssai.md) |  | [optional] [default to None]
**pdu_session_type** | [***models::PduSessionType**](PduSessionType.md) |  | 
**gpsi** | **String** | String identifying a Gpsi shall contain either an External Id or an MSISDN.  It shall be formatted as follows -External Identifier= \"extid-'extid', where 'extid'  shall be formatted according to clause 19.7.2 of 3GPP TS 23.003 that describes an  External Identifier.   | [optional] [default to None]
**h_smf_uri** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]
**smf_uri** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]
**pdu_session_ref** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]
**inter_plmn_api_root** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]
**intra_plmn_api_root** | **String** | String providing an URI formatted according to RFC 3986. | [optional] [default to None]
**pcf_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | [optional] [default to None]
**pcf_group_id** | **String** | Identifier of a group of NFs. | [optional] [default to None]
**pcf_set_id** | **String** | NF Set Identifier (see clause 28.12 of 3GPP TS 23.003), formatted as the following string \"set<Set ID>.<nftype>set.5gc.mnc<MNC>.mcc<MCC>\", or  \"set<SetID>.<NFType>set.5gc.nid<NID>.mnc<MNC>.mcc<MCC>\" with  <MCC> encoded as defined in clause 5.4.2 (\"Mcc\" data type definition)  <MNC> encoding the Mobile Network Code part of the PLMN, comprising 3 digits.    If there are only 2 significant digits in the MNC, one \"0\" digit shall be inserted    at the left side to fill the 3 digits coding of MNC.  Pattern: '^[0-9]{3}$' <NFType> encoded as a value defined in Table 6.1.6.3.3-1 of 3GPP TS 29.510 but    with lower case characters <Set ID> encoded as a string of characters consisting of    alphabetic characters (A-Z and a-z), digits (0-9) and/or the hyphen (-) and that    shall end with either an alphabetic character or a digit.   | [optional] [default to None]
**sel_mode** | [***models::DnnSelectionMode**](DnnSelectionMode.md) |  | [optional] [default to None]
**udm_group_id** | **String** | Identifier of a group of NFs. | [optional] [default to None]
**routing_indicator** | **String** |  | [optional] [default to None]
**h_nw_pub_key_id** | **i32** |  | [optional] [default to None]
**session_ambr** | [***models::Ambr**](Ambr.md) |  | 
**qos_flows_list** | [**Vec<models::QosFlowSetupItem>**](QosFlowSetupItem.md) |  | 
**h_smf_instance_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | [optional] [default to None]
**smf_instance_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | [optional] [default to None]
**pdu_session_smf_set_id** | **String** | NF Set Identifier (see clause 28.12 of 3GPP TS 23.003), formatted as the following string \"set<Set ID>.<nftype>set.5gc.mnc<MNC>.mcc<MCC>\", or  \"set<SetID>.<NFType>set.5gc.nid<NID>.mnc<MNC>.mcc<MCC>\" with  <MCC> encoded as defined in clause 5.4.2 (\"Mcc\" data type definition)  <MNC> encoding the Mobile Network Code part of the PLMN, comprising 3 digits.    If there are only 2 significant digits in the MNC, one \"0\" digit shall be inserted    at the left side to fill the 3 digits coding of MNC.  Pattern: '^[0-9]{3}$' <NFType> encoded as a value defined in Table 6.1.6.3.3-1 of 3GPP TS 29.510 but    with lower case characters <Set ID> encoded as a string of characters consisting of    alphabetic characters (A-Z and a-z), digits (0-9) and/or the hyphen (-) and that    shall end with either an alphabetic character or a digit.   | [optional] [default to None]
**pdu_session_smf_service_set_id** | **String** | NF Service Set Identifier (see clause 28.12 of 3GPP TS 23.003) formatted as the following  string \"set<Set ID>.sn<Service Name>.nfi<NF Instance ID>.5gc.mnc<MNC>.mcc<MCC>\", or  \"set<SetID>.sn<ServiceName>.nfi<NFInstanceID>.5gc.nid<NID>.mnc<MNC>.mcc<MCC>\" with  <MCC> encoded as defined in clause 5.4.2 (\"Mcc\" data type definition)   <MNC> encoding the Mobile Network Code part of the PLMN, comprising 3 digits.    If there are only 2 significant digits in the MNC, one \"0\" digit shall be inserted    at the left side to fill the 3 digits coding of MNC.  Pattern: '^[0-9]{3}$' <NID> encoded as defined in clause 5.4.2 (\"Nid\" data type definition)  <NFInstanceId> encoded as defined in clause 5.3.2  <ServiceName> encoded as defined in 3GPP TS 29.510  <Set ID> encoded as a string of characters consisting of alphabetic    characters (A-Z and a-z), digits (0-9) and/or the hyphen (-) and that shall end    with either an alphabetic character or a digit.  | [optional] [default to None]
**pdu_session_smf_binding** | [***models::SbiBindingLevel**](SbiBindingLevel.md) |  | [optional] [default to None]
**enable_pause_charging** | **bool** |  | [optional] [default to Some(false)]
**ue_ipv4_address** | **String** | String identifying a IPv4 address formatted in the 'dotted decimal' notation as defined in RFC 1166.  | [optional] [default to None]
**ue_ipv6_prefix** | [***models::Ipv6Prefix**](Ipv6Prefix.md) |  | [optional] [default to None]
**eps_pdn_cnx_info** | [***models::EpsPdnCnxInfo**](EpsPdnCnxInfo.md) |  | [optional] [default to None]
**eps_bearer_info** | [**Vec<models::EpsBearerInfo>**](EpsBearerInfo.md) |  | [optional] [default to None]
**max_integrity_protected_data_rate** | [***models::MaxIntegrityProtectedDataRate**](MaxIntegrityProtectedDataRate.md) |  | [optional] [default to None]
**max_integrity_protected_data_rate_dl** | [***models::MaxIntegrityProtectedDataRate**](MaxIntegrityProtectedDataRate.md) |  | [optional] [default to None]
**always_on_granted** | **bool** |  | [optional] [default to Some(false)]
**up_security** | [***models::UpSecurity**](UpSecurity.md) |  | [optional] [default to None]
**h_smf_service_instance_id** | **String** |  | [optional] [default to None]
**smf_service_instance_id** | **String** |  | [optional] [default to None]
**recovery_time** | [**chrono::DateTime::<chrono::Utc>**](DateTime.md) | string with format 'date-time' as defined in OpenAPI. | [optional] [default to None]
**forwarding_ind** | **bool** |  | [optional] [default to Some(false)]
**psa_tunnel_info** | [***models::TunnelInfo**](TunnelInfo.md) |  | [optional] [default to None]
**charging_id** | **String** |  | [optional] [default to None]
**charging_info** | [***models::ChargingInformation**](ChargingInformation.md) |  | [optional] [default to None]
**roaming_charging_profile** | [***models::RoamingChargingProfile**](RoamingChargingProfile.md) |  | [optional] [default to None]
**nef_ext_buf_support_ind** | **bool** |  | [optional] [default to Some(false)]
**ipv6_index** | **i32** | Represents information that identifies which IP pool or external server is used to allocate the IP address.  | [optional] [default to None]
**dn_aaa_address** | [***models::IpAddress**](IpAddress.md) |  | [optional] [default to None]
**redundant_pdu_session_info** | [***models::RedundantPduSessionInformation**](RedundantPduSessionInformation.md) |  | [optional] [default to None]
**ran_tunnel_info** | [***models::QosFlowTunnel**](QosFlowTunnel.md) |  | [optional] [default to None]
**add_ran_tunnel_info** | [**Vec<models::QosFlowTunnel>**](QosFlowTunnel.md) |  | [optional] [default to None]
**red_ran_tunnel_info** | [***models::QosFlowTunnel**](QosFlowTunnel.md) |  | [optional] [default to None]
**add_red_ran_tunnel_info** | [**Vec<models::QosFlowTunnel>**](QosFlowTunnel.md) |  | [optional] [default to None]
**nspu_support_ind** | **bool** |  | [optional] [default to None]
**smf_binding_info** | **String** |  | [optional] [default to None]
**satellite_backhaul_cat** | [***models::SatelliteBackhaulCategory**](SatelliteBackhaulCategory.md) |  | [optional] [default to None]
**ssc_mode** | **String** |  | [optional] [default to None]
**dlset_support_ind** | **bool** |  | [optional] [default to None]
**n9fsc_support_ind** | **bool** |  | [optional] [default to None]
**disaster_roaming_ind** | **bool** |  | [optional] [default to Some(false)]
**anchor_smf_oauth2_required** | **bool** |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


