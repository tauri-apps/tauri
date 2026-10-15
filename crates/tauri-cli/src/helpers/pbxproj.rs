// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::{BTreeMap, BTreeSet},
  fmt,
  path::{Path, PathBuf},
};

use crate::error::ErrorExt;

pub fn parse<P: AsRef<Path>>(path: P) -> crate::Result<Pbxproj> {
  let path = path.as_ref();
  let pbxproj =
    std::fs::read_to_string(path).fs_context("failed to read pbxproj file", path.to_path_buf())?;

  let mut proj = Pbxproj {
    path: path.to_owned(),
    raw_lines: pbxproj.split('\n').map(ToOwned::to_owned).collect(),
    xc_build_configuration: BTreeMap::new(),
    xc_configuration_list: BTreeMap::new(),
    additions: BTreeMap::new(),
    removed_lines: BTreeSet::new(),
    has_changes: false,
  };

  let mut state = State::Idle;

  let mut iter = proj.raw_lines.iter().enumerate();

  while let Some((line_number, line)) = iter.next() {
    match &state {
      State::Idle => {
        if line == "/* Begin XCBuildConfiguration section */" {
          state = State::XCBuildConfiguration;
        } else if line == "/* Begin XCConfigurationList section */" {
          state = State::XCConfigurationList;
        }
      }
      // XCBuildConfiguration
      State::XCBuildConfiguration => {
        if line == "/* End XCBuildConfiguration section */" {
          state = State::Idle;
        } else if let Some((_identation, token)) = split_at_identation(line) {
          let id: String = token.chars().take_while(|c| !c.is_whitespace()).collect();
          proj.xc_build_configuration.insert(
            id.clone(),
            XCBuildConfiguration {
              build_settings: Vec::new(),
              build_settings_end_line_number: None,
            },
          );
          state = State::XCBuildConfigurationObject { id };
        }
      }
      State::XCBuildConfigurationObject { id } => {
        if line.contains("buildSettings") {
          state = State::XCBuildConfigurationObjectBuildSettings { id: id.clone() };
        } else if split_at_identation(line).is_some_and(|(_ident, token)| token == "};") {
          state = State::XCBuildConfiguration;
        }
      }
      State::XCBuildConfigurationObjectBuildSettings { id } => {
        if let Some((identation, token)) = split_at_identation(line) {
          if token == "};" {
            proj
              .xc_build_configuration
              .get_mut(id)
              .unwrap()
              .build_settings_end_line_number = Some(line_number);
            state = State::XCBuildConfigurationObject { id: id.clone() };
          } else {
            let assignment = token.trim_end_matches(';');
            if let Some((key, value)) = assignment.split_once(" = ") {
              let mut end_line_number = line_number;
              // multiline value
              let value = if value == "(" {
                let mut value = value.to_string();
                for (next_line_number, next_line) in iter.by_ref() {
                  end_line_number = next_line_number;
                  value.push_str(next_line);
                  value.push('\n');

                  if let Some((_, token)) = split_at_identation(next_line)
                    && token == ");"
                  {
                    break;
                  }
                }
                value
              } else {
                value.trim().to_string()
              };

              proj
                .xc_build_configuration
                .get_mut(id)
                .unwrap()
                .build_settings
                .push(BuildSettings {
                  identation: identation.into(),
                  location: BuildSettingLocation::Existing {
                    line_number,
                    end_line_number,
                  },
                  key: key.trim().into(),
                  value,
                });
            }
          }
        }
      }
      // XCConfigurationList
      State::XCConfigurationList => {
        if line == "/* End XCConfigurationList section */" {
          state = State::Idle;
        } else if let Some((_identation, token)) = split_at_identation(line) {
          let Some((id, comment)) = token.split_once(' ') else {
            continue;
          };

          proj.xc_configuration_list.insert(
            id.to_string(),
            XCConfigurationList {
              comment: comment.trim_end_matches(" = {").to_string(),
              build_configurations: Vec::new(),
            },
          );
          state = State::XCConfigurationListObject { id: id.to_string() };
        }
      }
      State::XCConfigurationListObject { id } => {
        if line.contains("buildConfigurations") {
          state = State::XCConfigurationListObjectBuildConfigurations { id: id.clone() };
        } else if split_at_identation(line).is_some_and(|(_ident, token)| token == "};") {
          state = State::XCConfigurationList;
        }
      }
      State::XCConfigurationListObjectBuildConfigurations { id } => {
        if let Some((_identation, token)) = split_at_identation(line) {
          if token == ");" {
            state = State::XCConfigurationListObject { id: id.clone() };
          } else {
            let Some((build_configuration_id, comments)) = token.split_once(' ') else {
              continue;
            };
            proj
              .xc_configuration_list
              .get_mut(id)
              .unwrap()
              .build_configurations
              .push(BuildConfigurationRef {
                id: build_configuration_id.to_string(),
                comments: comments.trim_end_matches(',').to_string(),
              });
          }
        }
      }
    }
  }

  Ok(proj)
}

/// Quotes a value so it can be used as a string in a pbxproj file.
pub fn quote(value: &str) -> String {
  format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Reverts [`quote`], returning the raw value of a (possibly quoted) pbxproj string.
pub fn unquote(value: &str) -> String {
  let Some(inner) = value
    .strip_prefix('"')
    .and_then(|value| value.strip_suffix('"'))
  else {
    return value.to_string();
  };

  let mut unquoted = String::with_capacity(inner.len());
  let mut chars = inner.chars();
  while let Some(c) = chars.next() {
    if c == '\\' {
      if let Some(next) = chars.next() {
        unquoted.push(next);
      }
    } else {
      unquoted.push(c);
    }
  }
  unquoted
}

fn split_at_identation(s: &str) -> Option<(&str, &str)> {
  s.chars()
    .position(|c| !c.is_ascii_whitespace())
    .map(|pos| s.split_at(pos))
}

enum State {
  Idle,
  // XCBuildConfiguration
  XCBuildConfiguration,
  XCBuildConfigurationObject { id: String },
  XCBuildConfigurationObjectBuildSettings { id: String },
  // XCConfigurationList
  XCConfigurationList,
  XCConfigurationListObject { id: String },
  XCConfigurationListObjectBuildConfigurations { id: String },
}

pub struct Pbxproj {
  pub path: PathBuf,
  raw_lines: Vec<String>,
  pub xc_build_configuration: BTreeMap<String, XCBuildConfiguration>,
  pub xc_configuration_list: BTreeMap<String, XCConfigurationList>,

  // maps the line number to the lines to add before it, in order
  additions: BTreeMap<usize, Vec<String>>,
  // lines that were replaced by a new value (continuation of multiline values)
  removed_lines: BTreeSet<usize>,

  has_changes: bool,
}

impl fmt::Debug for Pbxproj {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Pbxproj")
      .field("xc_build_configuration", &self.xc_build_configuration)
      .field("xc_configuration_list", &self.xc_configuration_list)
      .finish()
  }
}

impl Pbxproj {
  pub fn has_changes(&self) -> bool {
    !self.additions.is_empty() || self.has_changes
  }

  fn serialize(&self) -> String {
    let mut proj = String::new();
    let last_line_number = self.raw_lines.len() - 1;

    for (number, line) in self.raw_lines.iter().enumerate() {
      if let Some(additions) = self.additions.get(&number) {
        for new in additions {
          proj.push_str(new);
          proj.push('\n');
        }
      }

      if self.removed_lines.contains(&number) {
        continue;
      }

      proj.push_str(line);
      if number != last_line_number {
        proj.push('\n');
      }
    }

    proj
  }

  pub fn save(&self) -> std::io::Result<()> {
    std::fs::write(&self.path, self.serialize())
  }

  pub fn set_build_settings(&mut self, build_configuration_id: &str, key: &str, value: &str) {
    let Some(build_configuration) = self.xc_build_configuration.get_mut(build_configuration_id)
    else {
      return;
    };

    if let Some(build_setting) = build_configuration
      .build_settings
      .iter_mut()
      .find(|s| s.key == key)
    {
      if build_setting.value != value {
        let new_line = format!("{}{key} = {value};", build_setting.identation);
        match build_setting.location {
          BuildSettingLocation::Existing {
            line_number,
            end_line_number,
          } => {
            let Some(line) = self.raw_lines.get_mut(line_number) else {
              return;
            };
            *line = new_line;
            // drop the remaining lines of a multiline value
            self.removed_lines.extend(line_number + 1..=end_line_number);
          }
          BuildSettingLocation::Added { anchor, index } => {
            let Some(line) = self
              .additions
              .get_mut(&anchor)
              .and_then(|additions| additions.get_mut(index))
            else {
              return;
            };
            *line = new_line;
          }
        }
        build_setting.value = value.to_string();
        self.has_changes = true;
      }
    } else {
      // new settings are added right before the buildSettings closing brace
      let Some(anchor) = build_configuration.build_settings_end_line_number else {
        return;
      };
      let identation = match build_configuration.build_settings.last() {
        Some(last_build_setting) => last_build_setting.identation.clone(),
        None => {
          let closing_brace = self.raw_lines.get(anchor).map(String::as_str);
          let closing_identation = closing_brace
            .and_then(split_at_identation)
            .map(|(identation, _)| identation)
            .unwrap_or_default();
          format!("{closing_identation}\t")
        }
      };

      let additions = self.additions.entry(anchor).or_default();
      additions.push(format!("{identation}{key} = {value};"));
      build_configuration.build_settings.push(BuildSettings {
        identation,
        location: BuildSettingLocation::Added {
          anchor,
          index: additions.len() - 1,
        },
        key: key.to_string(),
        value: value.to_string(),
      });
    }
  }
}

#[derive(Debug)]
pub struct XCBuildConfiguration {
  build_settings: Vec<BuildSettings>,
  // line number of the buildSettings closing brace
  build_settings_end_line_number: Option<usize>,
}

impl XCBuildConfiguration {
  pub fn get_build_setting(&self, key: &str) -> Option<&BuildSettings> {
    self.build_settings.iter().find(|s| s.key == key)
  }
}

#[derive(Debug, Clone)]
pub struct BuildSettings {
  identation: String,
  location: BuildSettingLocation,
  pub key: String,
  pub value: String,
}

#[derive(Debug, Clone, Copy)]
enum BuildSettingLocation {
  /// A setting present in the parsed file, spanning `line_number..=end_line_number`.
  Existing {
    line_number: usize,
    end_line_number: usize,
  },
  /// A setting added by [`Pbxproj::set_build_settings`], stored in `additions[anchor][index]`.
  Added { anchor: usize, index: usize },
}

#[derive(Debug, Clone)]
pub struct XCConfigurationList {
  pub comment: String,
  pub build_configurations: Vec<BuildConfigurationRef>,
}

#[derive(Debug, Clone)]
pub struct BuildConfigurationRef {
  pub id: String,
  pub comments: String,
}

#[cfg(test)]
mod tests {
  #[test]
  fn parse() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures_path = manifest_dir.join("tests").join("fixtures").join("pbxproj");

    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(fixtures_path.join("snapshots"));
    let _guard = settings.bind_to_scope();

    insta::assert_debug_snapshot!(
      "project.pbxproj",
      super::parse(fixtures_path.join("project.pbxproj")).expect("failed to parse pbxproj")
    );
  }

  #[test]
  fn modify() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures_path = manifest_dir.join("tests").join("fixtures").join("pbxproj");

    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(fixtures_path.join("snapshots"));
    let _guard = settings.bind_to_scope();

    let mut pbxproj =
      super::parse(fixtures_path.join("project.pbxproj")).expect("failed to parse pbxproj");

    pbxproj.set_build_settings(
      "DB_0E254D0FD84970B57F6410",
      "PRODUCT_NAME",
      "\"Tauri Test\"",
    );
    pbxproj.set_build_settings("DB_0E254D0FD84970B57F6410", "UNKNOWN", "9283j49238h");

    insta::assert_snapshot!("project-modified.pbxproj", pbxproj.serialize());
  }

  const RELEASE: &str = "DB_0E254D0FD84970B57F6410";

  fn fixture() -> super::Pbxproj {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    super::parse(
      manifest_dir
        .join("tests")
        .join("fixtures")
        .join("pbxproj")
        .join("project.pbxproj"),
    )
    .expect("failed to parse pbxproj")
  }

  /// Returns the trimmed lines of the buildSettings dictionary of the given build configuration.
  fn build_settings_lines(serialized: &str, build_configuration_id: &str) -> Vec<String> {
    serialized
      .lines()
      .skip_while(|l| !l.trim_start().starts_with(build_configuration_id))
      .skip_while(|l| !l.contains("buildSettings = {"))
      .skip(1)
      .take_while(|l| l.trim() != "};")
      .map(|l| l.trim().to_string())
      .collect()
  }

  #[test]
  fn add_multiple_settings() {
    let mut pbxproj = fixture();
    pbxproj.set_build_settings(RELEASE, "NEW_A", "a");
    pbxproj.set_build_settings(RELEASE, "NEW_B", "b");
    pbxproj.set_build_settings(RELEASE, "NEW_C", "c");

    let serialized = pbxproj.serialize();
    let lines = build_settings_lines(&serialized, RELEASE);
    assert_eq!(
      &lines[lines.len() - 4..],
      [
        "VALID_ARCHS = \"arm64\";",
        "NEW_A = a;",
        "NEW_B = b;",
        "NEW_C = c;"
      ]
    );
    // the new settings stay inside the buildSettings dictionary
    assert!(serialized.contains("\t\t\t\tNEW_C = c;\n\t\t\t};\n\t\t\tname = release;\n\t\t};"));
  }

  #[test]
  fn update_added_setting() {
    let mut pbxproj = fixture();
    pbxproj.set_build_settings(RELEASE, "NEW_A", "a");
    pbxproj.set_build_settings(RELEASE, "NEW_B", "b");
    pbxproj.set_build_settings(RELEASE, "NEW_A", "a2");

    let serialized = pbxproj.serialize();
    let lines = build_settings_lines(&serialized, RELEASE);
    assert_eq!(&lines[lines.len() - 2..], ["NEW_A = a2;", "NEW_B = b;"]);
    assert!(serialized.contains("\t\t\t\tNEW_B = b;\n\t\t\t};\n\t\t\tname = release;\n\t\t};"));
    assert_eq!(
      serialized.lines().count(),
      fixture().serialize().lines().count() + 2
    );
  }

  #[test]
  fn update_multiline_setting() {
    let mut pbxproj = fixture();
    pbxproj.set_build_settings(RELEASE, "ARCHS", "arm64");

    let serialized = pbxproj.serialize();
    let lines = build_settings_lines(&serialized, RELEASE);
    assert_eq!(lines[1], "ARCHS = arm64;");
    assert_eq!(lines[2], "ASSETCATALOG_COMPILER_APPICON_NAME = AppIcon;");
    assert_eq!(
      serialized.lines().count(),
      fixture().serialize().lines().count() - 2
    );
  }

  #[test]
  fn quote() {
    let product_name = r#"My "App" \ Name"#;
    let quoted = super::quote(product_name);
    assert_eq!(quoted, r#""My \"App\" \\ Name""#);
    assert_eq!(super::unquote(&quoted), product_name);

    let mut pbxproj = fixture();
    pbxproj.set_build_settings(RELEASE, "PRODUCT_NAME", &quoted);
    assert!(
      pbxproj
        .serialize()
        .contains(r#"PRODUCT_NAME = "My \"App\" \\ Name";"#)
    );
  }
}
