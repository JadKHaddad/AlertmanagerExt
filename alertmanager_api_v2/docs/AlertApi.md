# \AlertApi

All URIs are relative to *http://localhost/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_alerts**](AlertApi.md#get_alerts) | **GET** /alerts | 
[**post_alerts**](AlertApi.md#post_alerts) | **POST** /alerts | 



## get_alerts

> Vec<crate::models::GettableAlert> get_alerts(active, silenced, inhibited, unprocessed, filter, receiver)


Get a list of alerts

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**active** | Option<**bool**> | Show active alerts |  |[default to true]
**silenced** | Option<**bool**> | Show silenced alerts |  |[default to true]
**inhibited** | Option<**bool**> | Show inhibited alerts |  |[default to true]
**unprocessed** | Option<**bool**> | Show unprocessed alerts |  |[default to true]
**filter** | Option<[**Vec<String>**](String.md)> | A list of matchers to filter alerts by |  |
**receiver** | Option<**String**> | A regex matching receivers to filter alerts by |  |

### Return type

[**Vec<crate::models::GettableAlert>**](gettableAlert.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_alerts

> post_alerts(alerts)


Create new Alerts

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**alerts** | [**Vec<crate::models::PostableAlert>**](postableAlert.md) | The alerts to create | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

