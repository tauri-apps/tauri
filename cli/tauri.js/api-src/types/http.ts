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
