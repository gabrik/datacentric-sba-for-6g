# DnnConfiguration

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**pdu_session_types** | [***models::PduSessionTypes**](PduSessionTypes.md) |  | 
**ssc_modes** | [***models::SscModes**](SscModes.md) |  | 
**iwk_eps_ind** | **bool** |  | [optional] [default to None]
**param_5g_qos_profile** | [***models::SubscribedDefaultQos**](SubscribedDefaultQos.md) |  | [optional] [default to None]
**session_ambr** | [***models::Ambr**](Ambr.md) |  | [optional] [default to None]
**param_3gpp_charging_characteristics** | **String** |  | [optional] [default to None]
**static_ip_address** | [**Vec<models::IpAddress>**](IpAddress.md) |  | [optional] [default to None]
**up_security** | [***models::UpSecurity**](UpSecurity.md) |  | [optional] [default to None]
**pdu_session_continuity_ind** | [***models::PduSessionContinuityInd**](PduSessionContinuityInd.md) |  | [optional] [default to None]
**nidd_nef_id** | **String** | Identity of the NEF | [optional] [default to None]
**nidd_info** | [***models::NiddInformation**](NiddInformation.md) |  | [optional] [default to None]
**redundant_session_allowed** | **bool** |  | [optional] [default to None]
**acs_info** | [***models::AcsInfo**](AcsInfo.md) |  | [optional] [default to None]
**ipv4_frame_route_list** | [**Vec<models::FrameRouteInfo>**](FrameRouteInfo.md) |  | [optional] [default to None]
**ipv6_frame_route_list** | [**Vec<models::FrameRouteInfo>**](FrameRouteInfo.md) |  | [optional] [default to None]
**atsss_allowed** | **bool** |  | [optional] [default to Some(false)]
**secondary_auth** | **bool** |  | [optional] [default to None]
**uav_secondary_auth** | **bool** |  | [optional] [default to Some(false)]
**dn_aaa_ip_address_allocation** | **bool** |  | [optional] [default to None]
**dn_aaa_address** | [***models::IpAddress**](IpAddress.md) |  | [optional] [default to None]
**additional_dn_aaa_addresses** | [**Vec<models::IpAddress>**](IpAddress.md) |  | [optional] [default to None]
**dn_aaa_fqdn** | **String** | Fully Qualified Domain Name | [optional] [default to None]
**iptv_acc_ctrl_info** | **String** |  | [optional] [default to None]
**ipv4_index** | [***models::IpIndex**](IpIndex.md) |  | [optional] [default to None]
**ipv6_index** | [***models::IpIndex**](IpIndex.md) |  | [optional] [default to None]
**ecs_addr_config_info** | [***models::EcsAddrConfigInfo**](EcsAddrConfigInfo.md) |  | [optional] [default to None]
**additional_ecs_addr_config_infos** | [**Vec<models::EcsAddrConfigInfo>**](EcsAddrConfigInfo.md) |  | [optional] [default to None]
**shared_ecs_addr_config_info** | **String** |  | [optional] [default to None]
**additional_shared_ecs_addr_config_info_ids** | **Vec<models::SharedDataId>** |  | [optional] [default to None]
**eas_discovery_authorized** | **bool** |  | [optional] [default to Some(false)]
**onboarding_ind** | **bool** |  | [optional] [default to Some(false)]
**aerial_ue_ind** | [***models::AerialUeIndication**](AerialUeIndication.md) |  | [optional] [default to None]
**subscribed_max_ipv6_prefix_size** | **i32** |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


