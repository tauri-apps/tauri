// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Access the HTTP client written in Rust.
 *
 * This package is also accessible with `window.__TAURI__.http` when [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri) in `tauri.conf.json` is set to `true`.
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

/**
 * @since 1.0.0
 */
interface Duration {
  secs: number
  nanos: number
}

/**
 * @since 1.0.0
 */
interface ClientOptions {
  maxRedirections?: number
  /**
   * Defines the maximum number of redirects the client should follow.
   * If set to 0, no redirects will be followed.
   */
  connectTimeout?: number | Duration
}

/**
 * @since 1.0.0
 */
enum ResponseType {
  JSON = 1,
  Text = 2,
  Binary = 3
}

/**
 * @since 1.0.0
 */
interface FilePart<T> {
  file: string | T
  mime?: string
  fileName?: string
}

type Part = string | Uint8Array | FilePart<Uint8Array>

/**
 * The body object to be used on POST and PUT requests.
 *
 * @since 1.0.0
 */
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
   * @example
   * ```typescript
   * import { Body } from "@tauri-apps/api/http"
   * const body = Body.form({
   *   key: 'value',
   *   image: {
   *     file: '/path/to/file', // either a path or an array buffer of the file contents
   *     mime: 'image/jpeg', // optional
   *     fileName: 'image.jpg' // optional
   *   }
   * });
   *
   * // alternatively, use a FormData:
   * const form = new FormData();
   * form.append('key', 'value');
   * form.append('image', {
   *   file: '/path/to/file',
   *   mime: 'image/jpeg',
   *   fileName: 'image.jpg'
   * });
   * const formBody = Body.form(form);
   * ```
   *
   * @param data The body data.
   *
   * @returns The body object ready to be used on the POST and PUT requests.
   */
  static form(data: Record<string, Part> | FormData): Body {
    const form: Record<string, string | number[] | FilePart<number[]>> = {}

    const append = (
      key: string,
      v: string | Uint8Array | FilePart<Uint8Array> | File
    ): void => {
      if (v !== null) {
        let r
        if (typeof v === 'string') {
          r = v
        } else if (v instanceof Uint8Array || Array.isArray(v)) {
          r = Array.from(v)
        } else if (v instanceof File) {
          r = { file: v.name, mime: v.type, fileName: v.name }
        } else if (typeof v.file === 'string') {
          r = { file: v.file, mime: v.mime, fileName: v.fileName }
        } else {
          r = { file: Array.from(v.file), mime: v.mime, fileName: v.fileName }
        }
        form[String(key)] = r
      }
    }

    if (data instanceof FormData) {
      for (const [key, value] of data) {
        append(key, value)
      }
    } else {
      for (const [key, value] of Object.entries(data)) {
        append(key, value)
      }
    }
    return new Body('Form', form)
  }

  /**
   * Creates a new JSON body.
   * @example
   * ```typescript
   * import { Body } from "@tauri-apps/api/http"
   * Body.json({
   *   registered: true,
   *   name: 'tauri'
   * });
   * ```
   *
   * @param data The body JSON object.
   *
   * @returns The body object ready to be used on the POST and PUT requests.
   */
  static json(data: Record<any, any>): Body {
    return new Body('Json', data)
  }

  /**
   * Creates a new UTF-8 string body.
   * @example
   * ```typescript
   * import { Body } from "@tauri-apps/api/http"
   * Body.text('The body content as a string');
   * ```
   *
   * @param value The body string.
   *
   * @returns The body object ready to be used on the POST and PUT requests.
   */
  static text(value: string): Body {
    return new Body('Text', value)
  }

  /**
   * Creates a new byte array body.
   * @example
   * ```typescript
   * import { Body } from "@tauri-apps/api/http"
   * Body.bytes(new Uint8Array([1, 2, 3]));
   * ```
   *
   * @param bytes The body byte array.
   *
   * @returns The body object ready to be used on the POST and PUT requests.
   */
  static bytes(
    bytes: Iterable<number> | ArrayLike<number> | ArrayBuffer
  ): Body {
    // stringifying Uint8Array doesn't return an array of numbers, so we create one here
    return new Body(
      'Bytes',
      Array.from(bytes instanceof ArrayBuffer ? new Uint8Array(bytes) : bytes)
    )
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

/**
 * Options object sent to the backend.
 *
 * @since 1.0.0
 */
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

/**
 * Response object.
 *
 * @since 1.0.0
 * */
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

/**
 * @since 1.0.0
 */
class Client {
  id: number
  /** @ignore */
  constructor(id: number) {
    this.id = id
  }

  /**
   * Drops the client instance.
   * @example
   * ```typescript
   * import { getClient } from '@tauri-apps/api/http';
   * const client = await getClient();
   * await client.drop();
   * ```
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
   * @example
   * ```typescript
   * import { getClient } from '@tauri-apps/api/http';
   * const client = await getClient();
   * const response = await client.request({
   *   method: 'GET',
   *   url: 'http://localhost:3003/users',
   * });
   * ```
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
   * @example
   * ```typescript
   * import { getClient, ResponseType } from '@tauri-apps/api/http';
   * const client = await getClient();
   * const response = await client.get('http://localhost:3003/users', {
   *   timeout: 30,
   *   // the expected response type
   *   responseType: ResponseType.JSON
   * });
   * ```
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
   * @example
   * ```typescript
   * import { getClient, Body, ResponseType } from '@tauri-apps/api/http';
   * const client = await getClient();
   * const response = await client.post('http://localhost:3003/users', {
   *   body: Body.json({
   *     name: 'tauri',
   *     password: 'awesome'
   *   }),
   *   // in this case the server returns a simple string
   *   responseType: ResponseType.Text,
   * });
   * ```
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
   * @example
   * ```typescript
   * import { getClient, Body } from '@tauri-apps/api/http';
   * const client = await getClient();
   * const response = await client.put('http://localhost:3003/users/1', {
   *   body: Body.form({
   *     file: {
   *       file: '/home/tauri/avatar.png',
   *       mime: 'image/png',
   *       fileName: 'avatar.png'
   *     }
   *   })
   * });
   * ```
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
   * @example
   * ```typescript
   * import { getClient, Body } from '@tauri-apps/api/http';
   * const client = await getClient();
   * const response = await client.patch('http://localhost:3003/users/1', {
   *   body: Body.json({ email: 'contact@tauri.app' })
   * });
   * ```
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
   * @example
   * ```typescript
   * import { getClient } from '@tauri-apps/api/http';
   * const client = await getClient();
   * const response = await client.delete('http://localhost:3003/users/1');
   * ```
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
 * @example
 * ```typescript
 * import { getClient } from '@tauri-apps/api/http';
 * const client = await getClient();
 * ```
 *
 * @param options Client configuration.
 *
 * @returns A promise resolving to the client instance.
 *
 * @since 1.0.0
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
 * @example
 * ```typescript
 * import { fetch } from '@tauri-apps/api/http';
 * const response = await fetch('http://localhost:3003/users/2', {
 *   method: 'GET',
 *   timeout: 30,
 * });
 * ```
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
