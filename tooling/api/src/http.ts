// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Access the HTTP client written in Rust.
 *
 * This package is also accessible with `window.__TAURI__.http` when `tauri.conf.json > build > withGlobalTauri` is set to true.
 *
 * The APIs must be allowlisted on `tauri.conf.json`:
 * ```json
 * {
 *   "tauri": {
 *     "allowlist": {
 *       "http": {
 *         "all": true, // enable all http APIs
 *         "request": true // enable HTTP request API
 *       }
 *     }
 *   }
 * }
 * ```
 * It is recommended to allowlist only the APIs you use for optimal bundle size and security.
 *
 * ## Security
 *
 * This API has a scope configuration that forces you to restrict the URLs and paths that can be accessed using glob patterns.
 *
 * For instance, this scope configuration only allows making HTTP requests to the GitHub API for the `tauri-apps` organization:
 * ```json
 * {
 *   "tauri": {
 *     "allowlist": {
 *       "http": {
 *         "scope": ["https://api.github.com/repos/tauri-apps/*"]
 *       }
 *     }
 *   }
 * }
 * ```
 * Trying to execute any API with a URL not configured on the scope results in a promise rejection due to denied access.
 *
 * @module
 */

import { invokeTauriCommand } from './helpers/tauri'

interface Duration {
  secs: number
  nanos: number
}

interface ClientOptions {
  maxRedirections?: number
  connectTimeout?: number | Duration
}

enum ResponseType {
  JSON = 1,
  Text = 2,
  Binary = 3
}

interface FilePart<T> {
  value: string | T
  mime?: string
  fileName?: string
}

type Part = string | Uint8Array | FilePart<Uint8Array>

/** The body object to be used on POST and PUT requests. */
class Body {
  type: string
  payload: unknown

  /** @ignore */
  private constructor(type: string, payload: unknown) {
    this.type = type
    this.payload = payload
  }

  /**
   * Creates a new form data body. The form data is an object where each key is the entry name,
   * and the value is either a string or a file object.
   *
   * By default it sets the `application/x-www-form-urlencoded` Content-Type header,
   * but you can set it to `multipart/form-data` if the Cargo feature `http-multipart` is enabled.
   *
   * Note that a file path must be allowed in the `fs` allowlist scope.
   *
   * # Examples
   *
   * ```js
   * import { Body } from "@tauri-apps/api/http"
   * Body.form({
   *   key: 'value',
   *   image: {
   *     file: '/path/to/file', // either a path of an array buffer of the file contents
   *     mime: 'image/jpeg', // optional
   *     fileName: 'image.jpg' // optional
   *   }
   * })
   * ```
   *
   * @param data The body data.
   *
   * @return The body object ready to be used on the POST and PUT requests.
   */
  static form(data: Record<string, Part>): Body {
    const form: Record<string, string | number[] | FilePart<number[]>> = {}
    for (const key in data) {
      // eslint-disable-next-line security/detect-object-injection
      const v = data[key]
      let r
      if (typeof v === 'string') {
        r = v
      } else if (v instanceof Uint8Array || Array.isArray(v)) {
        r = Array.from(v)
      } else if (typeof v.value === 'string') {
        r = { value: v.value, mime: v.mime, fileName: v.fileName }
      } else {
        r = { value: Array.from(v.value), mime: v.mime, fileName: v.fileName }
      }
      // eslint-disable-next-line security/detect-object-injection
      form[key] = r
    }
    return new Body('Form', form)
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
  static bytes(bytes: Uint8Array): Body {
    // stringifying Uint8Array doesn't return an array of numbers, so we create one here
    return new Body('Bytes', Array.from(bytes))
  }
}

/** The request HTTP verb. */
type HttpVerb =
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
interface HttpOptions {
  method: HttpVerb
  url: string
  headers?: Record<string, any>
  query?: Record<string, any>
  body?: Body
  timeout?: number | Duration
  responseType?: ResponseType
}

/** Request options. */
type RequestOptions = Omit<HttpOptions, 'method' | 'url'>
/** Options for the `fetch` API. */
type FetchOptions = Omit<HttpOptions, 'url'>

/** @ignore */
interface IResponse<T> {
  url: string
  status: number
  headers: Record<string, string>
  rawHeaders: Record<string, string[]>
  data: T
}

/** Response object. */
class Response<T> {
  /** The request URL. */
  url: string
  /** The response status code. */
  status: number
  /** A boolean indicating whether the response was successful (status in the range 200–299) or not. */
  ok: boolean
  /** The response headers. */
  headers: Record<string, string>
  /** The response raw headers. */
  rawHeaders: Record<string, string[]>
  /** The response data. */
  data: T

  /** @ignore */
  constructor(response: IResponse<T>) {
    this.url = response.url
    this.status = response.status
    this.ok = this.status >= 200 && this.status < 300
    this.headers = response.headers
    this.rawHeaders = response.rawHeaders
    this.data = response.data
  }
}

class Client {
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
    const jsonResponse =
      !options.responseType || options.responseType === ResponseType.JSON
    if (jsonResponse) {
      options.responseType = ResponseType.Text
    }
    return invokeTauriCommand<IResponse<T>>({
      __tauriModule: 'Http',
      message: {
        cmd: 'httpRequest',
        client: this.id,
        options
      }
    }).then((res) => {
      const response = new Response(res)
      if (jsonResponse) {
        /* eslint-disable */
        try {
          // @ts-expect-error
          response.data = JSON.parse(response.data as string)
        } catch (e) {
          if (response.ok && (response.data as unknown as string) === '') {
            // @ts-expect-error
            response.data = {}
          } else if (response.ok) {
            throw Error(
              `Failed to parse response \`${response.data}\` as JSON: ${e};
              try setting the \`responseType\` option to \`ResponseType.Text\` or \`ResponseType.Binary\` if the API does not return a JSON response.`
            )
          }
        }
        /* eslint-enable */
        return response
      }
      return response
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

/** @internal */
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

export type {
  Duration,
  ClientOptions,
  Part,
  HttpVerb,
  HttpOptions,
  RequestOptions,
  FetchOptions
}

export { getClient, fetch, Body, Client, Response, ResponseType, FilePart }
