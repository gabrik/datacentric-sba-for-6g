# DnnUpfInfoItem

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dnn** | **String** | String representing a Data Network as defined in clause 9A of 3GPP TS 23.003;  it shall contain either a DNN Network Identifier, or a full DNN with both the Network  Identifier and Operator Identifier, as specified in 3GPP TS 23.003 clause 9.1.1 and 9.1.2. It shall be coded as string in which the labels are separated by dots  (e.g. \"Label1.Label2.Label3\").  | 
**dnai_list** | **Vec<models::Dnai>** |  | [optional] [default to None]
**pdu_session_types** | [**Vec<models::PduSessionType>**](PduSessionType.md) |  | [optional] [default to None]
**ipv4_address_ranges** | [**Vec<models::Ipv4AddressRange>**](Ipv4AddressRange.md) |  | [optional] [default to None]
**ipv6_prefix_ranges** | [**Vec<models::Ipv6PrefixRange>**](Ipv6PrefixRange.md) |  | [optional] [default to None]
**ipv4_index_list** | [**Vec<models::IpIndex>**](IpIndex.md) |  | [optional] [default to None]
**ipv6_index_list** | [**Vec<models::IpIndex>**](IpIndex.md) |  | [optional] [default to None]
**dnai_nw_instance_list** | **std::collections::HashMap<String, String>** | Map of network instance per DNAI for the DNN, where the key of the map is the DNAI. When present, the value of each entry of the map shall contain a N6 network instance that is configured for the DNAI indicated by the key.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


