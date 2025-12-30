// Copyright 2025 Tauri contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Context, Result};
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::path::Path;
use std::process::Command;
use which::which;

// Exit code mapping for `tauri audit` (centralized here):
// 0 = success (no policy violation, includes warn mode)
// 1 = policy violation (mode=error and failOn matched)
// 2 = audit tool execution failure (tool missing or command failed and no parsable JSON)
// 3 = config/schema/argument error
const EXIT_POLICY_VIOLATION: i32 = 1;
const EXIT_EXEC_FAILURE: i32 = 2;
const EXIT_CONFIG_ERROR: i32 = 3;

fn audit_exit(code: i32, msg: Option<&str>) -> ! {
  if let Some(m) = msg {
    eprintln!("{}", m);
  }
  std::process::exit(code)
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Mode {
  Off,
  Warn,
  Error,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
  Human,
  Json,
  Sarif,
}

#[derive(Debug, Parser)]
#[clap(about = "Run dependency audits for Rust (cargo-audit) and JS (future)")]
pub struct Options {
  /// Action mode when findings exceed policy. If omitted, `build.audit` config is used.
  #[clap(long, value_enum)]
  pub mode: Option<Mode>,

  /// Output format. If omitted, `build.audit` config is used.
  #[clap(long, value_enum)]
  pub format: Option<OutputFormat>,

  /// If tools are missing, attempt to install them (opt-in)
  #[clap(long, action)]
  pub install_tools: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
  Unknown,
  Low,
  Medium,
  High,
  Critical,
}

impl Severity {
  fn from_cvss(score: Option<f64>) -> Self {
    match score {
      Some(s) if s >= 9.0 => Severity::Critical,
      Some(s) if s >= 7.0 => Severity::High,
      Some(s) if s >= 4.0 => Severity::Medium,
      Some(s) if s > 0.0 => Severity::Low,
      _ => Severity::Unknown,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Finding {
  pub ecosystem: String,
  pub package: Option<String>,
  pub version: Option<String>,
  pub advisory_id: Option<String>,
  pub severity: Severity,
  pub title: Option<String>,
  pub url: Option<String>,
  pub fix_available: Option<bool>,
}
// audit context: lockfile paths discovered while running providers
#[derive(Default, Clone)]
struct AuditContext {
  js_lockfile: Option<String>,
  rust_lockfile: Option<String>,
}

// helper: parse yarn version string and detect classic (major == 1)
fn yarn_is_classic_from_version(s: &str) -> bool {
  s.split('.')
    .next()
    .and_then(|m| m.parse::<u64>().ok())
    .map(|maj| maj == 1)
    .unwrap_or(false)
}

pub fn command(options: Options) -> Result<()> {
  // load config (if available) to allow build.audit overrides
  crate::helpers::app_paths::resolve();
  let config_handle =
    match crate::helpers::config::get(tauri_utils::platform::Target::current(), &[]) {
      Ok(h) => Some(h),
      Err(e) => {
        // Map config/schema/argument errors to the centralized exit code (3)
        audit_exit(EXIT_CONFIG_ERROR, Some(&format!("{}", e)));
      }
    };

  // precedence: defaults <- config <- CLI (CLI overrides config when provided)
  let mut mode = Mode::Warn;
  let mut format = OutputFormat::Human;
  let mut sarif_path: Option<String> = None;
  let mut fail_on: Option<Severity> = None;
  let mut ignore_advisories: Vec<String> = Vec::new();
  let mut ignore_packages: Vec<String> = Vec::new();
  // audit context holds lockfile paths used for SARIF artifact locations

  let mut ctx = AuditContext::default();
  // Extract audit-related settings from the loaded config handle.
  // Prefer accessing typed `Config` fields directly; however, the `Config` structure
  // may contain nested optional fields and platform extensions. For flexibility and
  // to avoid fragile field-by-field code, perform a confined serde-json conversion
  // in this small helper only for the `build.audit` subtree. This keeps the rest
  // of the code working with typed values and restricts serialization to a tiny,
  // well-reviewed function.
  if let Some(handle) = config_handle {
    if let Some(cfg_meta) = &*handle.lock().unwrap() {
      // confined helper: parse the `build.audit` subtree via JSON value access
      #[allow(clippy::type_complexity)]
      fn extract_audit_from_meta(
        cfg_meta: &crate::helpers::config::ConfigMetadata,
      ) -> (
        Option<String>,
        Option<String>,
        Option<String>,
        Vec<String>,
        Vec<String>,
        Option<String>,
      ) {
        // use `&**cfg_meta` to coerce through `Deref<Config>` and serialize the inner `Config` value
        if let Ok(cfg_val) = serde_json::to_value(&**cfg_meta) {
          if let Some(audit_cfg) = cfg_val.get("build").and_then(|b| b.get("audit")) {
            let mode = audit_cfg
              .get("mode")
              .and_then(|v| v.as_str())
              .map(|s| s.to_string());
            let fail_on = audit_cfg
              .get("failOn")
              .and_then(|v| v.as_str())
              .map(|s| s.to_string());
            let mut ignore_advisories = Vec::new();
            if let Some(rust_cfg) = audit_cfg.get("rust") {
              if let Some(arr) = rust_cfg.get("ignore").and_then(|v| v.as_array()) {
                for v in arr.iter().filter_map(|x| x.as_str()) {
                  ignore_advisories.push(v.to_string());
                }
              }
            }
            let mut ignore_packages = Vec::new();
            if let Some(js_cfg) = audit_cfg.get("js") {
              if let Some(arr) = js_cfg.get("ignore").and_then(|v| v.as_array()) {
                for v in arr.iter().filter_map(|x| x.as_str()) {
                  ignore_packages.push(v.to_string());
                }
              }
            }
            let mut format = None;
            if let Some(of) = audit_cfg.get("output").and_then(|o| o.get("formats")) {
              if let Some(arr) = of.as_array() {
                if arr.iter().any(|v| v.as_str() == Some("sarif")) {
                  format = Some("sarif".to_string());
                } else if arr.iter().any(|v| v.as_str() == Some("json")) {
                  format = Some("json".to_string());
                }
              }
            }
            let sarif_path = audit_cfg
              .get("output")
              .and_then(|o| o.get("sarifPath"))
              .and_then(|s| s.as_str())
              .map(|s| s.to_string());

            return (
              mode,
              format,
              fail_on,
              ignore_advisories,
              ignore_packages,
              sarif_path,
            );
          }
        }
        (None, None, None, Vec::new(), Vec::new(), None)
      }

      let (m, f, fo, rust_ignores, js_ignores, sp) = extract_audit_from_meta(cfg_meta);
      if let Some(m) = m {
        mode = match m.as_str() {
          "off" => Mode::Off,
          "warn" => Mode::Warn,
          "error" => Mode::Error,
          _ => mode,
        };
      }
      if let Some(fo) = fo {
        fail_on = match fo.as_str() {
          "low" => Some(Severity::Low),
          "medium" => Some(Severity::Medium),
          "high" => Some(Severity::High),
          "critical" => Some(Severity::Critical),
          _ => fail_on,
        };
      }
      ignore_advisories.extend(rust_ignores);
      ignore_packages.extend(js_ignores);
      if let Some(fmt) = f {
        if fmt == "sarif" {
          format = OutputFormat::Sarif;
        } else if fmt == "json" {
          format = OutputFormat::Json;
        }
      }
      sarif_path = sp;
    }
  }

  // CLI overrides config when provided
  if let Some(cli_mode) = options.mode {
    mode = cli_mode;
  }
  if let Some(cli_fmt) = options.format {
    format = cli_fmt;
  }

  if let Mode::Off = mode {
    println!("audit mode is off");
    return Ok(());
  }

  // For MVP, implement Rust provider via `cargo audit --json`.
  let mut findings = Vec::new();
  let (rust_findings, rust_lockfile) = run_cargo_audit(options.install_tools)
    .context("failed to run cargo audit and parse results")?;
  ctx.rust_lockfile = rust_lockfile;
  findings.extend(rust_findings);

  let (js_findings, js_lockfile) =
    run_js_audit(options.install_tools).context("failed to run JS audit and parse results")?;
  ctx.js_lockfile = js_lockfile;
  findings.extend(js_findings);

  // apply ignore filters deterministically
  let filtered = apply_ignores(&findings, &ignore_advisories, &ignore_packages);

  // dedupe and sort for deterministic output
  let out = dedupe_and_sort(filtered);

  // evaluate policy (failOn + mode)
  // On policy violation we exit with code 1 per contract
  if let Err(e) = evaluate_policy(&out, &mode, fail_on) {
    // evaluation returns Err only for config/argument problems; map to exit code 3
    audit_exit(EXIT_CONFIG_ERROR, Some(&format!("{}", e)));
  }

  match format {
    OutputFormat::Human => print_human(&out),
    OutputFormat::Json => {
      let s = serde_json::to_string_pretty(&out).context("failed to serialize findings to JSON")?;
      println!("{}", s);
    }
    OutputFormat::Sarif => {
      let sarif = to_sarif(&out, &ctx);
      let s = serde_json::to_string_pretty(&sarif).context("failed to serialize SARIF")?;
      if let Some(path) = sarif_path {
        std::fs::write(path, &s).context("failed to write SARIF file")?;
      } else {
        println!("{}", s);
      }
    }
  }

  // Policy: if mode == Error and any finding has severity High or Critical, fail.
  // printing happens after policy evaluation above

  Ok(())
}

fn print_human(findings: &[Finding]) {
  if findings.is_empty() {
    println!("No vulnerabilities found (cargo-audit)");
    return;
  }
  println!("Found {} vulnerabilities:", findings.len());
  for f in findings {
    let sev = match f.severity {
      Severity::Critical => "CRITICAL",
      Severity::High => "HIGH",
      Severity::Medium => "MEDIUM",
      Severity::Low => "LOW",
      Severity::Unknown => "UNKNOWN",
    };
    println!(
      "- [{sev}] {} {}",
      f.advisory_id.as_deref().unwrap_or(""),
      f.title.as_deref().unwrap_or("")
    );
    if let Some(url) = &f.url {
      println!("    -> {}", url);
    }
  }
}
fn severity_rank(s: &Severity) -> i8 {
  match s {
    Severity::Unknown => 0,
    Severity::Low => 1,
    Severity::Medium => 2,
    Severity::High => 3,
    Severity::Critical => 4,
  }
}

#[allow(clippy::type_complexity)]
fn dedupe_and_sort(findings: Vec<Finding>) -> Vec<Finding> {
  use std::collections::BTreeMap;
  // key -> highest severity + first occurrence
  let mut map: BTreeMap<(String, Option<String>, Option<String>, Option<String>), Finding> =
    BTreeMap::new();
  for f in findings.into_iter() {
    let key = (
      f.ecosystem.clone(),
      f.package.clone(),
      f.advisory_id.clone(),
      f.title.clone(),
    );
    if let Some(existing) = map.get_mut(&key) {
      if severity_rank(&f.severity) > severity_rank(&existing.severity) {
        *existing = f;
      }
    } else {
      map.insert(key, f);
    }
  }

  let mut v: Vec<Finding> = map.into_values().collect();
  // sort by severity desc, ecosystem, package, advisory_id, title
  v.sort_by(|a, b| {
    severity_rank(&b.severity)
      .cmp(&severity_rank(&a.severity))
      .then(a.ecosystem.cmp(&b.ecosystem))
      .then(a.package.cmp(&b.package))
      .then(a.advisory_id.cmp(&b.advisory_id))
      .then(a.title.cmp(&b.title))
  });
  v
}

fn apply_ignores(
  findings: &[Finding],
  ignore_advisories: &[String],
  ignore_packages: &[String],
) -> Vec<Finding> {
  findings
    .iter()
    .filter(|f| {
      if let Some(id) = &f.advisory_id {
        if ignore_advisories.iter().any(|ig| ig == id) {
          return false;
        }
      }
      if let Some(pkg) = &f.package {
        if ignore_packages.iter().any(|ig| ig == pkg) {
          return false;
        }
      }
      true
    })
    .cloned()
    .collect()
}

fn to_sarif(findings: &[Finding], ctx: &AuditContext) -> serde_json::Value {
  use serde_json::json;
  // build rules and results
  let mut rules_map = std::collections::BTreeMap::<String, serde_json::Value>::new();
  let mut results = Vec::new();
  for f in findings.iter() {
    let rule_id = f
      .advisory_id
      .clone()
      .or_else(|| f.package.as_ref().map(|p| format!("{}:{}", f.ecosystem, p)))
      .unwrap_or_else(|| format!("{}:unknown", f.ecosystem));
    let level = match f.severity {
      Severity::Critical | Severity::High => "error",
      Severity::Medium => "warning",
      Severity::Low | Severity::Unknown => "note",
    };

    if !rules_map.contains_key(&rule_id) {
      rules_map.insert(
        rule_id.clone(),
        json!({
          "id": rule_id.clone(),
          "shortDescription": { "text": f.title.clone().unwrap_or_default() },
          "properties": { "ecosystem": f.ecosystem.clone() }
        }),
      );
    }

    let mut locations = Vec::new();
    if f.ecosystem == "rust" {
      if let Some(lock) = &ctx.rust_lockfile {
        locations.push(json!({"physicalLocation": {"artifactLocation": {"uri": lock}}}));
      }
    } else if f.ecosystem == "js" {
      if let Some(lock) = &ctx.js_lockfile {
        locations.push(json!({"physicalLocation": {"artifactLocation": {"uri": lock}}}));
      }
    }

    results.push(json!({
      "ruleId": rule_id,
      "level": level,
      "message": { "text": f.title.clone().unwrap_or_default() },
      "locations": locations,
    }));
  }

  let rules: Vec<serde_json::Value> = rules_map.into_values().collect();

  json!({
    "version": "2.1.0",
    "$schema": "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0-rtm.5.json",
    "runs": [
      {
        "tool": { "driver": { "name": "tauri-audit", "rules": rules } },
        "results": results
      }
    ]
  })
}

fn evaluate_policy(findings: &[Finding], mode: &Mode, fail_on: Option<Severity>) -> Result<()> {
  // Use rank comparison: a finding violates threshold if its rank >= fail_on rank
  let threshold = fail_on.unwrap_or(Severity::High);
  let thr_rank = severity_rank(&threshold);
  let count = findings
    .iter()
    .filter(|f| severity_rank(&f.severity) >= thr_rank)
    .count();

  if count == 0 {
    return Ok(());
  }

  match mode {
    Mode::Off => Ok(()),
    Mode::Warn => {
      println!("audit warning: {count} findings matched failOn threshold");
      Ok(())
    }
    Mode::Error => {
      // policy violation exit code 1
      eprintln!("audit failed: {count} findings matched failOn threshold");
      audit_exit(EXIT_POLICY_VIOLATION, Some("policy violation"));
    }
  }
}

// Testable helper: count how many findings meet or exceed the threshold
#[cfg(test)]
fn count_violations(findings: &[Finding], fail_on: Option<Severity>) -> usize {
  let threshold = fail_on.unwrap_or(Severity::High);
  let thr_rank = severity_rank(&threshold);
  findings
    .iter()
    .filter(|f| severity_rank(&f.severity) >= thr_rank)
    .count()
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn severity_threshold_counting() {
    let findings = vec![
      Finding {
        ecosystem: "rust".into(),
        package: None,
        version: None,
        advisory_id: None,
        severity: Severity::Low,
        title: None,
        url: None,
        fix_available: None,
      },
      Finding {
        ecosystem: "rust".into(),
        package: None,
        version: None,
        advisory_id: None,
        severity: Severity::High,
        title: None,
        url: None,
        fix_available: None,
      },
      Finding {
        ecosystem: "js".into(),
        package: None,
        version: None,
        advisory_id: None,
        severity: Severity::Critical,
        title: None,
        url: None,
        fix_available: None,
      },
    ];
    assert_eq!(count_violations(&findings, Some(Severity::High)), 2);
    assert_eq!(count_violations(&findings, Some(Severity::Low)), 3);
    assert_eq!(count_violations(&findings, Some(Severity::Critical)), 1);
  }

  #[test]
  fn ignores_exact_match() {
    let findings = vec![
      Finding {
        ecosystem: "rust".into(),
        package: Some("a".into()),
        version: None,
        advisory_id: Some("R1".into()),
        severity: Severity::High,
        title: Some("t".into()),
        url: None,
        fix_available: None,
      },
      Finding {
        ecosystem: "js".into(),
        package: Some("left-pad".into()),
        version: None,
        advisory_id: Some("100".into()),
        severity: Severity::Medium,
        title: Some("t2".into()),
        url: None,
        fix_available: None,
      },
    ];
    let filtered = apply_ignores(&findings, &vec!["R1".into()], &vec!["not-present".into()]);
    assert_eq!(filtered.len(), 1);
    let filtered2 = apply_ignores(&findings, &vec![], &vec!["left-pad".into()]);
    assert_eq!(filtered2.len(), 1);
  }

  #[test]
  fn dedupe_and_sort_behavior() {
    let findings = vec![
      Finding {
        ecosystem: "js".into(),
        package: Some("p".into()),
        version: None,
        advisory_id: Some("A".into()),
        severity: Severity::Low,
        title: Some("t1".into()),
        url: None,
        fix_available: None,
      },
      Finding {
        ecosystem: "js".into(),
        package: Some("p".into()),
        version: None,
        advisory_id: Some("A".into()),
        severity: Severity::High,
        title: Some("t1".into()),
        url: None,
        fix_available: None,
      },
      Finding {
        ecosystem: "rust".into(),
        package: Some("r".into()),
        version: None,
        advisory_id: Some("R".into()),
        severity: Severity::Medium,
        title: Some("t2".into()),
        url: None,
        fix_available: None,
      },
    ];
    let out = dedupe_and_sort(findings);
    // highest severity first -> High (js:p), then Medium (rust:r)
    assert_eq!(out[0].ecosystem, "js");
    assert_eq!(out[0].severity, Severity::High);
    assert_eq!(out[1].ecosystem, "rust");
  }

  #[test]
  fn sarif_structure_and_levels() {
    let findings = vec![
      Finding {
        ecosystem: "rust".into(),
        package: Some("r".into()),
        version: None,
        advisory_id: Some("R1".into()),
        severity: Severity::High,
        title: Some("rwarn".into()),
        url: None,
        fix_available: None,
      },
      Finding {
        ecosystem: "js".into(),
        package: Some("p".into()),
        version: None,
        advisory_id: Some("100".into()),
        severity: Severity::Low,
        title: Some("jlow".into()),
        url: None,
        fix_available: None,
      },
    ];
    let ctx = AuditContext {
      rust_lockfile: Some("Cargo.lock".into()),
      js_lockfile: Some("package-lock.json".into()),
    };
    let sarif = to_sarif(&findings, &ctx);
    let runs = sarif
      .get("runs")
      .and_then(|r| r.as_array())
      .expect("runs array");
    let tool = runs[0]
      .get("tool")
      .and_then(|t| t.get("driver"))
      .expect("driver");
    assert_eq!(
      tool.get("name").and_then(|n| n.as_str()),
      Some("tauri-audit")
    );
    let results = runs[0]
      .get("results")
      .and_then(|r| r.as_array())
      .expect("results");
    // find rust result and check level
    let rust_res = results
      .iter()
      .find(|res| res.get("level").and_then(|l| l.as_str()) == Some("error"))
      .expect("rust error level");
    let js_res = results
      .iter()
      .find(|res| res.get("level").and_then(|l| l.as_str()) == Some("note"))
      .expect("js note level");
    // check artifact locations present
    let rust_loc = rust_res
      .get("locations")
      .and_then(|l| l.as_array())
      .and_then(|a| a.get(0))
      .and_then(|loc| loc.get("physicalLocation"))
      .and_then(|p| p.get("artifactLocation"))
      .and_then(|al| al.get("uri"))
      .and_then(|u| u.as_str());
    assert_eq!(rust_loc, Some("Cargo.lock"));
    let js_loc = js_res
      .get("locations")
      .and_then(|l| l.as_array())
      .and_then(|a| a.get(0))
      .and_then(|loc| loc.get("physicalLocation"))
      .and_then(|p| p.get("artifactLocation"))
      .and_then(|al| al.get("uri"))
      .and_then(|u| u.as_str());
    assert_eq!(js_loc, Some("package-lock.json"));
  }

  #[test]
  fn yarn_classic_detection() {
    assert!(yarn_is_classic_from_version("1.22.10\n"));
    assert!(!yarn_is_classic_from_version("3.2.1\n"));
  }

  #[test]
  fn parse_cargo_example() {
    let example = json!({
      "vulnerabilities": {
        "list": [
          {
            "advisory": {
              "id": "RUSTSEC-2020-0001",
              "title": "example vuln",
              "url": "https://example",
              "package": "example-crate",
              "cvss": 7.5,
              "patched_versions": "^1.2.3"
            },
            "package": {"version": "1.0.0"}
          }
        ]
      }
    });

    let findings = parse_cargo_audit_json(&example);
    assert_eq!(findings.len(), 1);
    let f = &findings[0];
    assert_eq!(f.ecosystem, "rust");
    assert_eq!(f.advisory_id.as_deref(), Some("RUSTSEC-2020-0001"));
    assert!(matches!(f.severity, Severity::High));
    assert_eq!(f.package.as_deref(), Some("example-crate"));
    assert_eq!(f.version.as_deref(), Some("1.0.0"));
  }

  #[test]
  fn parse_js_advisories_example() {
    let example = json!({
      "advisories": {
        "100": {
          "id": 100,
          "title": "js vuln",
          "url": "https://example",
          "module_name": "left-pad",
          "severity": "high"
        }
      }
    });

    let findings = parse_js_audit_json(&example);
    assert_eq!(findings.len(), 1);
    let f = &findings[0];
    assert_eq!(f.ecosystem, "js");
    assert_eq!(f.package.as_deref(), Some("left-pad"));
    assert!(matches!(f.severity, Severity::High));
    assert_eq!(f.title.as_deref(), Some("js vuln"));
  }
}

fn run_cargo_audit(install_if_missing: bool) -> Result<(Vec<Finding>, Option<String>)> {
  // Determine working dir for cargo-audit: prefer `src-tauri` if present
  let workdir = if Path::new("src-tauri").is_dir() {
    "src-tauri"
  } else {
    "."
  };
  let lockfile_path = Path::new(workdir).join("Cargo.lock");
  let lockfile_opt = if lockfile_path.exists() {
    Some(lockfile_path.to_string_lossy().to_string())
  } else {
    None
  };

  let out = Command::new("cargo")
    .arg("audit")
    .arg("--json")
    .current_dir(workdir)
    .output();

  let output = match out {
    Ok(o) => o,
    Err(e) => {
      if install_if_missing {
        eprintln!("`cargo audit` not found; attempting to install via `cargo install cargo-audit`");
        let install = Command::new("cargo")
          .arg("install")
          .arg("cargo-audit")
          .status();
        match install {
          Ok(s) if s.success() => Command::new("cargo")
            .arg("audit")
            .arg("--json")
            .current_dir(workdir)
            .output()
            .with_context(|| "failed to run `cargo audit` after install")?,
          _ => {
            eprintln!("failed to install `cargo-audit`. Please install it manually with `cargo install cargo-audit`");
            audit_exit(EXIT_EXEC_FAILURE, Some("failed to install cargo-audit"));
          }
        }
      } else {
        eprintln!("`cargo audit` invocation failed: {}. Install it with `cargo install cargo-audit` or pass --install-tools.", e);
        audit_exit(EXIT_EXEC_FAILURE, Some("`cargo audit` invocation failed"));
      }
    }
  };

  // If command failed but printed JSON to stdout, accept it; otherwise treat as execution failure
  if !output.status.success() {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stdout.trim().is_empty() {
      eprintln!("`cargo audit` failed: {}", stderr.trim());
      audit_exit(
        EXIT_EXEC_FAILURE,
        Some("`cargo audit` failed and produced no JSON"),
      );
    }
  }

  let json: JsonValue = match serde_json::from_slice(&output.stdout) {
    Ok(j) => j,
    Err(_) => {
      eprintln!("failed to parse `cargo audit` JSON output");
      audit_exit(EXIT_EXEC_FAILURE, Some("failed to parse cargo audit JSON"));
    }
  };

  Ok((parse_cargo_audit_json(&json), lockfile_opt))
}

fn parse_cargo_audit_json(json: &JsonValue) -> Vec<Finding> {
  let mut findings = Vec::new();
  if let Some(vulns) = json.get("vulnerabilities") {
    if let Some(list) = vulns.get("list").and_then(|l| l.as_array()) {
      for item in list {
        let advisory = item.get("advisory").unwrap_or(&JsonValue::Null);
        let id = advisory
          .get("id")
          .and_then(|v| v.as_str())
          .map(|s| s.to_string())
          .or_else(|| {
            item
              .get("id")
              .and_then(|v| v.as_str())
              .map(|s| s.to_string())
          });
        let title = advisory
          .get("title")
          .and_then(|v| v.as_str())
          .map(|s| s.to_string());
        let url = advisory
          .get("url")
          .and_then(|v| v.as_str())
          .map(|s| s.to_string());
        let package = advisory
          .get("package")
          .and_then(|v| v.as_str())
          .map(|s| s.to_string())
          .or_else(|| {
            item
              .get("package")
              .and_then(|p| p.get("name"))
              .and_then(|n| n.as_str())
              .map(|s| s.to_string())
          });
        let version = item
          .get("package")
          .and_then(|p| p.get("version"))
          .and_then(|v| v.as_str())
          .map(|s| s.to_string());

        let cvss = advisory.get("cvss").and_then(|v| v.as_f64());
        let severity = Severity::from_cvss(cvss);

        let fix_available = advisory
          .get("patched_versions")
          .map(|v| !v.as_str().unwrap_or("").is_empty())
          .or_else(|| {
            item
              .get("solution")
              .map(|s| !s.as_str().unwrap_or("").is_empty())
          })
          .or(Some(false));

        findings.push(Finding {
          ecosystem: "rust".into(),
          package,
          version,
          advisory_id: id,
          severity,
          title,
          url,
          fix_available,
        });
      }
    }
  }

  findings
}

fn run_js_audit(install_if_missing: bool) -> Result<(Vec<Finding>, Option<String>)> {
  // Detect package manager by lockfile in current dir
  let pnpm_lock = Path::new("pnpm-lock.yaml");
  let npm_lock = Path::new("package-lock.json");
  let yarn_lock = Path::new("yarn.lock");

  if pnpm_lock.exists() && which("pnpm").is_ok() {
    let output = Command::new("pnpm").arg("audit").arg("--json").output();
    let output = match output {
      Ok(o) => o,
      Err(_e) => {
        if install_if_missing {
          eprintln!("pnpm not found; please install pnpm");
          audit_exit(EXIT_EXEC_FAILURE, Some("pnpm not available"));
        } else {
          return Ok((Vec::new(), Some("pnpm-lock.yaml".into())));
        }
      }
    };
    if !output.status.success() && String::from_utf8_lossy(&output.stdout).trim().is_empty() {
      eprintln!(
        "pnpm audit failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
      );
      audit_exit(
        EXIT_EXEC_FAILURE,
        Some("pnpm audit failed and produced no JSON"),
      );
    }
    let json: JsonValue = match serde_json::from_slice(&output.stdout) {
      Ok(j) => j,
      Err(_) => {
        eprintln!("failed to parse pnpm audit JSON output");
        audit_exit(EXIT_EXEC_FAILURE, Some("failed to parse pnpm audit JSON"));
      }
    };
    return Ok((parse_js_audit_json(&json), Some("pnpm-lock.yaml".into())));
  }

  if npm_lock.exists() && which("npm").is_ok() {
    let output = Command::new("npm").arg("audit").arg("--json").output();
    let output = match output {
      Ok(o) => o,
      Err(_e) => {
        if install_if_missing {
          eprintln!("npm not found; please install npm");
          audit_exit(EXIT_EXEC_FAILURE, Some("npm not available"));
        } else {
          return Ok((Vec::new(), Some("package-lock.json".into())));
        }
      }
    };
    if !output.status.success() && String::from_utf8_lossy(&output.stdout).trim().is_empty() {
      eprintln!(
        "npm audit failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
      );
      audit_exit(
        EXIT_EXEC_FAILURE,
        Some("npm audit failed and produced no JSON"),
      );
    }
    let json: JsonValue = match serde_json::from_slice(&output.stdout) {
      Ok(j) => j,
      Err(_) => {
        eprintln!("failed to parse npm audit JSON output");
        audit_exit(EXIT_EXEC_FAILURE, Some("failed to parse npm audit JSON"));
      }
    };
    return Ok((parse_js_audit_json(&json), Some("package-lock.json".into())));
  }

  if yarn_lock.exists() && which("yarn").is_ok() {
    // check yarn version
    let ver_out = Command::new("yarn").arg("--version").output();
    if let Ok(vout) = ver_out {
      let vstr = String::from_utf8_lossy(&vout.stdout);
      if yarn_is_classic_from_version(&vstr) {
        eprintln!(
          "Yarn classic (v1.x) detected; `yarn audit` classic output is unsupported by this MVP."
        );
        return Ok((Vec::new(), Some("yarn.lock".into())));
      }
    }

    // modern yarn: use `yarn npm audit --json`
    let out = Command::new("yarn")
      .arg("npm")
      .arg("audit")
      .arg("--json")
      .output();
    let output = match out {
      Ok(o) => o,
      Err(e) => {
        eprintln!("failed to run yarn npm audit: {}", e);
        audit_exit(EXIT_EXEC_FAILURE, Some("failed to run yarn audit"));
      }
    };
    if !output.status.success() && String::from_utf8_lossy(&output.stdout).trim().is_empty() {
      eprintln!(
        "yarn npm audit failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
      );
      audit_exit(
        EXIT_EXEC_FAILURE,
        Some("yarn npm audit failed and produced no JSON"),
      );
    }
    let json: JsonValue = match serde_json::from_slice(&output.stdout) {
      Ok(j) => j,
      Err(_) => {
        eprintln!("failed to parse yarn audit JSON output");
        audit_exit(EXIT_EXEC_FAILURE, Some("failed to parse yarn audit JSON"));
      }
    };
    return Ok((parse_js_audit_json(&json), Some("yarn.lock".into())));
  }

  Ok((Vec::new(), None))
}

fn parse_js_audit_json(json: &JsonValue) -> Vec<Finding> {
  let mut findings = Vec::new();
  if let Some(advisories) = json.get("advisories") {
    if let Some(map) = advisories.as_object() {
      for (_id, adv) in map.iter() {
        let id = adv
          .get("id")
          .and_then(|v| v.as_i64())
          .map(|n| n.to_string());
        let title = adv
          .get("title")
          .and_then(|v| v.as_str())
          .map(|s| s.to_string());
        let url = adv
          .get("url")
          .and_then(|v| v.as_str())
          .map(|s| s.to_string());
        let module_name = adv
          .get("module_name")
          .and_then(|v| v.as_str())
          .map(|s| s.to_string());
        let severity = adv
          .get("severity")
          .and_then(|v| v.as_str())
          .map(|s| match s {
            "low" => Severity::Low,
            "moderate" => Severity::Medium,
            "high" => Severity::High,
            "critical" => Severity::Critical,
            _ => Severity::Unknown,
          })
          .unwrap_or(Severity::Unknown);

        findings.push(Finding {
          ecosystem: "js".into(),
          package: module_name,
          version: None,
          advisory_id: id,
          severity,
          title,
          url,
          fix_available: None,
        });
      }
    }
  } else if let Some(vulns) = json.get("vulnerabilities") {
    if let Some(obj) = vulns.as_object() {
      for (pkg, detail) in obj.iter() {
        let severity = detail
          .get("severity")
          .and_then(|v| v.as_str())
          .map(|s| match s {
            "low" => Severity::Low,
            "moderate" => Severity::Medium,
            "high" => Severity::High,
            "critical" => Severity::Critical,
            _ => Severity::Unknown,
          })
          .unwrap_or(Severity::Unknown);
        let title = detail
          .get("title")
          .and_then(|v| v.as_str())
          .map(|s| s.to_string());
        let url = detail
          .get("url")
          .and_then(|v| v.as_str())
          .map(|s| s.to_string());

        findings.push(Finding {
          ecosystem: "js".into(),
          package: Some(pkg.clone()),
          version: None,
          advisory_id: None,
          severity,
          title,
          url,
          fix_available: None,
        });
      }
    }
  }

  findings
}
