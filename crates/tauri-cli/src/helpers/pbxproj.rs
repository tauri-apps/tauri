// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::BTreeMap,
  fmt,
  path::{Path, PathBuf},
};

pub fn parse<P: AsRef<Path>>(path: P) -> crate::Result<Pbxproj> {
  let path = path.as_ref();
  let pbxproj = std::fs::read_to_string(path)?;

  let mut proj = Pbxproj {
    path: path.to_owned(),
    raw_lines: pbxproj.split('\n').map(ToOwned::to_owned).collect(),
    xc_build_configuration: BTreeMap::new(),
    xc_configuration_list: BTreeMap::new(),
    additions: BTreeMap::new(),
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
            },
          );
          state = State::XCBuildConfigurationObject { id };
        }
      }
      State::XCBuildConfigurationObject { id } => {
        if line.contains("buildSettings") {
          state = State::XCBuildConfigurationObjectBuildSettings { id: id.clone() };
        } else if split_at_identation(line).map_or(false, |(_ident, token)| token == "};") {
          state = State::XCBuildConfiguration;
        }
      }
      State::XCBuildConfigurationObjectBuildSettings { id } => {
        if let Some((identation, token)) = split_at_identation(line) {
          if token == "};" {
            state = State::XCBuildConfigurationObject { id: id.clone() };
          } else {
            let assignment = token.trim_end_matches(';');
            if let Some((key, value)) = assignment.split_once(" = ") {
              // multiline value
              let value = if value == "(" {
                let mut value = value.to_string();
                loop {
                  let Some((_next_line_number, next_line)) = iter.next() else {
                    break;
                  };

                  value.push_str(next_line);
                  value.push('\n');

                  if let Some((_, token)) = split_at_identation(next_line) {
                    if token == ");" {
                      break;
                    }
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
                  line_number,
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
        } else if split_at_identation(line).map_or(false, |(_ident, token)| token == "};") {
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
  path: PathBuf,
  raw_lines: Vec<String>,
  pub xc_build_configuration: BTreeMap<String, XCBuildConfiguration>,
  pub xc_configuration_list: BTreeMap<String, XCConfigurationList>,

  // maps the line number to the line to add
  additions: BTreeMap<usize, String>,

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
      if let Some(new) = self.additions.get(&number) {
        proj.push_str(new);
        proj.push('\n');
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
        let Some(line) = self.raw_lines.get_mut(build_setting.line_number) else {
          return;
        };

        *line = format!("{}{key} = {value};", build_setting.identation);
        self.has_changes = true;
      }
    } else {
      let Some(last_build_setting) = build_configuration.build_settings.last().cloned() else {
        return;
      };
      build_configuration.build_settings.push(BuildSettings {
        identation: last_build_setting.identation.clone(),
        line_number: last_build_setting.line_number + 1,
        key: key.to_string(),
        value: value.to_string(),
      });
      self.additions.insert(
        last_build_setting.line_number + 1,
        format!("{}{key} = {value};", last_build_setting.identation),
      );
    }
  }
}

#[derive(Debug)]
pub struct XCBuildConfiguration {
  build_settings: Vec<BuildSettings>,
}

impl XCBuildConfiguration {
  pub fn get_build_setting(&self, key: &str) -> Option<&BuildSettings> {
    self.build_settings.iter().find(|s| s.key == key)
  }
}

#[derive(Debug, Clone)]
pub struct BuildSettings {
  identation: String,
  line_number: usize,
  pub key: String,
  pub value: String,
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
}
