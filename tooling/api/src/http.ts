// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Access the HTTP client written in Rust.
 * @packageDocumentation
 */

import { invokeTauriCommand } from './helpers/tauri'

export interface ClientOptions {
  maxRedirections: number
  connectTimeout: number
}

export enum ResponseType {
  JSON = 1,
  Text = 2,
  Binary = 3
}

export type Part = 'string' | number[]

/** The body object to be used on POST and PUT requests. */
export class Body {
  type: string
  payload: unknown

  /** @ignore */
  constructor(type: string, payload: unknown) {
    this.type = type
    this.payload = payload
  }

  /**
   * Creates a new form data body.
   *
   * @param data The body data.
   *
   * @return The body object ready to be used on the POST and PUT requests.
   */
  static form(data: Record<string, Part>): Body {
    return new Body('Form', data)
  }

  /**
   * Creates a new JSON body.
   *
   * @param data The body JSON object.
   *
   * @return The body object ready to be used on the POST and PUT requests.
   */
  static json(data: Record<any, any>): Body {
    return new Body('Json', data)
  }

  /**
   * Creates a new UTF-8 string body.
   *
   * @param data The body string.
   *
   * @return The body object ready to be used on the POST and PUT requests.
   */
  static text(value: string): Body {
    return new Body('Text', value)
  }

  /**
   * Creates a new byte array body.
   *
   * @param data The body byte array.
   *
   * @return The body object ready to be used on the POST and PUT requests.
   */
  static bytes(bytes: number[]): Body {
    return new Body('Bytes', bytes)
  }
}

/** The request HTTP verb. */
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

/** Options object sent to the backend. */
export interface HttpOptions {
  method: HttpVerb
  url: string
  headers?: Record<string, any>
  query?: Record<string, any>
  body?: Body
  timeout?: number
  responseType?: ResponseType
}

/** Request options. */
export type RequestOptions = Omit<HttpOptions, 'method' | 'url'>
/** Options for the `fetch` API. */
export type FetchOptions = Omit<HttpOptions, 'url'>

/** Response object. */
export interface Response<T> {
  /** The request URL. */
  url: string
  /** The response status code. */
  status: number
  /** The response headers. */
  headers: Record<string, string>
  /** The response data. */
  data: T
}

export class Client {
  id: number
  /** @ignore */
  constructor(id: number) {
    this.id = id
  }

  /**
   * Drops the client instance.
   *
   * @returns
   */
  async drop(): Promise<void> {
    return invokeTauriCommand({
      __tauriModule: 'Http',
      message: {
        cmd: 'dropClient',
        client: this.id
      }
    })
  }

  /**
   * Makes an HTTP request.
   *
   * @param options The request options.
   * @returns A promise resolving to the response.
   */
  async request<T>(options: HttpOptions): Promise<Response<T>> {
    return invokeTauriCommand({
      __tauriModule: 'Http',
      message: {
        cmd: 'httpRequest',
        client: this.id,
        options
      }
    })
  }

  /**
   * Makes a GET request.
   *
   * @param url The request URL.
   * @param options The request options.
   * @returns A promise resolving to the response.
   */
  async get<T>(url: string, options?: RequestOptions): Promise<Response<T>> {
    return this.request({
      method: 'GET',
      url,
      ...options
    })
  }

  /**
   * Makes a POST request.
   *
   * @param url The request URL.
   * @param body The body of the request.
   * @param options The request options.
   * @returns A promise resolving to the response.
   */
  async post<T>(
    url: string,
    body?: Body,
    options?: RequestOptions
  ): Promise<Response<T>> {
    return this.request({
      method: 'POST',
      url,
      body,
      ...options
    })
  }

  /**
   * Makes a PUT request.
   *
   * @param url The request URL.
   * @param body The body of the request.
   * @param options Request options.
   * @returns A promise resolving to the response.
   */
  async put<T>(
    url: string,
    body?: Body,
    options?: RequestOptions
  ): Promise<Response<T>> {
    return this.request({
      method: 'PUT',
      url,
      body,
      ...options
    })
  }

  /**
   * Makes a PATCH request.
   *
   * @param url The request URL.
   * @param options The request options.
   * @returns A promise resolving to the response.
   */
  async patch<T>(url: string, options?: RequestOptions): Promise<Response<T>> {
    return this.request({
      method: 'PATCH',
      url,
      ...options
    })
  }

  /**
   * Makes a DELETE request.
   *
   * @param url The request URL.
   * @param options The request options.
   * @returns A promise resolving to the response.
   */
  async delete<T>(url: string, options?: RequestOptions): Promise<Response<T>> {
    return this.request({
      method: 'DELETE',
      url,
      ...options
    })
  }
}

/**
 * Creates a new client using the specified options.
 *
 * @param options Client configuration.
 *
 * @return A promise resolving to the client instance.
 */
async function getClient(options?: ClientOptions): Promise<Client> {
  return invokeTauriCommand<number>({
    __tauriModule: 'Http',
    message: {
      cmd: 'createClient',
      options
    }
  }).then((id) => new Client(id))
}

/** @ignore */
let defaultClient: Client | null = null

/**
 * Perform an HTTP request using the default client.
 *
 * @param url The request URL.
 * @param options The fetch options.
 * @return The response object.
 */
async function fetch<T>(
  url: string,
  options?: FetchOptions
): Promise<Response<T>> {
  if (defaultClient === null) {
    defaultClient = await getClient()
  }
  return defaultClient.request({
    url,
    method: options?.method ?? 'GET',
    ...options
  })
}

export { getClient, fetch }
