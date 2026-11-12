#![cfg(unix)]
#[path = "../../../../crates/tauri/src/dev_reload.rs"]
mod dev_reload;

use dev_reload::ReloadHost;
use serde_json::json;
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, time::Duration};

struct Fixture(PathBuf);
impl Fixture {
  fn new() -> Self {
    let path = std::env::temp_dir().join(format!(
      "tauri-reload-{}-{}",
      std::process::id(),
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
    ));
    fs::create_dir_all(&path).unwrap();
    Self(path)
  }
  fn worker(&self, name: &str, body: &str) -> PathBuf {
    let path = self.0.join(name);
    fs::write(
      &path,
      format!("#!/usr/bin/env python3\nimport json, sys, time\n{body}\n"),
    )
    .unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    path
  }
  fn publish(&self, path: PathBuf) {
    fs::write(self.0.join("current"), path.to_str().unwrap()).unwrap();
  }
  fn host(&self) -> ReloadHost {
    ReloadHost::new(self.0.join("current"), json!(0), Duration::from_millis(500))
  }
}
impl Drop for Fixture {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}
fn counter(label: &str) -> String {
  format!("print(json.dumps({{'protocol':1}}), flush=True)\nfor line in sys.stdin:\n r=json.loads(line)\n n=r['state']+1\n print(json.dumps({{'state':n,'result':{{'count':n,'label':'{label}'}}}}), flush=True)")
}
#[test]
fn replaces_logic_without_losing_host_state_and_rejects_bad_generation() {
  let f = Fixture::new();
  f.publish(f.worker("one", &counter("one")));
  let mut host = f.host();
  assert_eq!(
    host.call("increment", json!(null)).unwrap(),
    json!({"count":1,"label":"one"})
  );
  f.publish(f.worker("two", &counter("two")));
  assert_eq!(
    host.call("increment", json!(null)).unwrap(),
    json!({"count":2,"label":"two"})
  );
  f.publish(f.worker("bad", "print('{\"protocol\":99}', flush=True)"));
  assert_eq!(host.call("increment", json!(null)).unwrap()["count"], 3);
  fs::remove_file(f.0.join("current")).unwrap();
  assert_eq!(host.call("increment", json!(null)).unwrap()["count"], 4);
}
#[test]
fn timeout_rejects_call_without_committing_state_and_allows_recovery() {
  let f = Fixture::new();
  f.publish(f.worker("one", &counter("one")));
  let mut host = f.host();
  host.call("increment", json!(null)).unwrap();
  f.publish(f.worker(
    "hung",
    "print('{\"protocol\":1}', flush=True)\nfor line in sys.stdin: time.sleep(60)",
  ));
  assert!(host.call("increment", json!(null)).is_err());
  f.publish(f.worker("two", &counter("two")));
  assert_eq!(host.call("increment", json!(null)).unwrap()["count"], 2);
}
#[test]
fn malformed_response_cannot_replace_committed_state() {
  let f = Fixture::new();
  f.publish(f.worker("bad", "print('{\"protocol\":1}', flush=True)\nfor line in sys.stdin: print('{\"state\":999}', flush=True)"));
  let mut host = f.host();
  assert!(host.call("increment", json!(null)).is_err());
  f.publish(f.worker("good", &counter("good")));
  assert_eq!(host.call("increment", json!(null)).unwrap()["count"], 1);
}
