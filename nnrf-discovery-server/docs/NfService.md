# NfService

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**service_instance_id** | **String** |  | 
**service_name** | [***models::ServiceName**](ServiceName.md) |  | 
**versions** | [**Vec<models::NfServiceVersion>**](NFServiceVersion.md) |  | 
**scheme** | [***models::UriScheme**](UriScheme.md) |  | 
**nf_service_status** | [***models::NfServiceStatus**](NFServiceStatus.md) |  | 
**fqdn** | **String** | Fully Qualified Domain Name | [optional] [default to None]
**inter_plmn_fqdn** | **String** | Fully Qualified Domain Name | [optional] [default to None]
**ip_end_points** | [**Vec<models::IpEndPoint>**](IpEndPoint.md) |  | [optional] [default to None]
**api_prefix** | **String** |  | [optional] [default to None]
**default_notification_subscriptions** | [**Vec<models::DefaultNotificationSubscription>**](DefaultNotificationSubscription.md) |  | [optional] [default to None]
**capacity** | **u16** |  | [optional] [default to None]
**load** | **u8** |  | [optional] [default to None]
**load_time_stamp** | [**chrono::DateTime::<chrono::Utc>**](DateTime.md) | string with format 'date-time' as defined in OpenAPI. | [optional] [default to None]
**priority** | **u16** |  | [optional] [default to None]
**recovery_time** | [**chrono::DateTime::<chrono::Utc>**](DateTime.md) | string with format 'date-time' as defined in OpenAPI. | [optional] [default to None]
**supported_features** | **String** | A string used to indicate the features supported by an API that is used as defined in clause  6.6 in 3GPP TS 29.500. The string shall contain a bitmask indicating supported features in  hexadecimal representation Each character in the string shall take a value of \"0\" to \"9\",  \"a\" to \"f\" or \"A\" to \"F\" and shall represent the support of 4 features as described in  table 5.2.2-3. The most significant character representing the highest-numbered features shall  appear first in the string, and the character representing features 1 to 4 shall appear last  in the string. The list of features and their numbering (starting with 1) are defined  separately for each API. If the string contains a lower number of characters than there are  defined features for an API, all features that would be represented by characters that are not  present in the string are not supported.  | [optional] [default to None]
**nf_service_set_id_list** | **Vec<models::NfServiceSetId>** |  | [optional] [default to None]
**s_nssais** | [**Vec<models::ExtSnssai>**](ExtSnssai.md) |  | [optional] [default to None]
**per_plmn_snssai_list** | [**Vec<models::PlmnSnssai>**](PlmnSnssai.md) |  | [optional] [default to None]
**vendor_id** | **String** | Vendor ID of the NF Service instance (Private Enterprise Number assigned by IANA) | [optional] [default to None]
**supported_vendor_specific_features** | [**std::collections::HashMap<String, Vec<models::VendorSpecificFeature>>**](array.md) | The key of the map is the IANA-assigned SMI Network Management Private Enterprise Codes  | [optional] [default to None]
**oauth2_required** | **bool** |  | [optional] [default to None]
**allowed_operations_per_nf_type** | [**std::collections::HashMap<String, Vec<String>>**](array.md) | A map (list of key-value pairs) where NF Type serves as key | [optional] [default to None]
**allowed_operations_per_nf_instance** | [**std::collections::HashMap<String, Vec<String>>**](array.md) | A map (list of key-value pairs) where NF Instance Id serves as key | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


