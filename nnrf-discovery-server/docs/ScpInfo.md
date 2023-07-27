# ScpInfo

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**scp_domain_info_list** | [**std::collections::HashMap<String, models::ScpDomainInfo>**](ScpDomainInfo.md) | A map (list of key-value pairs) where the key of the map shall be the string identifying an SCP domain  | [optional] [default to None]
**scp_prefix** | **String** |  | [optional] [default to None]
**scp_ports** | **std::collections::HashMap<String, i32>** | Port numbers for HTTP and HTTPS. The key of the map shall be \"http\" or \"https\".  | [optional] [default to None]
**address_domains** | **Vec<String>** |  | [optional] [default to None]
**ipv4_addresses** | **Vec<models::Ipv4Addr>** |  | [optional] [default to None]
**ipv6_prefixes** | [**Vec<models::Ipv6Prefix>**](Ipv6Prefix.md) |  | [optional] [default to None]
**ipv4_addr_ranges** | [**Vec<models::Ipv4AddressRange>**](Ipv4AddressRange.md) |  | [optional] [default to None]
**ipv6_prefix_ranges** | [**Vec<models::Ipv6PrefixRange>**](Ipv6PrefixRange.md) |  | [optional] [default to None]
**served_nf_set_id_list** | **Vec<models::NfSetId>** |  | [optional] [default to None]
**remote_plmn_list** | [**Vec<models::PlmnId>**](PlmnId.md) |  | [optional] [default to None]
**remote_snpn_list** | [**Vec<models::PlmnIdNid>**](PlmnIdNid.md) |  | [optional] [default to None]
**ip_reachability** | [***models::IpReachability**](IpReachability.md) |  | [optional] [default to None]
**scp_capabilities** | [**Vec<models::ScpCapability>**](ScpCapability.md) |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


