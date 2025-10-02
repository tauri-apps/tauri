// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use ureq::{http::Response, Body};

pub fn get(url: &str) -> Result<Response<Body>, ureq::Error> {
  #[cfg(feature = "platform-certs")]
  {
    let agent = ureq::Agent::config_builder()
      .tls_config(
        ureq::tls::TlsConfig::builder()
          .root_certs(ureq::tls::RootCerts::PlatformVerifier)
          .build(),
      )
      .build()
      .new_agent();
    agent.get(url).call()
  }
  #[cfg(not(feature = "platform-certs"))]
  {
    ureq::get(url).call()
  }
}
