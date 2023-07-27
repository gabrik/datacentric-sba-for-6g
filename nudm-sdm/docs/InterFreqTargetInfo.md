# InterFreqTargetInfo

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**dl_carrier_freq** | **u32** | Integer value indicating the ARFCN applicable for a downlink, uplink or bi-directional (TDD) NR global frequency raster, as definition of \"ARFCN-ValueNR\" IE in clause 6.3.2 of 3GPP TS 38.331.  | 
**cell_id_list** | **Vec<models::PhysCellId>** | When present, this IE shall contain a list of the physical cell identities where the UE is requested to perform measurement logging for the indicated frequency.  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


