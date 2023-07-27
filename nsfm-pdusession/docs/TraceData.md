# TraceData

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**trace_ref** | **String** | Trace Reference (see 3GPP TS 32.422).It shall be encoded as the concatenation of MCC, MNC and Trace ID as follows: 'MCC'<MNC'-'Trace ID'The Trace ID shall be encoded as a 3 octet string in hexadecimal representation. Each character in the Trace ID string shall take a value of \"0\" to \"9\", \"a\" to \"f\" or \"A\" to \"F\" and shall represent 4 bits. The most significant character representing the 4 most significant bits of the Trace ID shall appear first  in the string, and the character representing the 4 least significant bit of the Trace ID shall appear last in the string.  | 
**trace_depth** | [***models::TraceDepth**](TraceDepth.md) |  | 
**ne_type_list** | **String** | List of NE Types (see 3GPP TS 32.422).It shall be encoded as an octet string in hexadecimal representation. Each character in the string shall take a value of \"0\" to \"9\", \"a\" to \"f\" or \"A\" to \"F\" and shall represent 4 bits. The most significant character representing the 4 most significant bits shall appear first in the string, and the character representing the 4 least significant bit shall appear last in the string.Octets shall be coded according to 3GPP TS 32.422.  | 
**event_list** | **String** | Triggering events (see 3GPP TS 32.422).It shall be encoded as an octet string in hexadecimal representation. Each character in the string shall take a value of \"0\" to \"9\", \"a\" to \"f\" or \"A\" to \"F\" and shall represent 4 bits. The most significant character representing the 4 most significant bits shall appear first in the string, and the character representing the 4 least significant bit shall appear last in the string. Octets shall be coded according to 3GPP TS 32.422.  | 
**collection_entity_ipv4_addr** | **String** | String identifying a IPv4 address formatted in the 'dotted decimal' notation as defined in RFC 1166.  | [optional] [default to None]
**collection_entity_ipv6_addr** | [***models::Ipv6Addr**](Ipv6Addr.md) |  | [optional] [default to None]
**interface_list** | **String** | List of Interfaces (see 3GPP TS 32.422).It shall be encoded as an octet string in hexadecimal representation. Each character in the string shall take a value of \"0\" to \"9\", \"a\" to \"f\" or \"A\" to \"F\" and shall represent 4 bits. The most significant character representing the 4 most significant bits shall appear first in the string, and the character representing the  4 least significant bit shall appear last in the string. Octets shall be coded according to 3GPP TS 32.422. If this attribute is not present, all the interfaces applicable to the list of NE types indicated in the neTypeList attribute should be traced.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


