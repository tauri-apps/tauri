// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// TODO: remove these custom types and rely on types from http crate

// custom wry types
mod request;
mod response;

// re-expose default http types
pub use http::uri::InvalidUri;
pub use http::{header, method, status, version, Uri};
// re-export httprange helper as it can be useful and we need it locally
pub use http_range::HttpRange;
pub use tauri_utils::mime_type::MimeType;

pub use self::request::{Request, RequestParts};
pub use self::response::{Builder as ResponseBuilder, Response, ResponseParts};
