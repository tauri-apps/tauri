// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// ignore_for_file: constant_identifier_names

/// Access the HTTP client written in Rust.
///
/// This package is also accessible with `window.__TAURI__.http` when
/// [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri)
/// in `tauri.conf.json` is set to `true`.
///
/// The APIs must be allowlisted on `tauri.conf.json`:
///
///```json
///{
///  "tauri": {
///    "allowlist": {
///      "http": {
///        "all": true, // enable all http APIs
///        "request": true // enable HTTP request API
///      }
///    }
///  }
///}
///```
///
/// It is recommended to allowlist only the APIs you use for optimal bundle size
/// and security.
///
/// ## Security
///
/// This API has a scope configuration that forces you to restrict the URLs and
/// paths that can be accessed using glob patterns.
///
/// For instance, this scope configuration only allows making HTTP requests to
/// the GitHub API for the `tauri-apps` organization:
/// ```json
/// {
///   "tauri": {
///     "allowlist": {
///       "http": {
///         "scope": ["https://api.github.com/repos/tauri-apps/*"]
///       }
///     }
///   }
/// }
/// ```
///
/// Trying to execute any API with a URL not configured on the scope results in
/// a [Future] rejection due to denied access.
library tauri_http;

import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:collection/collection.dart';
import 'package:meta/meta.dart';
import 'package:universal_html/html.dart' as html;

import 'helpers/tauri.dart'
    show TauriCommand, TauriCommandMessage, TauriModule, invokeTauriCommand;

typedef FormParts = Map<String, Part<dynamic>>;

class InvalidFilePartTypeException<T> implements Exception {
  const InvalidFilePartTypeException(this.type);

  final T type;

  @override
  String toString() => 'Invalid file part type.\n'
      'part must be of type: '
      'String || '
      'ByteBuffer || '
      'Iterable<int> || '
      'Uint8List || '
      'FilePart<Uint8List> || '
      'FilePart<String> || '
      'FilePart<ByteBuffer> || '
      'FilePart<Iterable<int>>, '
      'but ${type.runtimeType} was provided.';
}

class InvalidTypeException<T> implements Exception {
  const InvalidTypeException(this.type, this.dataType);

  final T type;

  final String dataType;

  @override
  String toString() => 'Invalid $dataType type.\n'
      '$dataType must be of type: '
      'String || '
      'ByteBuffer || '
      'Iterable<int> || '
      'Uint8List || '
      'but ${type.runtimeType} was provided.';
}

/// @since 1.0.0
class ClientOptions {
  const ClientOptions({
    this.maxRedirections,
    this.connectTimeout,
  });

  /// Defines the maximum number of redirects the client should follow.
  /// If set to 0, no redirects will be followed.
  final int? maxRedirections;

  final Duration? connectTimeout;
}

/// @since 1.0.0
enum ResponseType {
  JSON(1),
  Text(2),
  Binary(3);

  const ResponseType(this.id);

  final int id;
}

/// [file] must be one of the following types:
///   * [Uint8List],
///   * [ByteBuffer],
///   * [Iterable]<int>,
///   * [String].
///
/// @since 1.0.0
class FilePart<T> {
  const FilePart({
    required this.file,
    this.mime,
    this.fileName,
  });

  final T file;
  final String? mime;
  final String? fileName;

  Map<String, dynamic> toJSON() => <String, dynamic>{
        'file': file is String || file is Iterable<int> || file is Uint8List
            ? file
            : file is ByteBuffer
                ? Uint8List.view(file as ByteBuffer)
                : file.toString(),
        'mime': mime,
        'fileName': fileName,
      };
}

/// Part class to mimic JS's API where part can be:
///  * [String] or
///  * [Uint8List] or
///  * [ByteBuffer] or
///  * [Iterable]<int> or
///  * [FilePart]`<Uint8List | String | ByteBuffer | Iterable<int>>`.
class Part<T> {
  const Part._({required this.part});

  static Part<T> createPart<T>(final T part) {
    if (_isValidFileType<T>(part)) {
      return Part<T>._(part: part);
    }

    if (part is FilePart) {
      if (_isValidFileType<dynamic>(part.file)) {
        return Part<T>._(part: part);
      }
    }

    throw InvalidFilePartTypeException<T>(part);
  }

  static bool _isValidFileType<T>(final T part) =>
      part is String ||
      part is Uint8List ||
      part is ByteBuffer ||
      part is Iterable<int>;

  final T part;
}

/// The body object to be used on POST and PUT requests.
///
/// @since 1.0.0
class Body {
  /// Creates a new form data body. The form data is an object where each key is
  /// the entry name, and the value is either a [String] or a [html.File]
  /// object.
  ///
  /// By default it sets the `application/x-www-form-urlencoded`
  /// Content-Type header, but you can set it to `multipart/form-data` if the
  /// Cargo feature `http-multipart` is enabled.
  ///
  /// Note that a file path must be allowed in the `fs` allowlist scope.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/http.dart' show Body;
  ///
  /// final Body body = Body.form(
  ///   html.FormElement()
  ///     ..attributes = <String, String>{
  ///       'key': 'value',
  ///       'image': jsonEncode(
  ///         js.JsObject.jsify(
  ///           const FilePart<String>(
  ///             // Uint8List || ByteBuffer || Iterable<int> || String
  ///             file: '/path/to/file',
  ///             mime: 'image/jpeg', // optional
  ///             fileName: 'image.jpg', // optional
  ///           ).toJSON(),
  ///         ),
  ///       ),
  ///     },
  ///  );
  /// ```
  factory Body.form(final html.FormData formData) {
    // html.FormData's method `getAll`, returns a `List<Object>` and
    // takes a String argument `name`. Supposedly, this could return all the
    // attributes we need, but the method is undocumented and I couldn't find
    // any usage examples. Even if this is the intended behavior, its ambiguous
    // what we need to pass as the `name` argument. Most probably it is the
    // form's name, but it is not a mandatory attribute, so we wouldn't be able
    // identify the form it isn't set. It could also be the form's id, but
    // that's probably not the case.
    //
    // See: https://api.flutter.dev/flutter/dart-html/FormData-class.html
    final Map<String, dynamic> form = <String, dynamic>{};
    final List<Object> attributes = formData.getAll(
      formData.get('name').toString(),
    );

    final Iterable<MapEntry<String, dynamic>?> entries = attributes
        .map(
          (final Object obj) => _formPart(
            formData.get(obj.toString()).toString(),
            obj,
          ),
        )
        .toList()
      ..removeWhere(
        (final MapEntry<String, dynamic>? entry) => entry?.key == 'null',
      );

    form.addEntries(entries.whereNotNull());
    return Body._('Form', form);
  }

  /// Creates a new JSON body.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/http.dart' show Body;
  ///
  /// Body.json(
  ///   <dynamic, dynamic>{
  ///     'registered': true,
  ///     'name': 'tauri',
  ///   },
  /// );
  /// ```
  factory Body.json(final Map<dynamic, dynamic> data) => Body._('Json', data);

  /// Creates a new UTF-8 string body.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/http.dart' show Body;
  ///
  /// Body.text('The body content as a string');
  /// ```
  factory Body.text(final String value) => Body._('Text', value);

  // stringifying Uint8List doesn't return an array of numbers, so we create one
  // here
  /// Creates a new byte array body.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/http.dart' show Body;
  ///
  /// Body.bytes(Uint8List([1, 2, 3]));
  /// ```
  factory Body.bytes(final dynamic bytes) {
    late final Uint8List iterable;

    if (bytes is Iterable<int>) {
      iterable = Uint8List.fromList(bytes.toList());
    } else if (bytes is Uint8List) {
      iterable = bytes;
    } else if (bytes is ByteBuffer) {
      iterable = Uint8List.view(bytes);
    } else {
      throw UnsupportedError(
        'bytes must be Iterable<int> || Uint8List || ByteBuffer, '
        'but ${bytes.runtimeType} was provided.',
      );
    }

    return Body._('Bytes', iterable);
  }

  const Body._(this.type, this.payload);

  final String type;
  final dynamic payload;

  /// ```dart
  /// final FormParts form = <String, Part<dynamic>>{
  ///   'key': Part<String>.createPart('value'),
  ///   'image': Part<FilePart<Uint8List>>.createPart(
  ///     FilePart<Uint8List>>(
  ///       file: Uint8List.from(<int>[1, 2, 3]),
  ///       fileName: 'image.png',
  ///       mime: 'image/png',
  ///     ),
  ///   ),
  /// };
  ///
  /// final Body formBody = Body.formParts(form);
  /// ```
  static Body formParts<T>(final FormParts parts) {
    final Map<String, dynamic> form = <String, dynamic>{};

    final Iterable<MapEntry<String, dynamic>?> entries = parts.entries.map(
      (final MapEntry<String, Part<dynamic>> entry) => _formPart(
        entry.key,
        entry.value,
      ),
    );

    form.addEntries(entries.whereNotNull());
    return Body._('Form', form);
  }

  static MapEntry<String, dynamic>? _formPart<T>(
    final String key,
    final T part,
  ) {
    if (part != null) {
      dynamic convertedValue;

      if (part is String || part is Iterable<int>) {
        convertedValue = part;
      } else if (part is Uint8List) {
        convertedValue = List<int>.from(part);
      } else if (part is ByteBuffer) {
        convertedValue = List<int>.from(Uint8List.view(part));
      } else if (part is html.File) {
        convertedValue = FilePart<String>(
          file: part.name,
          mime: part.type,
          fileName: part.name,
        );
      } else if (part is FilePart<dynamic>) {
        if (part.file is Uint8List || part.file is Iterable<int>) {
          convertedValue = FilePart<List<int>>(
            file: List<int>.from(part.file),
            fileName: part.fileName,
            mime: part.mime,
          );
        } else if (part.file is ByteBuffer) {
          convertedValue = FilePart<List<int>>(
            file: List<int>.from(Uint8List.view(part.file)),
            fileName: part.fileName,
            mime: part.mime,
          );
        } else if (part.file is String) {
          convertedValue = part as FilePart<String>;
        } else {
          throw InvalidFilePartTypeException<dynamic>(part.file);
        }
      } else {
        throw InvalidFilePartTypeException<T>(part);
      }

      return MapEntry<String, dynamic>(key, convertedValue);
    }

    return null;
  }
}

/// The request HTTP verb.
enum HttpVerb {
  GET,
  POST,
  PUT,
  DELETE,
  PATCH,
  HEAD,
  OPTIONS,
  CONNECT,
  TRACE;
}

/// Options object sent to the backend.
///
/// @since 1.0.0
class HttpOptions {
  const HttpOptions({
    required this.url,
    required this.fetchOptions,
  });

  final String url;
  final FetchOptions fetchOptions;

  HttpOptions copyWith({
    final String? url,
    final FetchOptions? fetchOptions,
  }) =>
      HttpOptions(
        url: url ?? this.url,
        fetchOptions: fetchOptions ?? this.fetchOptions,
      );
}

/// Request options.
class RequestOptions {
  const RequestOptions({
    this.headers,
    this.query,
    this.body,
    this.timeout,
    this.responseType,
  });

  final Map<String, dynamic>? headers;
  final Map<String, dynamic>? query;
  final Body? body;
  final Duration? timeout;
  final ResponseType? responseType;

  RequestOptions copyWith({
    final Map<String, dynamic>? headers,
    final Map<String, dynamic>? query,
    final Body? body,
    final Duration? timeout,
    final ResponseType? responseType,
  }) =>
      RequestOptions(
        headers: headers ?? this.headers,
        query: query ?? this.query,
        body: body ?? this.body,
        timeout: timeout ?? this.timeout,
        responseType: responseType ?? this.responseType,
      );
}

/// Options for the `fetch` API.
class FetchOptions {
  const FetchOptions({
    required this.method,
    this.requestOptions,
  });

  final HttpVerb method;
  final RequestOptions? requestOptions;

  FetchOptions copyWith({
    final HttpVerb? method,
    final RequestOptions? requestOptions,
  }) =>
      FetchOptions(
        method: method ?? this.method,
        requestOptions: requestOptions ?? this.requestOptions,
      );
}

mixin IResponse<T> {
  abstract final String url;
  abstract final int status;
  abstract final Map<String, String> headers;
  abstract final Map<String, List<String>> rawHeaders;
  abstract final T data;
}

/// Response object.
///
/// @since 1.0.0
class Response<T> {
  factory Response(final IResponse<T> response) => Response<T>._(
        url: response.url,
        status: response.status,
        ok: response.status >= 200 && response.status < 300,
        headers: response.headers,
        rawHeaders: response.rawHeaders,
        data: response.data,
      );

  const Response._({
    required this.url,
    required this.status,
    required this.ok,
    required this.headers,
    required this.rawHeaders,
    required this.data,
  });

  /// The request URL.
  final String url;

  /// The response status code.
  final int status;

  /// A boolean indicating whether the response was successful (status in the
  /// range 200-299) or not.
  final bool ok;

  /// The response headers.
  final Map<String, String> headers;

  /// The response raw headers.
  final Map<String, List<String>> rawHeaders;

  /// The response data.
  final T data;

  Response<T> copyWith({
    final String? url,
    final int? status,
    final bool? ok,
    final Map<String, String>? headers,
    final Map<String, List<String>>? rawHeaders,
    final T? data,
  }) =>
      Response<T>._(
        url: url ?? this.url,
        status: status ?? this.status,
        ok: ok ?? this.ok,
        headers: headers ?? this.headers,
        rawHeaders: rawHeaders ?? this.rawHeaders,
        data: data ?? this.data,
      );
}

/// @since 1.0.0
class Client {
  const Client(this.id);

  final int id;

  @internal
  static Client? defaultClient;

  /// Drops the client instance.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/http.dart' show getClient;
  ///
  /// (() async {
  ///   final Client client = await getClient();
  ///
  ///   await client.drop();
  /// })();
  /// ```
  Future<void> drop() async => invokeTauriCommand<void>(
        TauriCommand(
          tauriModule: TauriModule.Http,
          message: TauriCommandMessage(
            cmd: 'dropClient',
            client: id,
          ),
        ),
      );

  /// Makes an HTTP request.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/http.dart' show getClient;
  ///
  /// (() async {
  ///   final Client client = await getClient();
  ///
  ///   final Response response = await client.request(
  ///     httpOptions: HttpOptions(
  ///       url: 'http://localhost:3003/users',
  ///       fetchOptions: FetchOptions(
  ///         method: HttpVerb.GET,
  ///       ),
  ///     ),
  ///   );
  /// })();
  /// ```
  Future<Response<T>> request<T>({
    required final HttpOptions httpOptions,
  }) async {
    HttpOptions modifiedOptions = httpOptions;
    final ResponseType? responseType =
        httpOptions.fetchOptions.requestOptions?.responseType;
    final bool jsonResponse =
        responseType == null || responseType == ResponseType.JSON;

    if (jsonResponse) {
      modifiedOptions = httpOptions.copyWith(
        fetchOptions: httpOptions.fetchOptions.copyWith(
          requestOptions: httpOptions.fetchOptions.requestOptions?.copyWith(
                responseType: ResponseType.Text,
              ) ??
              const RequestOptions(responseType: ResponseType.Text),
        ),
      );
    }

    return invokeTauriCommand<IResponse<T>>(
      TauriCommand(
        tauriModule: TauriModule.Http,
        message: TauriCommandMessage(
          cmd: 'httpRequest',
          client: id,
          options: modifiedOptions,
        ),
      ),
    ).then(
      (final IResponse<T> res) {
        Response<T> response = Response<T>(res);
        if (jsonResponse) {
          try {
            response = response.copyWith(
              data: jsonDecode(response.data.toString()),
            );
          } on Exception catch (e) {
            if (response.ok && response.data.toString().isEmpty) {
              response = response.copyWith(data: <dynamic, dynamic>{} as T);
            } else if (response.ok) {
              throw Exception(
                'Failed to parse response `${response.data}` as JSON: $e;\n'
                'try setting the `responseType` option to `ResponseType.Text` '
                'or `ResponseType.Binary` if the API does not return a JSON '
                'response.',
              );
            }
          }
          return response;
        }
        return response;
      },
    );
  }

  /// Makes a GET request.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/http.dart' show getClient, ResponseType;
  ///
  /// (() async {
  ///   final Client client = await getClient();
  ///
  ///   final response = await client.get(
  ///     url: 'http://localhost:3003/users',
  ///     requestOptions: RequestOptions(
  ///       timeout: 30,
  ///       // the expected response type
  ///       responseType: ResponseType.JSON,
  ///     ),
  ///   );
  /// })();
  /// ```
  Future<Response<T>> get<T>({
    required final String url,
    final RequestOptions? requestOptions,
  }) async =>
      request<T>(
        httpOptions: HttpOptions(
          url: url,
          fetchOptions: FetchOptions(
            method: HttpVerb.GET,
            requestOptions: requestOptions,
          ),
        ),
      );

  /// Makes a POST request.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/http.dart'
  ///   show getClient, Body, ResponseType;
  ///
  /// (() async {
  ///   final Client client = await getClient();
  ///   final Response response = await client.post(
  ///     url: 'http://localhost:3003/users',
  ///     requestOptions: RequestOptions(
  ///       body: Body.json(
  ///         <dynamic, dynamic>{
  ///           'name': 'tauri',
  ///           'password': 'awesome',
  ///         },
  ///       ),
  ///       // in this case the server returns a simple string
  ///       responseType: ResponseType.Text,
  ///     ),
  ///   );
  /// })();
  /// ```
  Future<Response<T>> post<T>({
    required final String url,
    final RequestOptions? requestOptions,
  }) async =>
      request<T>(
        httpOptions: HttpOptions(
          url: url,
          fetchOptions: FetchOptions(
            method: HttpVerb.POST,
            requestOptions: requestOptions,
          ),
        ),
      );

  /// Makes a PUT request.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/http.dart' show getClient, Body;
  ///
  /// (() async {
  ///   final Client client = await getClient();
  ///   final Response response = await client.put(
  ///     url: 'http://localhost:3003/users/1',
  ///     requestOptions: RequestOptions(
  ///       body: Body.formData<FilePart<String>>(
  ///         FilePart<String>(
  ///           file: '/home/tauri/avatar.png',
  ///           mime: 'image/png',
  ///           fileName: 'avatar.png',
  ///         ),
  ///       ),
  ///     ),
  ///   );
  /// })();
  /// ```
  Future<Response<T>> put<T>({
    required final String url,
    final RequestOptions? requestOptions,
  }) async =>
      request<T>(
        httpOptions: HttpOptions(
          url: url,
          fetchOptions: FetchOptions(
            method: HttpVerb.PUT,
            requestOptions: requestOptions,
          ),
        ),
      );

  /// Makes a PATCH request.
  ///
  /// ```dart
  /// import 'package:tauri_apps/api/http.dart' show getClient, Body;
  ///
  /// (() async {
  ///   final Client client = await getClient();
  ///   final Response response = await client.patch(
  ///     url: 'http://localhost:3003/users/1',
  ///     requestOptions: RequestOptions(
  ///       body: Body.json(<dynamic, dynamic>>{'email': 'contact@tauri.app'}),
  ///     ),
  ///   );
  /// });
  /// ```
  Future<Response<T>> patch<T>({
    required final String url,
    final RequestOptions? requestOptions,
  }) async =>
      request<T>(
        httpOptions: HttpOptions(
          url: url,
          fetchOptions: FetchOptions(
            method: HttpVerb.PATCH,
            requestOptions: requestOptions,
          ),
        ),
      );

  /// Makes a DELETE request.
  ///
  /// ```dart
  /// import from 'package:tauri_apps/api/http.dart' show getClient;
  ///
  /// (() async {
  ///   final Client client = await getClient();
  ///   final Response response = await client.delete(
  ///     url: 'http://localhost:3003/users/1',
  ///   );
  /// });
  /// ```
  Future<Response<T>> delete<T>({
    required final String url,
    final RequestOptions? requestOptions,
  }) async =>
      request<T>(
        httpOptions: HttpOptions(
          url: url,
          fetchOptions: FetchOptions(
            method: HttpVerb.DELETE,
            requestOptions: requestOptions,
          ),
        ),
      );
}

/// Creates a new client using the specified options.
/// ```dart
/// import 'package:tauri_apps/api/http.dart' show getClient;
///
/// (() async {
///   final Client client = await getClient();
/// })();
/// ```
/// @since 1.0.0
Future<Client> getClient({
  final ClientOptions options = const ClientOptions(),
}) =>
    invokeTauriCommand<int>(
      TauriCommand(
        message: TauriCommandMessage(
          cmd: 'createClient',
          options: options,
        ),
        tauriModule: TauriModule.Http,
      ),
    ).then(Client.new);

/// Perform an HTTP request using the default client.
///
/// ```dart
/// import 'package:tauri_apps/api/http.dart' show fetch;
///
/// (() async {
///   final Response response = await fetch(
///     url: 'http://localhost:3003/users/2',
///     fetchOptions: FetchOptions(
///       method: HttpVerb.GET,
///       requestOptions: RequestOptions(timeout: 30),
///     ),
///   );
/// })();
/// ```
Future<Response<T>> fetch<T>({
  required final String url,
  final FetchOptions? fetchOptions,
}) async {
  Client.defaultClient ??= await getClient();

  return Client.defaultClient!.request<T>(
    httpOptions: HttpOptions(
      url: url,
      fetchOptions: fetchOptions ??
          FetchOptions(
            method: fetchOptions?.method ?? HttpVerb.GET,
          ),
    ),
  );
}
