// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use ureq::{http::Response, Agent, Body};

const CLI_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub fn get(url: &str) -> Result<Response<Body>, ureq::Error> {
  #[allow(unused_mut)]
  let mut config_builder = ureq::Agent::config_builder()
    .user_agent(CLI_USER_AGENT)
    .proxy(ureq::Proxy::try_from_env());

  #[cfg(feature = "platform-certs")]
  {
    config_builder = config_builder.tls_config(
      ureq::tls::TlsConfig::builder()
        .root_certs(ureq::tls::RootCerts::PlatformVerifier)
        .build(),
    );
  }

  let agent: Agent = config_builder.build().into();
  agent.get(url).call()
}
