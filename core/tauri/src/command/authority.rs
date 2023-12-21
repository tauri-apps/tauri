use std::collections::BTreeMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, RwLock};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use state::TypeMap;

use serde_json::Value;
use tauri_macros::default_runtime;

use crate::command;
use crate::ipc::InvokeMessage;
use crate::{Runtime, RuntimeHandle};

/// Alias for `Result<T, tauri::command::Error>`.
pub type Result<T> = ::std::result::Result<T, Error>;

pub struct Caps;

pub struct RuntimeAuthority {
  caps: Caps,
  cmds: (),
}

#[derive(Debug)]
pub enum Error {
  NotFound,
}

pub trait IntoCommandResponse {
  fn into_command_response(self) -> CommandResponse;
}

#[derive(Debug, Default)]
pub struct RuntimeAuthorityScope {
  raw: BTreeMap<String, Value>,
  cache: TypeMap![Send + Sync],
}

impl RuntimeAuthorityScope {
  pub fn get_typed<T: Scoped + Debug + 'static>(&self, key: &str) -> Option<&T> {
    match self.cache.try_get() {
      cached @ Some(_) => cached,
      //None => match dbg!(self.raw.get(key).and_then(Value::deserialize::<T>)) {
      None => match self
        .raw
        .get(key)
        .and_then(|r| dbg!(serde_json::from_value::<T>(dbg!(r.clone()))).ok())
        //.and_then(|r| dbg!(serde_json::from_str::<T>(&r)).ok())
      {
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

    impl<R: RuntimeHandle> StrictCommand<R> for Cmd {
      type Scope = WeakScope;
      type Response = CommandResponse;

      fn name(&self) -> &'static str {
        "test"
      }

      fn handle(
        &self,
        _scope: &Self::Scope,
        request: CommandRequest<R>,
      ) -> Result<Self::Response, Error> {
        let arg1 = CommandArg::from_command(CommandItem {
          name: "test",
          key: "string",
          message: &request.msg,
        })
        .unwrap();

        test(arg1);

        Ok(CommandResponse)
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

impl<'a> TryFrom<&'a RuntimeAuthorityScope> for &'a WeakScope {
  type Error = Error;

  fn try_from(value: &'a RuntimeAuthorityScope) -> ::std::result::Result<Self, Self::Error> {
    value.get_typed("weak-scope").ok_or(Error::NotFound)
  }
}

#[default_runtime(crate::WryHandle, wry)]
struct CommandRequest<R: RuntimeHandle> {
  scope: Arc<RuntimeAuthorityScope>,
  //msg: InvokeMessage<R>,
  _m: PhantomData<R>,
}

struct CommandResponse(String);

impl<S> IntoCommandResponse for S
where
  S: Serialize,
{
  fn into_command_response(self: S) -> CommandResponse {
    CommandResponse(serde_json::to_string(&self).unwrap())
  }
}

static_assertions::assert_obj_safe!(Command);
#[default_runtime(crate::WryHandle, wry)]
trait Command<R: RuntimeHandle> {
  fn name(&self) -> &'static str;

  fn handler(&self) -> fn(CommandRequest<R>) -> Result<CommandResponse>;

  //fn handle(&self, request: CommandRequest<R>) -> Result<CommandResponse>;
}

#[default_runtime(crate::WryHandle, wry)]
trait StrictCommand<R: RuntimeHandle> {
  type Scope: Scoped + Debug + 'static;
  type Response: IntoCommandResponse;

  fn name() -> &'static str;

  fn handle(scope: &Self::Scope, request: CommandRequest<R>) -> Result<Self::Response>;
}

fn strict_handler<R: RuntimeHandle, T: StrictCommand<R>>(
  request: CommandRequest<R>,
) -> Result<CommandResponse> {
  let scopes = dbg!(request.scope.clone());
  let scope = dbg!(scopes.get_typed::<T::Scope>(T::name())).unwrap();
  T::handle(scope, request).map(IntoCommandResponse::into_command_response)
}

impl<T, R> Command<R> for T
where
  R: RuntimeHandle,
  T: StrictCommand<R>,
{
  fn name(&self) -> &'static str {
    T::name()
  }

  fn handler(&self) -> fn(CommandRequest<R>) -> Result<CommandResponse> {
    strict_handler::<R, T>
  }
}

struct Echo;

impl Scoped for String {}

impl<R: RuntimeHandle> StrictCommand<R> for Echo {
  type Scope = String;
  type Response = String;

  fn name() -> &'static str {
    "echo"
  }

  fn handle(scope: &Self::Scope, request: CommandRequest<R>) -> Result<Self::Response> {
    assert_eq!(scope, "ECHO");
    Ok("THIS IS AN ECHO".into())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::EventLoopMessage;
  use tauri_runtime_wry::WryHandle;

  #[tokio::test]
  async fn echo() {
    let cmd: Box<dyn Command<crate::WryHandle> + Send + Sync> = Box::new(Echo);
    let request: CommandRequest<crate::WryHandle> = CommandRequest {
      scope: Arc::new(RuntimeAuthorityScope {
        raw: [(cmd.name().into(), Value::String("ECHO".into()))].into(),
        cache: Default::default(),
      }),
      _m: PhantomData::default(),
    };
    let handler = dbg!(cmd.handler());

    assert_eq!(cmd.name(), "echo");
    crate::async_runtime::spawn(async move {
      assert_eq!(
        handler(request).unwrap().0,
        String::from("\"THIS IS AN ECHO\"")
      );
    })
    .await
    .unwrap();
  }
}
