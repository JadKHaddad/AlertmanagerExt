# \SilenceApi

All URIs are relative to *http://localhost/api/v2*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_silence**](SilenceApi.md#delete_silence) | **DELETE** /silence/{silenceID} | 
[**get_silence**](SilenceApi.md#get_silence) | **GET** /silence/{silenceID} | 
[**get_silences**](SilenceApi.md#get_silences) | **GET** /silences | 
[**post_silences**](SilenceApi.md#post_silences) | **POST** /silences | 



## delete_silence

> delete_silence(silence_id)


Delete a silence by its ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**silence_id** | **uuid::Uuid** | ID of the silence to get | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_silence

> crate::models::GettableSilence get_silence(silence_id)


Get a silence by its ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**silence_id** | **uuid::Uuid** | ID of the silence to get | [required] |

### Return type

[**crate::models::GettableSilence**](gettableSilence.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_silences

> Vec<crate::models::GettableSilence> get_silences(filter)


Get a list of silences

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**filter** | Option<[**Vec<String>**](String.md)> | A list of matchers to filter silences by |  |

### Return type

[**Vec<crate::models::GettableSilence>**](gettableSilence.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## post_silences

> crate::models::PostSilences200Response post_silences(silence)


Post a new silence or update an existing one

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**silence** | [**PostableSilence**](PostableSilence.md) | The silence to create | [required] |

### Return type

[**crate::models::PostSilences200Response**](postSilences_200_response.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

