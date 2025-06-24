// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Schema generation for ACL items.

use std::{
  collections::{btree_map::Values, BTreeMap},
  fs,
  path::{Path, PathBuf},
  slice::Iter,
};

use schemars::{json_schema, Schema};
use serde_json::json;

use super::{Error, PERMISSION_SCHEMAS_FOLDER_NAME};
use crate::{platform::Target, write_if_changed};

use super::{
  capability::CapabilityFile,
  manifest::{Manifest, PermissionFile},
  Permission, PermissionSet, PERMISSION_SCHEMA_FILE_NAME,
};

/// Capability schema file name.
pub const CAPABILITIES_SCHEMA_FILE_NAME: &str = "schema.json";
/// Path of the folder where schemas are saved.
pub const CAPABILITIES_SCHEMA_FOLDER_PATH: &str = "gen/schemas";

// TODO: once MSRV is high enough, remove generic and use impl <trait>
// see https://github.com/tauri-apps/tauri/commit/b5561d74aee431f93c0c5b0fa6784fc0a956effe#diff-7c31d393f83cae149122e74ad44ac98e7d70ffb45c9e5b0a94ec52881b6f1cebR30-R42
/// Permission schema generator trait
pub trait PermissionSchemaGenerator<
  'a,
  Ps: Iterator<Item = &'a PermissionSet>,
  P: Iterator<Item = &'a Permission>,
>
{
  /// Whether has a default permission set or not.
  fn has_default_permission_set(&self) -> bool;

  /// Default permission set description if any.
  fn default_set_description(&self) -> Option<&str>;

  /// Default permission set's permissions if any.
  fn default_set_permissions(&self) -> Option<&Vec<String>>;

  /// Permissions sets to generate schema for.
  fn permission_sets(&'a self) -> Ps;

  /// Permissions to generate schema for.
  fn permissions(&'a self) -> P;

  /// A utility function to generate a schema for a permission identifier
  fn perm_id_schema(name: Option<&str>, id: &str, description: Option<&str>) -> Schema {
    let command_name = match name {
      Some(name) if name == super::APP_ACL_KEY => id.to_string(),
      Some(name) => format!("{name}:{id}"),
      _ => id.to_string(),
    };

    json_schema!({
      "type": "string",
      "const": command_name,
      "description": description,
      "markdownDescription": description
    })
  }

  /// Generate schemas for all possible permissions.
  fn gen_possible_permission_schemas(&'a self, name: Option<&str>) -> Vec<Schema> {
    let mut permission_schemas = Vec::new();

    // schema for default set
    if self.has_default_permission_set() {
      let description = self.default_set_description().unwrap_or_default();
      let description = if let Some(permissions) = self.default_set_permissions() {
        add_permissions_to_description(description, permissions, true)
      } else {
        description.to_string()
      };
      if !description.is_empty() {
        let default = Self::perm_id_schema(name, "default", Some(&description));
        permission_schemas.push(default);
      }
    }

    // schema for each permission set
    for set in self.permission_sets() {
      let description = add_permissions_to_description(&set.description, &set.permissions, false);
      let schema = Self::perm_id_schema(name, &set.identifier, Some(&description));
      permission_schemas.push(schema);
    }

    // schema for each permission
    for perm in self.permissions() {
      let schema = Self::perm_id_schema(name, &perm.identifier, perm.description.as_deref());
      permission_schemas.push(schema);
    }

    permission_schemas
  }
}

fn add_permissions_to_description(
  description: &str,
  permissions: &[String],
  is_default: bool,
) -> String {
  if permissions.is_empty() {
    return description.to_string();
  }
  let permissions_list = permissions
    .iter()
    .map(|permission| format!("- `{permission}`"))
    .collect::<Vec<_>>()
    .join("\n");
  let default_permission_set = if is_default {
    "default permission set"
  } else {
    "permission set"
  };
  format!("{description}\n#### This {default_permission_set} includes:\n\n{permissions_list}")
}

impl<'a>
  PermissionSchemaGenerator<
    'a,
    Values<'a, std::string::String, PermissionSet>,
    Values<'a, std::string::String, Permission>,
  > for Manifest
{
  fn has_default_permission_set(&self) -> bool {
    self.default_permission.is_some()
  }

  fn default_set_description(&self) -> Option<&str> {
    self
      .default_permission
      .as_ref()
      .map(|d| d.description.as_str())
  }

  fn default_set_permissions(&self) -> Option<&Vec<String>> {
    self.default_permission.as_ref().map(|d| &d.permissions)
  }

  fn permission_sets(&'a self) -> Values<'a, std::string::String, PermissionSet> {
    self.permission_sets.values()
  }

  fn permissions(&'a self) -> Values<'a, std::string::String, Permission> {
    self.permissions.values()
  }
}

impl<'a> PermissionSchemaGenerator<'a, Iter<'a, PermissionSet>, Iter<'a, Permission>>
  for PermissionFile
{
  fn has_default_permission_set(&self) -> bool {
    self.default.is_some()
  }

  fn default_set_description(&self) -> Option<&str> {
    self.default.as_ref().and_then(|d| d.description.as_deref())
  }

  fn default_set_permissions(&self) -> Option<&Vec<String>> {
    self.default.as_ref().map(|d| &d.permissions)
  }

  fn permission_sets(&'a self) -> Iter<'a, PermissionSet> {
    self.set.iter()
  }

  fn permissions(&'a self) -> Iter<'a, Permission> {
    self.permission.iter()
  }
}

/// Collect and include all possible identifiers in `Identifier` defintion in the schema
fn extend_identifier_schema(schema: &mut Schema, acl: &BTreeMap<String, Manifest>) {
  if let Some(serde_json::Value::Object(identifier_schema)) = schema.get_mut("Identifier") {
    let permission_schemas = acl
      .iter()
      .flat_map(|(name, manifest)| manifest.gen_possible_permission_schemas(Some(name)))
      .collect::<Vec<_>>();

    let new_subschemas = json_schema!({
      "oneOf": permission_schemas
    });

    identifier_schema.insert("subschemas".to_owned(), new_subschemas.into());
    identifier_schema.insert("object".to_owned(), serde_json::Value::Null);
    identifier_schema.insert("instance_type".to_owned(), serde_json::Value::Null);
    identifier_schema.insert("description".to_owned(), "Permission identifier".into());
  }
}

/// Collect permission schemas and its associated scope schema and schema definitons from plugins
/// and replace `PermissionEntry` extend object syntax with a new schema that does conditional
/// checks to serve the relavent scope schema for the right permissions schema, in a nutshell, it
/// will look something like this:
/// ```text
/// PermissionEntry {
///   anyOf {
///     String,  // default string syntax
///     Object { // extended object syntax
///       allOf { // JSON allOf is used but actually means anyOf
///         {
///           "if": "identifier" property anyOf "fs" plugin permission,
///           "then": add "allow" and "deny" properties that match "fs" plugin scope schema
///         },
///         {
///           "if": "identifier" property anyOf "http" plugin permission,
///           "then": add "allow" and "deny" properties that match "http" plugin scope schema
///         },
///         ...etc,
///         {
///           No "if" or "then", just "allow" and "deny" properties with default "#/definitions/Value"
///         },
///       }
///     }
///   }
/// }
/// ```
fn extend_permission_entry_schema(root_schema: &mut Schema, acl: &BTreeMap<String, Manifest>) {
  const IDENTIFIER: &str = "identifier";
  const ALLOW: &str = "allow";
  const DENY: &str = "deny";

  let mut collected_defs = vec![];

  if let Some(serde_json::Value::Object(obj)) = root_schema
    .get_mut("definitions")
    .unwrap()
    .get_mut("PermissionEntry")
  {
    let any_of = obj.get_mut("anyOf").unwrap().as_array_mut().unwrap();
    let serde_json::Value::Object(extened_permission_entry) = any_of.last_mut().unwrap() else {
      unreachable!("PermissionsEntry should be an object not a boolean");
    };

    // remove default properties and save it to be added later as a fallback
    let default_properties = extened_permission_entry.remove("properties").unwrap();

    let defaut_identifier = default_properties.get(IDENTIFIER).cloned().unwrap();

    let mut all_of = vec![];

    let schemas = acl.iter().filter_map(|(name, manifest)| {
      manifest
        .global_scope_schema()
        .unwrap_or_else(|e| panic!("invalid JSON schema for plugin {name}: {e}"))
        .map(|s| (s, manifest.gen_possible_permission_schemas(Some(name))))
    });

    for ((scope_schema, defs), acl_perm_schema) in schemas {
      let if_schema = json_schema!({
        "properties": {
          IDENTIFIER: {
            "anyOf": acl_perm_schema
          }
        }
      });

      let then_schema = json_schema!({
        "properties": {
          ALLOW: scope_schema.clone(),
          DENY: scope_schema.clone(),
        }
      });

      all_of.push(json_schema!({
        "properties": {
          ALLOW: scope_schema.clone(),
          DENY: scope_schema.clone(),
          IDENTIFIER: defaut_identifier
        },
        "if": if_schema,
        "then": then_schema
      }));

      collected_defs.extend(defs);
    }

    // add back default properties as a fallback
    let default_obj = json_schema!({
      "properties": default_properties
    });
    all_of.push(default_obj);

    // replace extended PermissionEntry with the new schema
    extened_permission_entry.insert("allOf".to_owned(), all_of.into());
  }

  // extend root schema with definitions collected from plugins
  root_schema
    .get_mut("definitions")
    .unwrap()
    .as_object_mut()
    .unwrap()
    .extend(collected_defs);
}

/// Generate schema for CapabilityFile with all possible plugins permissions
pub fn generate_capability_schema(
  acl: &BTreeMap<String, Manifest>,
  target: Target,
) -> crate::Result<()> {
  let mut schema = schemars::schema_for!(CapabilityFile);

  extend_identifier_schema(&mut schema, acl);
  extend_permission_entry_schema(&mut schema, acl);

  let schema_str = serde_json::to_string_pretty(&schema).unwrap();
  let out_dir = PathBuf::from(CAPABILITIES_SCHEMA_FOLDER_PATH);
  fs::create_dir_all(&out_dir)?;

  let schema_path = out_dir.join(format!("{target}-{CAPABILITIES_SCHEMA_FILE_NAME}"));
  if schema_str != fs::read_to_string(&schema_path).unwrap_or_default() {
    fs::write(&schema_path, schema_str)?;

    fs::copy(
      schema_path,
      out_dir.join(format!(
        "{}-{CAPABILITIES_SCHEMA_FILE_NAME}",
        if target.is_desktop() {
          "desktop"
        } else {
          "mobile"
        }
      )),
    )?;
  }

  Ok(())
}

/// Extend schema with collected permissions from the passed [`PermissionFile`]s.
fn extend_permission_file_schema(schema: &mut Schema, permissions: &[PermissionFile]) {
  // collect possible permissions
  let permission_schemas: Vec<_> = permissions
    .iter()
    .flat_map(|p| p.gen_possible_permission_schemas(None))
    .collect();

  let definitions = schema
    .get_mut("definitions")
    .unwrap()
    .as_object_mut()
    .unwrap();
  if let Some(serde_json::Value::Object(obj)) = definitions.get_mut("PermissionSet") {
    let permissions_obj = obj
      .get_mut("permissions")
      .unwrap()
      .as_object_mut()
      .unwrap()
      .get_mut("permissions");
    if let Some(serde_json::Value::Object(permissions_obj)) = permissions_obj {
      // replace the permissions property schema object
      // from a mere string to a referecnce to `PermissionKind`
      permissions_obj.insert(
        "items".to_owned(),
        json!({
          "reference": "#/definitions/PermissionKind"
        }),
      );

      // add the new `PermissionKind` definition in the schema that
      // is a list of all possible permissions collected
      definitions.insert(
        "PermissionKind".into(),
        json!({
          "type": "string",
          "oneOf": permission_schemas
        }),
      );
    }
  }
}

/// Generate and write a schema based on the format of a [`PermissionFile`].
pub fn generate_permissions_schema<P: AsRef<Path>>(
  permissions: &[PermissionFile],
  out_dir: P,
) -> Result<(), Error> {
  let mut schema = schemars::schema_for!(PermissionFile);

  extend_permission_file_schema(&mut schema, permissions);

  let schema_str = serde_json::to_string_pretty(&schema)?;

  let out_dir = out_dir.as_ref().join(PERMISSION_SCHEMAS_FOLDER_NAME);
  fs::create_dir_all(&out_dir).map_err(|e| Error::CreateDir(e, out_dir.clone()))?;

  let schema_path = out_dir.join(PERMISSION_SCHEMA_FILE_NAME);
  write_if_changed(&schema_path, schema_str).map_err(|e| Error::WriteFile(e, schema_path))?;

  Ok(())
}
