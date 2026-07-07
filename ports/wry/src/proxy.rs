#[derive(Debug, Clone)]
pub struct ProxyEndpoint {
  /// Proxy server host (e.g. 192.168.0.100, localhost, example.com, etc.)
  pub host: String,
  /// Proxy server port (e.g. 1080, 3128, etc.)
  pub port: String,
}

#[derive(Debug, Clone)]
pub enum ProxyConfig {
  /// Connect to proxy server via HTTP CONNECT
  Http(ProxyEndpoint),
  /// Connect to proxy server via SOCKSv5
  Socks5(ProxyEndpoint),
}
