use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde::de::DeserializeOwned;
use serde::Deserialize;
use state::TypeMap;

use tauri_macros::default_runtime;
use tauri_utils::acl::Value;

use crate::command;
use crate::command::{CommandArg, CommandItem};
use crate::ipc::InvokeMessage;
use crate::Runtime;

pub struct Caps;

pub struct RuntimeAuthority {
  caps: Caps,
  cmds: (),
}

pub enum CommandError {}
pub trait IntoCommandResponse {}

pub struct RuntimeAuthorityScope {
  raw: BTreeMap<String, Value>,
  cache: TypeMap![Send + Sync],
}

impl RuntimeAuthorityScope {
  pub fn get_typed<T: Scoped + 'static>(&self, key: &str) -> Option<&T> {
    match self.cache.try_get() {
      cached @ Some(_) => cached,
      None => match self.raw.get(key).and_then(Value::deserialize::<T>) {
        None => None,
        Some(value) => {
          let _ = self.cache.set(value);
          self.cache.try_get()
        }
      },
    }
  }
}

macro_rules! command {
  ($command:ident) => {
    $command!()
  };
}

#[command]
fn test(string: String) -> String {
  string
}

macro_rules! test {
  () => {{
    struct Cmd;

    impl<R: Runtime> StrictCommand<R> for Cmd {
      const ID: &'static str = "test";

      type Scope = WeakScope;

      fn handle(
        &self,
        _scope: &Self::Scope,
        request: CommandRequest<R>,
      ) -> Result<(), CommandError> {
        let arg1 = CommandArg::from_command(CommandItem {
          name: "test",
          key: "string",
          message: &request.msg,
        })
        .unwrap();

        test(arg1);

        Ok(())
      }
    }

    Box::new(Cmd)
  }};
}

trait Scoped: Send + Sync + DeserializeOwned {}

#[derive(Debug, Deserialize)]
struct WeakScope;

impl Scoped for WeakScope {}

impl Scoped for () {}

#[default_runtime(crate::Wry, wry)]
struct CommandRequest<R: Runtime> {
  scope: Arc<RuntimeAuthorityScope>,
  msg: InvokeMessage<R>,
}

struct CommandResponse;

static_assertions::assert_obj_safe!(Command);
#[default_runtime(crate::Wry, wry)]
trait Command<R: Runtime> {
  fn name(&self) -> &'static str;

  fn handle(&self, request: CommandRequest<R>) -> Result<(), CommandError>;
}

#[default_runtime(crate::Wry, wry)]
trait StrictCommand<R: Runtime> {
  const ID: &'static str;

  type Scope: Scoped + 'static;

  fn handle(&self, scope: &Self::Scope, request: CommandRequest<R>) -> Result<(), CommandError>;
}

impl<T, R> Command<R> for T
where
  R: Runtime,
  T: StrictCommand<R>,
{
  fn name(&self) -> &'static str {
    T::ID
  }

  fn handle(&self, request: CommandRequest<R>) -> Result<(), CommandError> {
    let scopes = request.scope.clone();
    let scope = scopes.get_typed::<T::Scope>(T::ID).unwrap();
    self.handle(scope, request)
  }
}

/*
/// Represents any item that can be turned into a command.
pub trait CommandHandler {
  fn handle(&self, invoke: InvokeMessage);
}*/

struct MyCommand {
  inner: Arc<Mutex<String>>,
}

impl<R: Runtime> StrictCommand<R> for MyCommand {
  const ID: &'static str = "my_command";
  type Scope = ();

  fn handle(&self, _scope: &Self::Scope, request: CommandRequest<R>) -> Result<(), CommandError> {
    self.inner.lock().unwrap();

    Ok(())
  }
}

fn asdf() {
  let _: &[Box<dyn Command<crate::Wry> + Send + Sync>] = &[command!(test)];
}
