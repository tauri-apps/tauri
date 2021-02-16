import { invoke } from './tauri'

export enum ResponseType {
  JSON = 1,
  Text = 2,
  Binary = 3
}

export enum BodyType {
  Form = 1,
  File = 2,
  Auto = 3
}

export type Body = object | string | BinaryType

export type HttpVerb =
  | 'GET'
  | 'POST'
  | 'PUT'
  | 'DELETE'
  | 'PATCH'
  | 'HEAD'
  | 'OPTIONS'
  | 'CONNECT'
  | 'TRACE'

export interface HttpOptions {
  method: HttpVerb
  url: string
  headers?: Record<string, any>
  params?: Record<string, any>
  body?: Body
  followRedirects: boolean
  maxRedirections: boolean
  connectTimeout: number
  readTimeout: number
  timeout: number
  allowCompression: boolean
  responseType?: ResponseType
  bodyType: BodyType
}

export type PartialOptions = Omit<HttpOptions, 'method' | 'url'>

/**
 * makes a HTTP request
 *
 * @param options request options
 *
 * @return promise resolving to the response
 */
async function request<T>(options: HttpOptions): Promise<T> {
  return await invoke<void>({
    __tauriModule: 'Http',
    message: {
      cmd: 'httpRequest',
      options: options
    }
  })
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
async function post<T>(
  url: string,
  body: Body,
  options: PartialOptions
): Promise<T> {
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
async function put<T>(
  url: string,
  body: Body,
  options: PartialOptions
): Promise<T> {
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
async function deleteRequest<T>(
  url: string,
  options: PartialOptions
): Promise<T> {
  return await request({
    method: 'DELETE',
    url,
    ...options
  })
}

export { request, get, post, put, patch, deleteRequest as httpDelete }
