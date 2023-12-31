# NfProfile

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**nf_instance_id** | [***uuid::Uuid**](UUID.md) | String uniquely identifying a NF instance. The format of the NF Instance ID shall be a  Universally Unique Identifier (UUID) version 4, as described in IETF RFC 4122.   | 
**nf_instance_name** | **String** |  | [optional] [default to None]
**nf_type** | [***models::NfType**](NFType.md) |  | 
**nf_status** | [***models::NfStatus**](NFStatus.md) |  | 
**collocated_nf_instances** | [**Vec<models::CollocatedNfInstance>**](CollocatedNfInstance.md) |  | [optional] [default to None]
**plmn_list** | [**Vec<models::PlmnId>**](PlmnId.md) |  | [optional] [default to None]
**s_nssais** | [**Vec<models::ExtSnssai>**](ExtSnssai.md) |  | [optional] [default to None]
**per_plmn_snssai_list** | [**Vec<models::PlmnSnssai>**](PlmnSnssai.md) |  | [optional] [default to None]
**nsi_list** | **Vec<String>** |  | [optional] [default to None]
**fqdn** | **String** | Fully Qualified Domain Name | [optional] [default to None]
**inter_plmn_fqdn** | **String** | Fully Qualified Domain Name | [optional] [default to None]
**ipv4_addresses** | **Vec<models::Ipv4Addr>** |  | [optional] [default to None]
**ipv6_addresses** | [**Vec<models::Ipv6Addr>**](Ipv6Addr.md) |  | [optional] [default to None]
**capacity** | **u16** |  | [optional] [default to None]
**load** | **u8** |  | [optional] [default to None]
**load_time_stamp** | [**chrono::DateTime::<chrono::Utc>**](DateTime.md) | string with format 'date-time' as defined in OpenAPI. | [optional] [default to None]
**locality** | **String** |  | [optional] [default to None]
**priority** | **u16** |  | [optional] [default to None]
**udr_info** | [***models::UdrInfo**](UdrInfo.md) |  | [optional] [default to None]
**udr_info_list** | [**std::collections::HashMap<String, models::UdrInfo>**](UdrInfo.md) | A map (list of key-value pairs) where a (unique) valid JSON string serves as key of UdrInfo  | [optional] [default to None]
**udm_info** | [***models::UdmInfo**](UdmInfo.md) |  | [optional] [default to None]
**udm_info_list** | [**std::collections::HashMap<String, models::UdmInfo>**](UdmInfo.md) | A map (list of key-value pairs) where a (unique) valid JSON string serves as key of UdmInfo  | [optional] [default to None]
**ausf_info** | [***models::AusfInfo**](AusfInfo.md) |  | [optional] [default to None]
**ausf_info_list** | [**std::collections::HashMap<String, models::AusfInfo>**](AusfInfo.md) | A map (list of key-value pairs) where a (unique) valid JSON string serves as key of AusfInfo  | [optional] [default to None]
**amf_info** | [***models::AmfInfo**](AmfInfo.md) |  | [optional] [default to None]
**amf_info_list** | [**std::collections::HashMap<String, models::AmfInfo>**](AmfInfo.md) | A map (list of key-value pairs) where a (unique) valid JSON string serves as key of AmfInfo  | [optional] [default to None]
**smf_info** | [***models::SmfInfo**](SmfInfo.md) |  | [optional] [default to None]
**smf_info_list** | [**std::collections::HashMap<String, models::SmfInfo>**](SmfInfo.md) | A map (list of key-value pairs) where a (unique) valid JSON string serves as key of SmfInfo  | [optional] [default to None]
**upf_info** | [***models::UpfInfo**](UpfInfo.md) |  | [optional] [default to None]
**upf_info_list** | [**std::collections::HashMap<String, models::UpfInfo>**](UpfInfo.md) | A map (list of key-value pairs) where a (unique) valid JSON string serves as key of UpfInfo  | [optional] [default to None]
**pcf_info** | [***models::PcfInfo**](PcfInfo.md) |  | [optional] [default to None]
**pcf_info_list** | [**std::collections::HashMap<String, models::PcfInfo>**](PcfInfo.md) | A map (list of key-value pairs) where a (unique) valid JSON string serves as key of PcfInfo  | [optional] [default to None]
**bsf_info** | [***models::BsfInfo**](BsfInfo.md) |  | [optional] [default to None]
**bsf_info_list** | [**std::collections::HashMap<String, models::BsfInfo>**](BsfInfo.md) | A map (list of key-value pairs) where a (unique) valid JSON string serves as key of BsfInfo  | [optional] [default to None]
**chf_info** | [***models::ChfInfo**](ChfInfo.md) |  | [optional] [default to None]
**chf_info_list** | [**std::collections::HashMap<String, models::ChfInfo>**](ChfInfo.md) | A map (list of key-value pairs) where a (unique) valid JSON string serves as key of ChfInfo  | [optional] [default to None]
**udsf_info** | [***models::UdsfInfo**](UdsfInfo.md) |  | [optional] [default to None]
**udsf_info_list** | [**std::collections::HashMap<String, models::UdsfInfo>**](UdsfInfo.md) | A map (list of key-value pairs) where a (unique) valid JSON string serves as key of UdsfInfo  | [optional] [default to None]
**nwdaf_info** | [***models::NwdafInfo**](NwdafInfo.md) |  | [optional] [default to None]
**nwdaf_info_list** | [**std::collections::HashMap<String, models::NwdafInfo>**](NwdafInfo.md) | A map (list of key-value pairs) where a (unique) valid JSON string serves as key of NwdafInfo  | [optional] [default to None]
**nef_info** | [***models::NefInfo**](NefInfo.md) |  | [optional] [default to None]
**pcscf_info_list** | [**std::collections::HashMap<String, models::PcscfInfo>**](PcscfInfo.md) | A map (list of key-value pairs) where a (unique) valid JSON string serves as key of PcscfInfo  | [optional] [default to None]
**hss_info_list** | [**std::collections::HashMap<String, models::HssInfo>**](HssInfo.md) | A map (list of key-value pairs) where a (unique) valid JSON string serves as key of HssInfo  | [optional] [default to None]
**custom_info** | [***serde_json::Value**](.md) |  | [optional] [default to None]
**recovery_time** | [**chrono::DateTime::<chrono::Utc>**](DateTime.md) | string with format 'date-time' as defined in OpenAPI. | [optional] [default to None]
**nf_service_persistence** | **bool** |  | [optional] [default to Some(false)]
**nf_services** | [**Vec<models::NfService>**](NFService.md) |  | [optional] [default to None]
**nf_service_list** | [**std::collections::HashMap<String, models::NfService>**](NFService.md) | A map (list of key-value pairs) where serviceInstanceId serves as key of NFService  | [optional] [default to None]
**default_notification_subscriptions** | [**Vec<models::DefaultNotificationSubscription>**](DefaultNotificationSubscription.md) |  | [optional] [default to None]
**lmf_info** | [***models::LmfInfo**](LmfInfo.md) |  | [optional] [default to None]
**gmlc_info** | [***models::GmlcInfo**](GmlcInfo.md) |  | [optional] [default to None]
**snpn_list** | [**Vec<models::PlmnIdNid>**](PlmnIdNid.md) |  | [optional] [default to None]
**nf_set_id_list** | **Vec<models::NfSetId>** |  | [optional] [default to None]
**serving_scope** | **Vec<String>** |  | [optional] [default to None]
**lc_h_support_ind** | **bool** |  | [optional] [default to Some(false)]
**olc_h_support_ind** | **bool** |  | [optional] [default to Some(false)]
**nf_set_recovery_time_list** | [**std::collections::HashMap<String, models::DateTime>**](DateTime.md) | A map (list of key-value pairs) where NfSetId serves as key of DateTime | [optional] [default to None]
**service_set_recovery_time_list** | [**std::collections::HashMap<String, models::DateTime>**](DateTime.md) | A map (list of key-value pairs) where NfServiceSetId serves as key of DateTime  | [optional] [default to None]
**scp_domains** | **Vec<String>** |  | [optional] [default to None]
**scp_info** | [***models::ScpInfo**](ScpInfo.md) |  | [optional] [default to None]
**sepp_info** | [***models::SeppInfo**](SeppInfo.md) |  | [optional] [default to None]
**vendor_id** | **String** | Vendor ID of the NF Service instance (Private Enterprise Number assigned by IANA) | [optional] [default to None]
**supported_vendor_specific_features** | [**std::collections::HashMap<String, Vec<models::VendorSpecificFeature>>**](array.md) | The key of the map is the IANA-assigned SMI Network Management Private Enterprise Codes  | [optional] [default to None]
**aanf_info_list** | [**std::collections::HashMap<String, models::AanfInfo>**](AanfInfo.md) | A map (list of key-value pairs) where a (unique) valid JSON string serves as key of AanfInfo  | [optional] [default to None]
**mfaf_info** | [***models::MfafInfo**](MfafInfo.md) |  | [optional] [default to None]
**easdf_info_list** | [**std::collections::HashMap<String, models::EasdfInfo>**](EasdfInfo.md) | A map(list of key-value pairs) where a (unique) valid JSON string serves as key of EasdfInfo  | [optional] [default to None]
**dccf_info** | [***models::DccfInfo**](DccfInfo.md) |  | [optional] [default to None]
**nsacf_info_list** | [**std::collections::HashMap<String, models::NsacfInfo>**](NsacfInfo.md) | A map (list of key-value pairs) where a (unique) valid JSON string serves as key of NsacfInfo  | [optional] [default to None]
**mb_smf_info_list** | [**std::collections::HashMap<String, models::MbSmfInfo>**](MbSmfInfo.md) | A map (list of key-value pairs) where a (unique) valid JSON string serves as key of MbSmfInfo  | [optional] [default to None]
**tsctsf_info_list** | [**std::collections::HashMap<String, models::TsctsfInfo>**](TsctsfInfo.md) | A map (list of key-value pairs) where a (unique) valid JSON string serves as key of TsctsfInfo  | [optional] [default to None]
**mb_upf_info_list** | [**std::collections::HashMap<String, models::MbUpfInfo>**](MbUpfInfo.md) | A map (list of key-value pairs) where a (unique) valid JSON string serves as key of MbUpfInfo  | [optional] [default to None]
**trust_af_info** | [***models::TrustAfInfo**](TrustAfInfo.md) |  | [optional] [default to None]
**nssaaf_info** | [***models::NssaafInfo**](NssaafInfo.md) |  | [optional] [default to None]
**hni_list** | **Vec<models::Fqdn>** |  | [optional] [default to None]
**iwmsc_info** | [***models::IwmscInfo**](IwmscInfo.md) |  | [optional] [default to None]
**mnpf_info** | [***models::MnpfInfo**](MnpfInfo.md) |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


