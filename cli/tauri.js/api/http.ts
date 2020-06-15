import tauri from './tauri'
import { HttpOptions, Body, BodyType, ResponseType } from './models'

/**
 * makes a HTTP request
 *
 * @param options request options
 *
 * @return promise resolving to the response
 */
function request<T> (options: HttpOptions): Promise<T> {
  return tauri.httpRequest(options)
}

/**
 * makes a GET request
 *
 * @param url request URL
 * @param options request options
 *
 * @return promise resolving to the response
 */
function get<T> (url: string, options: HttpOptions): Promise<T> {
  return request({
    method: 'GET',
    url,
    ...options
  })
}

/**
 * makes a POST request
 *
 * @param url request URL
 * @param body request body
 * @param options request options
 *
 * @return promise resolving to the response
 */
function post<T> (url: string, body: Body, options: HttpOptions): Promise<T> {
  return request({
    method: 'POST',
    url,
    body,
    ...options
  })
}

/**
 * makes a PUT request
 *
 * @param url request URL
 * @param body request body
 * @param options request options
 *
 * @return promise resolving to the response
 */
function put<T> (url: string, body: Body, options: HttpOptions): Promise<T> {
  return request({
    method: 'PUT',
    url,
    body,
    ...options
  })
}

/**
 * makes a PATCH request
 *
 * @param url request URL
 * @param options request options
 *
 * @return promise resolving to the response
 */
function patch<T> (url: string, options: HttpOptions): Promise<T> {
  return request({
    method: 'PATCH',
    url,
    ...options
  })
}

/**
 * makes a DELETE request
 *
 * @param url request URL
 * @param options request options
 *
 * @return promise resolving to the response
 */
function deleteRequest<T> (url: string, options: HttpOptions): Promise<T> {
  return request({
    method: 'DELETE',
    url,
    ...options
  })
}

export default {
  request,
  get,
  post,
  put,
  patch,
  delete: deleteRequest,
  ResponseType,
  BodyType
}
