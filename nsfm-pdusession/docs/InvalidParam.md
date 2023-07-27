# InvalidParam

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**param** | **String** | If the invalid parameter is an attribute in a JSON body, this IE shall contain the  attribute's name and shall be encoded as a JSON Pointer. If the invalid parameter is  an HTTP header, this IE shall be formatted as the concatenation of the string \"header \"  plus the name of such header. If the invalid parameter is a query parameter, this IE  shall be formatted as the concatenation of the string \"query \" plus the name of such  query parameter. If the invalid parameter is a variable part in the path of a resource  URI, this IE shall contain the name of the variable, including the symbols \"{\" and \"}\"  used in OpenAPI specification as the notation to represent variable path segments.  | 
**reason** | **String** | A human-readable reason, e.g. \"must be a positive integer\". In cases involving failed  operations in a PATCH request, the reason string should identify the operation that  failed using the operation's array index to assist in correlation of the invalid  parameter with the failed operation, e.g.\" Replacement value invalid for attribute  (failed operation index= 4)\"  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


