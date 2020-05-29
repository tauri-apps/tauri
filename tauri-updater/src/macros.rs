/// Helper to `print!` and immediately `flush` `stdout`
macro_rules! print_flush {
  ($literal:expr) => {
      print!($literal);
      ::std::io::Write::flush(&mut ::std::io::stdout())?;
  };
  ($literal:expr, $($arg:expr),*) => {
      print!($literal, $($arg),*);
      ::std::io::Write::flush(&mut ::std::io::stdout())?;
  }
}

/// Set ssl cert env. vars to make sure openssl can find required files
macro_rules! set_ssl_vars {
  () => {
    #[cfg(target_os = "linux")]
    {
      if ::std::env::var_os("SSL_CERT_FILE").is_none() {
        ::std::env::set_var("SSL_CERT_FILE", "/etc/ssl/certs/ca-certificates.crt");
      }
      if ::std::env::var_os("SSL_CERT_DIR").is_none() {
        ::std::env::set_var("SSL_CERT_DIR", "/etc/ssl/certs");
      }
    }
  };
}
