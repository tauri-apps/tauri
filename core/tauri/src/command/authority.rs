use std::any::Any;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::future::{Future, Ready};
use std::marker::PhantomData;
use std::pin::Pin;
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

#[default_runtime(crate::WryHandle, wry)]
pub type CommandHandle<R> = fn(Box<dyn Send + Sync>, CommandRequest<R>) -> Result<CommandResponse>;

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

pub type VCmdState = Arc<dyn Any + Send + Sync>;

#[default_runtime(crate::WryHandle, wry)]
pub type VCmdHandle<R> = fn(Option<VCmdState>, CommandRequest<R>) -> VCmdFuture;

#[default_runtime(crate::WryHandle, wry)]
pub type VCmdSetup<R> = fn(&VCmd<R>);

pub type VCmdFuture = Pin<Box<dyn Future<Output = Result<CommandResponse>> + Send>>;

/*#[default_runtime(crate::WryHandle, wry)]
pub struct VCmdHandler<R: RuntimeHandle> {
  state: Option<VCmdState>,
  setup: Option<fn(Option<&VCmdState>) -> Ready<()>>,
  handle: VCmdHandle<R>,
}

impl<R: RuntimeHandle> VCmdHandler<R> {
  pub async fn handle(self, request: CommandRequest<R>) -> Result<CommandResponse> {
    if let Some(setup) = self.setup {
      setup(self.state.as_ref()).await
    }

    (self.handle)(self.state, request).await
  }
}*/

#[default_runtime(crate::WryHandle, wry)]
pub struct VCmd<R: RuntimeHandle> {
  pub name: &'static str,
  pub state: Option<VCmdState>,
  pub setup: Option<fn(Option<&VCmdState>) -> Ready<()>>,
  pub handle: VCmdHandle<R>,
}

impl<R> Clone for VCmd<R>
where
  R: RuntimeHandle,
{
  #[inline(always)]
  fn clone(&self) -> Self {
    Self {
      name: self.name,
      state: self.state.clone(),
      setup: self.setup,
      handle: self.handle,
    }
  }
}

impl<R: RuntimeHandle> VCmd<R> {
  #[inline(always)]
  fn setup(&self) -> Self {
    VCmd {
      name: self.name,
      state: self.state.clone(),
      setup: self.setup,
      handle: self.handle,
    }
  }

  pub async fn handle(self, request: CommandRequest<R>) -> Result<CommandResponse> {
    if let Some(setup) = self.setup {
      setup(self.state.as_ref()).await
    }

    (self.handle)(self.state, request).await
  }
}

#[default_runtime(crate::WryHandle, wry)]
pub struct VCmds<R: RuntimeHandle>(BTreeMap<&'static str, VCmd<R>>);

impl<R: RuntimeHandle, T: Into<BTreeMap<&'static str, VCmd<R>>>> From<T> for VCmds<R> {
  fn from(value: T) -> Self {
    Self(value.into())
  }
}

fn echo<R: RuntimeHandle>() -> VCmd<R> {
  VCmd {
    name: "echo",
    state: None,
    setup: None,
    handle: |_, _r| Box::pin(async move { Ok(CommandResponse("THIS IS AN ECHO".into())) }),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn req(scope: impl Into<BTreeMap<String, Value>>) -> CommandRequest<crate::WryHandle> {
    CommandRequest {
      scope: Arc::new(RuntimeAuthorityScope {
        raw: scope.into(),
        cache: Default::default(),
      }),
      _m: PhantomData::default(),
    }
  }

  #[tokio::test]
  async fn echo_vcmd() {
    let request = req([]);
    let echo = super::echo();
    //let cmds = VCmds::from([(echo.name, echo)]);

    let handler = echo.clone();
    crate::async_runtime::spawn(async move {
      assert_eq!(
        handler.handle(request).await.unwrap().0,
        String::from("THIS IS AN ECHO")
      );
    })
    .await
    .unwrap();
  }

  #[tokio::test]
  async fn echo() {
    let cmd: Box<dyn Command<crate::WryHandle> + Send + Sync> = Box::new(Echo);
    let request = req([(cmd.name().into(), Value::String("ECHO".into()))]);
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
