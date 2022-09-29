// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

macro_rules! bind_string_arg {
  ($arg:expr, $clap_arg:expr, $arg_name:ident, $clap_field:ident) => {{
    let arg = $arg;
    let mut clap_arg = $clap_arg;
    if let Some(value) = &arg.$arg_name {
      clap_arg = clap_arg.$clap_field(value);
    }
    clap_arg
  }};
}

macro_rules! bind_value_arg {
  ($arg:expr, $clap_arg:expr, $field:ident) => {{
    let arg = $arg;
    let mut clap_arg = $clap_arg;
    if let Some(value) = arg.$field {
      clap_arg = clap_arg.$field(value);
    }
    clap_arg
  }};
}

macro_rules! bind_string_slice_arg {
  ($arg:expr, $clap_arg:expr, $field:ident) => {{
    let arg = $arg;
    let mut clap_arg = $clap_arg;
    if let Some(value) = &arg.$field {
      clap_arg = clap_arg.$field(value);
    }
    clap_arg
  }};
}

macro_rules! bind_if_arg {
  ($arg:expr, $clap_arg:expr, $field:ident) => {{
    let arg = $arg;
    let mut clap_arg = $clap_arg;
    if let Some(value) = &arg.$field {
      clap_arg = clap_arg.$field(&value[0], &value[1]);
    }
    clap_arg
  }};
}
