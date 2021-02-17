import { invoke } from './tauri'

export interface ClientOptions {
  maxRedirections: boolean
  connectTimeout: number
}

export enum ResponseType {
  JSON = 1,
  Text = 2,
  Binary = 3
}

export type Part = 'string' | number[]

export class Body {
  type: string
  payload: unknown

  constructor(type: string, payload: unknown) {
    this.type = type
    this.payload = payload
  }

  static form(data: Record<string, Part>): Body {
    return new Body('Form', data)
  }

  static json(data: Record<any, any>): Body {
    return new Body('Json', data)
  }

  static text(value: string): Body {
    return new Body('Text', value)
  }

  static bytes(bytes: number[]): Body {
    return new Body('Bytes', bytes)
  }
}

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
  query?: Record<string, any>
  body?: Body
  timeout?: number
  responseType?: ResponseType
}

export type RequestOptions = Omit<HttpOptions, 'method' | 'url'>
export type FetchOptions = Omit<HttpOptions, 'url'>

export interface Response<T> {
  url: string
  status: number
  headers: Record<string, string>
  data: T
}

export class Client {
  id: number
  constructor(id: number) {
    this.id = id
  }

  /**
   * drops the client instance
   */
  async drop(): Promise<void> {
    return invoke({
      module: 'Http',
      message: {
        cmd: 'dropClient',
        client: this.id
      }
    })
  }

  /**
   * makes a HTTP request
   *
   * @param options request options
   *
   * @return promise resolving to the response
   */
  async request<T>(options: HttpOptions): Promise<Response<T>> {
    return invoke({
      module: 'Http',
      message: {
        cmd: 'httpRequest',
        client: this.id,
        options
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
  async get<T>(url: string, options: RequestOptions): Promise<Response<T>> {
    return this.request({
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
  async post<T>(
    url: string,
    body: Body,
    options: RequestOptions
  ): Promise<Response<T>> {
    return this.request({
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
  async put<T>(
    url: string,
    body: Body,
    options: RequestOptions
  ): Promise<Response<T>> {
    return this.request({
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
  async patch<T>(url: string, options: RequestOptions): Promise<Response<T>> {
    return this.request({
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
  async delete<T>(url: string, options: RequestOptions): Promise<Response<T>> {
    return this.request({
      method: 'DELETE',
      url,
      ...options
    })
  }
}

async function getClient(options?: ClientOptions): Promise<Client> {
  return invoke<number>({
    module: 'Http',
    message: {
      cmd: 'createClient',
      options
    }
  }).then((id) => new Client(id))
}

let defaultClient: Client | null = null

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
