# \AlertgroupApi

All URIs are relative to *http://localhost/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_alert_groups**](AlertgroupApi.md#get_alert_groups) | **GET** /alerts/groups | 



## get_alert_groups

> Vec<crate::models::AlertGroup> get_alert_groups(active, silenced, inhibited, filter, receiver)


Get a list of alert groups

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**active** | Option<**bool**> | Show active alerts |  |[default to true]
**silenced** | Option<**bool**> | Show silenced alerts |  |[default to true]
**inhibited** | Option<**bool**> | Show inhibited alerts |  |[default to true]
**filter** | Option<[**Vec<String>**](String.md)> | A list of matchers to filter alerts by |  |
**receiver** | Option<**String**> | A regex matching receivers to filter alerts by |  |

### Return type

[**Vec<crate::models::AlertGroup>**](alertGroup.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

