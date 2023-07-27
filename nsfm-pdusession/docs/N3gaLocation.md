# N3gaLocation

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**n3gpp_tai** | [***models::Tai**](Tai.md) |  | [optional] [default to None]
**n3_iwf_id** | **String** | This IE shall contain the N3IWF identifier received over NGAP and shall be encoded as a  string of hexadecimal characters. Each character in the string shall take a value of \"0\"  to \"9\", \"a\" to \"f\" or \"A\" to \"F\" and shall represent 4 bits. The most significant  character representing the 4 most significant bits of the N3IWF ID shall appear first in  the string, and the character representing the 4 least significant bit of the N3IWF ID  shall appear last in the string.   | [optional] [default to None]
**ue_ipv4_addr** | **String** | String identifying a IPv4 address formatted in the 'dotted decimal' notation as defined in RFC 1166.  | [optional] [default to None]
**ue_ipv6_addr** | [***models::Ipv6Addr**](Ipv6Addr.md) |  | [optional] [default to None]
**port_number** | **u32** | Unsigned Integer, i.e. only value 0 and integers above 0 are permissible. | [optional] [default to None]
**protocol** | [***models::TransportProtocol**](TransportProtocol.md) |  | [optional] [default to None]
**tnap_id** | [***models::TnapId**](TnapId.md) |  | [optional] [default to None]
**twap_id** | [***models::TwapId**](TwapId.md) |  | [optional] [default to None]
**hfc_node_id** | [***models::HfcNodeId**](HfcNodeId.md) |  | [optional] [default to None]
**gli** | [***swagger::ByteArray**](ByteArray.md) | string with format 'bytes' as defined in OpenAPI | [optional] [default to None]
**w5gban_line_type** | [***models::LineType**](LineType.md) |  | [optional] [default to None]
**gci** | **String** | Global Cable Identifier uniquely identifying the connection between the 5G-CRG or FN-CRG to the 5GS. See clause 28.15.4 of 3GPP TS 23.003. This shall be encoded as a string per clause 28.15.4 of 3GPP TS 23.003, and compliant with the syntax specified  in clause 2.2  of IETF RFC 7542 for the username part of a NAI. The GCI value is specified in CableLabs WR-TR-5WWC-ARCH.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


