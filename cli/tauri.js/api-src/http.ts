import tauri from './tauri'
import { HttpOptions, Body, BodyType, ResponseType, PartialOptions } from './types/http'

/**
 * makes a HTTP request
 *
 * @param options request options
 *
 * @return promise resolving to the response
 */
async function request<T>(options: HttpOptions): Promise<T> {
  return await tauri.httpRequest<T>(options)
}

/**
 * makes a GET request
 *
 * @param url request URL
 * @param options request options
 *
 * @return promise resolving to the response
 */
async function get<T>(url: string, options: PartialOptions): Promise<T> {
  return await request({
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
async function post<T>(url: string, body: Body, options: PartialOptions): Promise<T> {
  return await request({
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
async function put<T>(url: string, body: Body, options: PartialOptions): Promise<T> {
  return await request({
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
async function patch<T>(url: string, options: PartialOptions): Promise<T> {
  return await request({
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
async function deleteRequest<T>(url: string, options: PartialOptions): Promise<T> {
  return await request({
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
