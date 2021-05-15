use crate::cli::Args;
use anyhow::Error;
use futures::TryFutureExt;
use hyper::http::uri::Authority;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Method, Request, Response, Server, Uri};
use std::convert::{Infallible, TryInto};

type HttpClient = Client<hyper::client::HttpConnector>;

async fn handle(
  client: HttpClient,
  req: Request<Body>,
  args: Args,
) -> Result<Response<Body>, Error> {
  match (req.method(), req.uri().path()) {
    (&Method::POST, "/session") => {
      println!("we saw a session post: {:#?}", req);
    }
    _ => {
      println!("we saw a general request: {:#?}", req);
    }
  }

  let req = forward_to_native_driver(req, args)?;
  dbg!(client.request(dbg!(req)).err_into().await)
}

/// Transform the request to a request for the native webdriver server.
fn forward_to_native_driver(mut req: Request<Body>, args: Args) -> Result<Request<Body>, Error> {
  let headers = req.headers_mut();
  headers.remove("host");
  headers.insert("host", "localhost:4445".parse()?);
  let (mut parts, body) = req.into_parts();
  parts.uri = dbg!(map_uri_native_port(parts.uri, args)?);
  Ok(Request::from_parts(parts, body))
}

/// Map a [`Uri`] port to the native webdriver port in [`Args`].
fn map_uri_native_port(uri: Uri, args: Args) -> Result<Uri, Error> {
  let mut parts = uri.into_parts();
  parts.authority = Some("localhost:4445".parse()?);
  parts.scheme = Some("http".parse()?);
  Ok(parts.try_into()?)
}

#[tokio::main(flavor = "current_thread")]
pub async fn run(args: Args) {
  let address = std::net::SocketAddr::from(([127, 0, 0, 1], args.port));

  // the client we use to proxy requests to the native webdriver
  let client = Client::builder()
    .http1_preserve_header_case(true)
    .http1_title_case_headers(true)
    .retry_canceled_requests(false)
    //.set_host(false)
    .build_http();

  // pass a copy of the client to the http request handler
  let service = make_service_fn(move |_| {
    let client = client.clone();
    async move {
      Ok::<_, Infallible>(service_fn(move |request| {
        handle(client.clone(), request, args)
      }))
    }
  });

  // set up a http1 server that uses the service we just created
  let server = Server::bind(&address)
    .http1_title_case_headers(true)
    .http1_preserve_header_case(true)
    .http1_only(true)
    .serve(service);

  if let Err(e) = server.await {
    eprintln!("tauri-driver http server error: {}", e);
  }
}
