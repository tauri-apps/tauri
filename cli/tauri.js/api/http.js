import tauri from './tauri'

/**
 * @typedef {number} ResponseType
 */
/**
 * @enum {ResponseType}
 */
const ResponseType = {
  JSON: 1,
  Text: 2,
  Binary: 3
}

/**
 * @typedef {number} BodyType
 */
/**
 * @enum {BodyType}
 */
const BodyType = {
  Form: 1,
  File: 2,
  Auto: 3
}

/**
 * @typedef {Object} HttpOptions
 * @property {String} options.method GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS, CONNECT or TRACE
 * @property {String} options.url the request URL
 * @property {Object} [options.headers] the request headers
 * @property {Object} [options.propertys] the request query propertys
 * @property {Object|String|Binary} [options.body] the request body
 * @property {Boolean} followRedirects whether to follow redirects or not
 * @property {Number} maxRedirections max number of redirections
 * @property {Number} connectTimeout request connect timeout
 * @property {Number} readTimeout request read timeout
 * @property {Number} timeout request timeout
 * @property {Boolean} allowCompression
 * @property {ResponseType} [responseType=1] response type
 * @property {BodyType} [bodyType=3] body type
*/

/**
 * makes a HTTP request
 *
 * @param {HttpOptions}  options request options
 *
 * @return {Promise<any>} promise resolving to the response
 */
function request (options) {
  return tauri.httpRequest(options)
}

/**
 * makes a GET request
 *
 * @param {String}  url request URL
 * @param {String|Object|Binary}  body request body
 * @param {HttpOptions}  options request options
 *
 * @return {Promise<any>} promise resolving to the response
 */
function get (url, options = {}) {
  return request({
    method: 'GET',
    url,
    ...options
  })
}

/**
 * makes a POST request
 *
 * @param {String}  url request URL
 * @param {String|Object|Binary}  body request body
 * @param {HttpOptions}  options request options
 *
 * @return {Promise<any>} promise resolving to the response
 */
function post (url, body = void 0, options = {}) {
  return request({
    method: 'POST',
    url,
    body,
    ...options
  })
}

/**
 * makes a PUT request
 *
 * @param {String}  url request URL
 * @param {String|Object|Binary}  body request body
 * @param {HttpOptions}  options request options
 *
 * @return {Promise<any>} promise resolving to the response
 */
function put (url, body = void 0, options = {}) {
  return request({
    method: 'PUT',
    url,
    body,
    ...options
  })
}

/**
 * makes a PATCH request
 *
 * @param {String}  url request URL
 * @param {HttpOptions}  options request options
 *
 * @return {Promise<any>} promise resolving to the response
 */
function patch (url, options = {}) {
  return request({
    method: 'PATCH',
    url,
    ...options
  })
}

/**
 * makes a DELETE request
 *
 * @param {String}  url request URL
 * @param {HttpOptions}  options request options
 *
 * @return {Promise<any>} promise resolving to the response
 */
function deleteRequest (url, options = {}) {
  return request({
    method: 'DELETE',
    url,
    ...options
  })
}

export {
  request,
  get,
  post,
  put,
  patch,
  deleteRequest,
  ResponseType,
  BodyType
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
