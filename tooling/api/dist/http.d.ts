interface ClientOptions {
    maxRedirections: number;
    connectTimeout: number;
}
declare enum ResponseType {
    JSON = 1,
    Text = 2,
    Binary = 3
}
declare type Part = string | number[];
/** The body object to be used on POST and PUT requests. */
declare class Body {
    type: string;
    payload: unknown;
    /** @ignore */
    private constructor();
    /**
     * Creates a new form data body.
     *
     * @param data The body data.
     *
     * @return The body object ready to be used on the POST and PUT requests.
     */
    static form(data: Record<string, Part>): Body;
    /**
     * Creates a new JSON body.
     *
     * @param data The body JSON object.
     *
     * @return The body object ready to be used on the POST and PUT requests.
     */
    static json(data: Record<any, any>): Body;
    /**
     * Creates a new UTF-8 string body.
     *
     * @param data The body string.
     *
     * @return The body object ready to be used on the POST and PUT requests.
     */
    static text(value: string): Body;
    /**
     * Creates a new byte array body.
     *
     * @param data The body byte array.
     *
     * @return The body object ready to be used on the POST and PUT requests.
     */
    static bytes(bytes: number[]): Body;
}
/** The request HTTP verb. */
declare type HttpVerb = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'HEAD' | 'OPTIONS' | 'CONNECT' | 'TRACE';
/** Options object sent to the backend. */
interface HttpOptions {
    method: HttpVerb;
    url: string;
    headers?: Record<string, any>;
    query?: Record<string, any>;
    body?: Body;
    timeout?: number;
    responseType?: ResponseType;
}
/** Request options. */
declare type RequestOptions = Omit<HttpOptions, 'method' | 'url'>;
/** Options for the `fetch` API. */
declare type FetchOptions = Omit<HttpOptions, 'url'>;
/** @ignore */
interface IResponse<T> {
    url: string;
    status: number;
    headers: Record<string, string>;
    rawHeaders: Record<string, string[]>;
    data: T;
}
/** Response object. */
declare class Response<T> {
    /** The request URL. */
    url: string;
    /** The response status code. */
    status: number;
    /** A boolean indicating whether the response was successful (status in the range 200–299) or not. */
    ok: boolean;
    /** The response headers. */
    headers: Record<string, string>;
    /** The response raw headers. */
    rawHeaders: Record<string, string[]>;
    /** The response data. */
    data: T;
    /** @ignore */
    constructor(response: IResponse<T>);
}
declare class Client {
    id: number;
    /** @ignore */
    constructor(id: number);
    /**
     * Drops the client instance.
     *
     * @returns
     */
    drop(): Promise<void>;
    /**
     * Makes an HTTP request.
     *
     * @param options The request options.
     * @returns A promise resolving to the response.
     */
    request<T>(options: HttpOptions): Promise<Response<T>>;
    /**
     * Makes a GET request.
     *
     * @param url The request URL.
     * @param options The request options.
     * @returns A promise resolving to the response.
     */
    get<T>(url: string, options?: RequestOptions): Promise<Response<T>>;
    /**
     * Makes a POST request.
     *
     * @param url The request URL.
     * @param body The body of the request.
     * @param options The request options.
     * @returns A promise resolving to the response.
     */
    post<T>(url: string, body?: Body, options?: RequestOptions): Promise<Response<T>>;
    /**
     * Makes a PUT request.
     *
     * @param url The request URL.
     * @param body The body of the request.
     * @param options Request options.
     * @returns A promise resolving to the response.
     */
    put<T>(url: string, body?: Body, options?: RequestOptions): Promise<Response<T>>;
    /**
     * Makes a PATCH request.
     *
     * @param url The request URL.
     * @param options The request options.
     * @returns A promise resolving to the response.
     */
    patch<T>(url: string, options?: RequestOptions): Promise<Response<T>>;
    /**
     * Makes a DELETE request.
     *
     * @param url The request URL.
     * @param options The request options.
     * @returns A promise resolving to the response.
     */
    delete<T>(url: string, options?: RequestOptions): Promise<Response<T>>;
}
/**
 * Creates a new client using the specified options.
 *
 * @param options Client configuration.
 *
 * @return A promise resolving to the client instance.
 */
declare function getClient(options?: ClientOptions): Promise<Client>;
/**
 * Perform an HTTP request using the default client.
 *
 * @param url The request URL.
 * @param options The fetch options.
 * @return The response object.
 */
declare function fetch<T>(url: string, options?: FetchOptions): Promise<Response<T>>;
export type { ClientOptions, Part, HttpVerb, HttpOptions, RequestOptions, FetchOptions };
export { getClient, fetch, Body, Client, Response, ResponseType };
