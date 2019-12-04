use std::net::TcpListener;

use rand;

use rand::distributions::{Distribution, Uniform};

pub fn get_available_port() -> Option<u16> {
  let mut rng = rand::thread_rng();
  let die = Uniform::from(8000..9000);

  for _i in 0..100 {
    let port = die.sample(&mut rng);
    if port_is_available(port) {
      return Some(port);
    }
  }
  None
}

pub fn port_is_available(port: u16) -> bool {
  match TcpListener::bind(("127.0.0.1", port)) {
    Ok(_) => true,
    Err(_) => false,
  }
}
