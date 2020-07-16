use crate::helpers::{config::get as get_config, TauriHtml};
use attohttpc::{Method, RequestBuilder};
use http::header::HeaderName;
use tiny_http::{Response, Server};
use url::Url;

#[derive(Default)]
pub struct Dev {
  exit_on_panic: bool,
}

impl Dev {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn exit_on_panic(mut self, exit_on_panic: bool) -> Self {
    self.exit_on_panic = exit_on_panic;
    self
  }

  pub fn run(self) -> crate::Result<()> {
    let config = get_config()?;
    let dev_path = Url::parse(&config.build.dev_path)?;
    let dev_port = dev_path.port().unwrap_or(80);
    std::thread::spawn(move || proxy_dev_server(&dev_path, dev_port));
    unimplemented!()
  }
}

fn proxy_dev_server(dev_path: &Url, dev_port: u16) -> crate::Result<()> {
  let config = get_config()?;

  let server_url = format!(
    "{}:{}",
    dev_path.host_str().expect("failed to read dev_path host"),
    dev_port + 1,
  );
  let server = Server::http(server_url).expect("failed to create proxy server");
  for request in server.incoming_requests() {
    let request_url = request.url();
    let mut request_builder = RequestBuilder::new(
      Method::from_bytes(request.method().to_string().as_bytes()).unwrap(),
      dev_path.join(&request_url)?.to_string(),
    );

    for header in request.headers() {
      request_builder = request_builder.header(
        HeaderName::from_bytes(header.field.as_str().as_bytes())?,
        header.value.as_str(),
      );
    }

    if request_url == "/" {
      let response = request_builder.send()?.text()?;
      let tauri_html = TauriHtml::new(&config.build.dist_dir, response).generate()?;
      request.respond(Response::from_data(tauri_html))?;
    } else {
      let response = request_builder.send()?.bytes()?;
      request.respond(Response::from_data(response))?;
    }
  }
  Ok(())
}
