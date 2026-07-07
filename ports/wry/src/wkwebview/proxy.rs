// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use objc2_foundation::NSObject;
use std::ffi::{c_char, CString};

use crate::{proxy::ProxyEndpoint, Error};

#[allow(non_camel_case_types)]
pub type nw_endpoint_t = *mut NSObject;
#[allow(non_camel_case_types)]
pub type nw_protocol_options_t = *mut NSObject;
#[allow(non_camel_case_types)]
pub type nw_proxy_config_t = *mut NSObject;

#[link(name = "Network", kind = "framework")]
extern "C" {
  fn nw_endpoint_create_host(host: *const c_char, port: *const c_char) -> nw_endpoint_t;
  pub fn nw_proxy_config_create_socksv5(proxy_endpoint: nw_endpoint_t) -> nw_proxy_config_t;
  pub fn nw_proxy_config_create_http_connect(
    proxy_endpoint: nw_endpoint_t,
    proxy_tls_options: nw_protocol_options_t,
  ) -> nw_proxy_config_t;
}

impl TryFrom<ProxyEndpoint> for nw_endpoint_t {
  type Error = Error;
  fn try_from(endpoint: ProxyEndpoint) -> Result<Self, Error> {
    unsafe {
      let endpoint_host =
        CString::new(endpoint.host).map_err(|_| Error::ProxyEndpointCreationFailed)?;
      let endpoint_port =
        CString::new(endpoint.port).map_err(|_| Error::ProxyEndpointCreationFailed)?;
      let endpoint = nw_endpoint_create_host(endpoint_host.as_ptr(), endpoint_port.as_ptr());

      if endpoint.is_null() {
        Err(Error::ProxyEndpointCreationFailed)
      } else {
        Ok(endpoint)
      }
    }
  }
}
