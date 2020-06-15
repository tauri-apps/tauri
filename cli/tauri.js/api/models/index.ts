// http

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

export type HttpVerb = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'HEAD' | 'OPTIONS' | 'CONNECT' | 'TRACE'

/**
 * @typedef {Object} HttpOptions
 * @property method GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS, CONNECT or TRACE
 * @property url the request URL
 * @property [headers] the request headers
 * @property [propertys] the request query propertys
 * @property [body] the request body
 * @property followRedirects whether to follow redirects or not
 * @property maxRedirections max number of redirections
 * @property connectTimeout request connect timeout
 * @property readTimeout request read timeout
 * @property timeout request timeout
 * @property allowCompression
 * @property [responseType=1] response type
 * @property [bodyType=3] body type
*/
export interface HttpOptions {
  method: HttpVerb
  url: string
  headers?: Record<string, any>
  propertys?: Record<string, any>
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

// events

export interface Event {
  type: string
  payload: unknown
}

export type EventCallback = (event: Event) => void
