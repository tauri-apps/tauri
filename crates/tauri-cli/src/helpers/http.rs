// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use ureq::{Agent, Body, http::Response};

const CLI_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub fn get(url: &str) -> Result<Response<Body>, ureq::Error> {
  agent(true).get(url).call()
}

/// HTTP agent with the CLI user agent, proxy and TLS configuration.
///
/// When `http_status_as_error` is false, 4xx and 5xx responses are returned as regular
/// responses so their body can be read.
pub fn agent(http_status_as_error: bool) -> Agent {
  #[allow(unused_mut)]
  let mut config_builder = ureq::Agent::config_builder()
    .user_agent(CLI_USER_AGENT)
    .proxy(ureq::Proxy::try_from_env())
    .http_status_as_error(http_status_as_error);

  #[cfg(feature = "platform-certs")]
  {
    config_builder = config_builder.tls_config(
      ureq::tls::TlsConfig::builder()
        .root_certs(ureq::tls::RootCerts::PlatformVerifier)
        .build(),
    );
  }

  config_builder.build().into()
}
