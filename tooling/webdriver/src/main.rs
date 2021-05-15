mod cli;
mod webdriver;
mod server;

fn main() {
  let args = pico_args::Arguments::from_env().into();

  // start the native webdriver on the port specified in args
  let mut driver = webdriver::native(&args);
  driver
    .spawn()
    .expect("error while running native webdriver");

  // start our webdriver intermediary node
  server::run(args);
}
