export declare enum ResponseType {
    JSON = 1,
    Text = 2,
    Binary = 3
}
export declare enum BodyType {
    Form = 1,
    File = 2,
    Auto = 3
}
export declare type Body = object | string | BinaryType;
export declare type HttpVerb = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH' | 'HEAD' | 'OPTIONS' | 'CONNECT' | 'TRACE';
export interface HttpOptions {
    method: HttpVerb;
    url: string;
    headers?: Record<string, any>;
    params?: Record<string, any>;
    body?: Body;
    followRedirects: boolean;
    maxRedirections: boolean;
    connectTimeout: number;
    readTimeout: number;
    timeout: number;
    allowCompression: boolean;
    responseType?: ResponseType;
    bodyType: BodyType;
}
export declare type PartialOptions = Omit<HttpOptions, 'method' | 'url'>;
/**
 * makes a HTTP request
 *
 * @param options request options
 *
 * @return promise resolving to the response
 */
declare function request<T>(options: HttpOptions): Promise<T>;
/**
 * makes a GET request
 *
 * @param url request URL
 * @param options request options
 *
 * @return promise resolving to the response
 */
declare function get<T>(url: string, options: PartialOptions): Promise<T>;
/**
 * makes a POST request
 *
 * @param url request URL
 * @param body request body
 * @param options request options
 *
 * @return promise resolving to the response
 */
declare function post<T>(url: string, body: Body, options: PartialOptions): Promise<T>;
/**
 * makes a PUT request
 *
 * @param url request URL
 * @param body request body
 * @param options request options
 *
 * @return promise resolving to the response
 */
declare function put<T>(url: string, body: Body, options: PartialOptions): Promise<T>;
/**
 * makes a PATCH request
 *
 * @param url request URL
 * @param options request options
 *
 * @return promise resolving to the response
 */
declare function patch<T>(url: string, options: PartialOptions): Promise<T>;
/**
 * makes a DELETE request
 *
 * @param url request URL
 * @param options request options
 *
 * @return promise resolving to the response
 */
declare function deleteRequest<T>(url: string, options: PartialOptions): Promise<T>;
declare const _default: {
    request: typeof request;
    get: typeof get;
    post: typeof post;
    put: typeof put;
    patch: typeof patch;
    delete: typeof deleteRequest;
    ResponseType: typeof ResponseType;
    BodyType: typeof BodyType;
};
export default _default;
