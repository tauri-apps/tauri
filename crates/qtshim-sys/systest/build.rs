// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Generates the checks that `qtshim-sys/src/lib.rs` matches `qtshim-sys/cpp/qtshim.h`.
//!
//! 1. `ctest` (the libc project's FFI checker): constant values, size/alignment of every struct
//!    and type alias, struct field offsets/sizes, by-value round trips through C, and that every
//!    declared function resolves to the C symbol. Output: `$OUT_DIR/ctest_main.rs` + a C library.
//! 2. ctest does not compare function *signatures* or field *types* (only addresses and sizes),
//!    so this script also translates every Rust declaration into a C++ type and compiles
//!    `std::is_same` checks against the header (`sigcheck.cpp`). The compiler output is saved
//!    for `tests/header.rs` instead of failing the build, so a mismatch shows up as a test
//!    failure and never breaks `cargo build --workspace`.
//! 3. Tables of the header's enum constants (values computed by the C compiler), its function
//!    names, and the Rust declarations, for the coverage and distinctness tests.

use std::{
  collections::BTreeSet,
  env,
  fmt::Write as _,
  fs,
  path::{Path, PathBuf},
};

use quote::ToTokens;

fn main() {
  let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
  let sys_dir = manifest.parent().unwrap().to_path_buf();
  let include = sys_dir.join("cpp");
  let header = include.join("qtshim.h");
  let sys_lib = sys_dir.join("src/lib.rs");
  let out = PathBuf::from(env::var("OUT_DIR").unwrap());
  let webengine = env::var_os("CARGO_FEATURE_WEBENGINE").is_some();

  println!("cargo:rerun-if-changed={}", header.display());
  println!("cargo:rerun-if-changed={}", sys_lib.display());
  println!("cargo:rerun-if-changed=build.rs");

  let cfg: Vec<(String, Option<String>)> = if webengine {
    vec![("feature".into(), Some("webengine".into()))]
  } else {
    vec![]
  };

  // --- 1. ctest -------------------------------------------------------------------------------
  let mut gen_ = ctest::TestGenerator::new();
  gen_
    .header("qtshim.h")
    .include(&include)
    .edition(2024)
    // Opaque handles (`struct { _private: [u8; 0] }`) are incomplete types in C: no size to
    // compare. Their existence is checked in sigcheck.cpp.
    .skip_struct(|s| is_opaque(s.ident()))
    // Function pointer aliases: their size/alignment is still compared, but a by-value round
    // trip would fill a fn pointer with a byte pattern, which is not a valid Rust value.
    .skip_roundtrip(|name| name.starts_with("qs_") && name.ends_with("_fn"));
  if webengine {
    gen_.cfg("feature", Some("webengine"));
  }
  // `ctest::generate_test` would also compile the C side, panicking (and failing the whole
  // workspace build) on a type mismatch it detects at compile time. Compile it here instead and
  // turn diagnostics into a failing test.
  let rs = gen_
    .generate_files(&sys_lib, "ctest_main.rs")
    .expect("ctest generation failed");
  let c_file = rs.with_extension("c");
  let mut c_build = cc::Build::new();
  c_build.file(&c_file).include(&include).flags(CTEST_C_FLAGS);
  let diagnostics = syntax_check(&c_build, &c_file);
  if diagnostics.is_empty() {
    c_build.compile("ctest_main");
  } else {
    let stub = format!(
      "fn main() {{\n  panic!(\"ctest's C side does not compile against qtshim.h:\\n{{}}\", {diagnostics:?});\n}}\n"
    );
    fs::write(&rs, stub).unwrap();
  }

  // --- parse both sides -----------------------------------------------------------------------
  let target = env::var("TARGET").unwrap();
  let expanded = ctest::expand(&sys_lib, &cfg, target).expect("expanding qtshim-sys failed");
  let rust = RustItems::parse(&expanded);
  let hdr = HeaderItems::parse(&fs::read_to_string(&header).unwrap());

  // --- 2. signature / field type checks -------------------------------------------------------
  let sigcheck = out.join("sigcheck.cpp");
  fs::write(&sigcheck, rust.sigcheck_source()).unwrap();
  let log = compile_sigcheck(&sigcheck, &include);
  let log = rust.problems.iter().fold(log, |mut acc, p| {
    writeln!(acc, "{p}").unwrap();
    acc
  });
  fs::write(out.join("sigcheck.log"), &log).unwrap();

  // --- 3. tables ------------------------------------------------------------------------------
  let all_consts: Vec<&String> = hdr.enums.iter().flatten().collect();
  let mut c = String::from("#include \"qtshim.h\"\n#include <stdint.h>\n");
  writeln!(c, "const int64_t qs_systest_header_consts[] = {{").unwrap();
  for name in &all_consts {
    writeln!(c, "  (int64_t){name},").unwrap();
  }
  writeln!(c, "  0\n}};").unwrap();
  let c_path = out.join("header_consts.c");
  fs::write(&c_path, c).unwrap();
  cc::Build::new()
    .file(&c_path)
    .include(&include)
    .warnings_into_errors(true)
    .compile("qtshim_systest_consts");

  let mut t = String::new();
  writeln!(
    t,
    "/// `QS_*` enumerators of `qtshim.h`, grouped by `enum {{ ... }}` block."
  )
  .unwrap();
  writeln!(t, "pub const HEADER_ENUMS: &[&[&str]] = &[").unwrap();
  for group in &hdr.enums {
    writeln!(t, "  &{group:?},").unwrap();
  }
  writeln!(t, "];").unwrap();
  writeln!(t, "/// Number of entries in `HEADER_ENUMS` (flattened).").unwrap();
  writeln!(
    t,
    "pub const HEADER_CONST_COUNT: usize = {};",
    all_consts.len()
  )
  .unwrap();
  writeln!(t, "/// Functions declared in `qtshim.h`.").unwrap();
  writeln!(t, "pub const HEADER_FNS: &[&str] = &{:?};", hdr.fns).unwrap();
  writeln!(t, "/// Callback typedefs declared in `qtshim.h`.").unwrap();
  writeln!(
    t,
    "pub const HEADER_CALLBACKS: &[&str] = &{:?};",
    hdr.callbacks
  )
  .unwrap();
  writeln!(
    t,
    "/// Functions declared in `qtshim-sys` (with this build's features)."
  )
  .unwrap();
  let rust_fns: Vec<&String> = rust.fns.iter().map(|f| &f.0).collect();
  writeln!(t, "pub const RUST_FNS: &[&str] = &{rust_fns:?};").unwrap();
  writeln!(t, "/// Function pointer aliases declared in `qtshim-sys`.").unwrap();
  let rust_cbs: Vec<&String> = rust.callbacks.iter().map(|f| &f.0).collect();
  writeln!(t, "pub const RUST_CALLBACKS: &[&str] = &{rust_cbs:?};").unwrap();
  writeln!(
    t,
    "/// Constants declared in `qtshim-sys`, with their Rust values."
  )
  .unwrap();
  writeln!(t, "pub const RUST_CONSTS: &[(&str, i64)] = &[").unwrap();
  for name in &rust.consts {
    writeln!(t, "  ({name:?}, qtshim_sys::{name} as i64),").unwrap();
  }
  writeln!(t, "];").unwrap();
  fs::write(out.join("tables.rs"), t).unwrap();
}

/// The flags `ctest::generate_test` compiles its C file with (it relies on `-Werror` to catch
/// e.g. field type mismatches in its field-pointer accessors).
const CTEST_C_FLAGS: &[&str] = &[
  "-Wall",
  "-Wextra",
  "-Werror",
  "-Wno-unused-parameter",
  "-Wno-type-limits",
  "-Wno-address-of-packed-member",
  "-Wno-unknown-warning-option",
  "-Wno-deprecated-declarations",
];

/// Compiles `file` with `build`'s compiler and flags in syntax-only mode; returns the
/// diagnostics if it fails.
fn syntax_check(build: &cc::Build, file: &Path) -> String {
  let output = build
    .get_compiler()
    .to_command()
    .args(["-fsyntax-only", "-fno-diagnostics-color"])
    .arg(file)
    .output()
    .expect("failed to run the C compiler");
  if output.status.success() {
    String::new()
  } else {
    String::from_utf8_lossy(&output.stderr).into_owned()
  }
}

/// Opaque handle types (incomplete in C).
fn is_opaque(name: &str) -> bool {
  matches!(
    name,
    "QsReply" | "QsWindow" | "QsView" | "QsProfile" | "QsJob"
  )
}

/// Runs the C++ compiler in syntax-only mode and returns its diagnostics (empty when all checks
/// pass).
fn compile_sigcheck(file: &Path, include: &Path) -> String {
  let compiler = cc::Build::new().cpp(true).get_compiler();
  let output = compiler
    .to_command()
    .args(["-std=c++17", "-fsyntax-only", "-fno-diagnostics-color"])
    .arg("-I")
    .arg(include)
    .arg(file)
    .output()
    .expect("failed to run the C++ compiler");
  let mut log = String::from_utf8_lossy(&output.stderr).into_owned();
  if !output.status.success() && log.is_empty() {
    log = format!("C++ compiler exited with {}", output.status);
  }
  log
}

// ---------------------------------------------------------------------------------------------
// Rust side

#[derive(Default)]
struct RustItems {
  /// (name, C++ function pointer type)
  fns: Vec<(String, String)>,
  /// (name, C++ function pointer type)
  callbacks: Vec<(String, String)>,
  /// (struct, field, C++ type)
  fields: Vec<(String, String, String)>,
  opaque: Vec<String>,
  consts: Vec<String>,
  /// Declarations that can't be expressed in C (wrong ABI, unknown types).
  problems: Vec<String>,
}

impl RustItems {
  fn parse(expanded: &str) -> Self {
    let file = syn::parse_file(expanded).expect("parsing expanded qtshim-sys failed");
    let mut items = Self::default();
    for item in &file.items {
      items.item(item);
    }
    items
  }

  fn item(&mut self, item: &syn::Item) {
    match item {
      syn::Item::Const(c) => self.consts.push(c.ident.to_string()),
      syn::Item::Type(t) => {
        let name = t.ident.to_string();
        match self.cty(&t.ty) {
          Ok(cty) => self.callbacks.push((name, cty)),
          Err(e) => self.problems.push(format!("type alias {name}: {e}")),
        }
      }
      syn::Item::Struct(s) => {
        let name = s.ident.to_string();
        if !has_repr_c(&s.attrs) {
          self
            .problems
            .push(format!("struct {name} is not #[repr(C)]"));
        }
        if is_opaque(&name) {
          self.opaque.push(name);
          return;
        }
        for field in &s.fields {
          let fname = field.ident.as_ref().unwrap().to_string();
          match self.cty(&field.ty) {
            Ok(cty) => self.fields.push((name.clone(), fname, cty)),
            Err(e) => self.problems.push(format!("{name}::{fname}: {e}")),
          }
        }
      }
      syn::Item::ForeignMod(m) => {
        if m.abi.name.as_ref().map(|n| n.value()) != Some("C".into()) {
          self
            .problems
            .push("extern block with a non-\"C\" ABI".into());
        }
        for item in &m.items {
          let syn::ForeignItem::Fn(f) = item else {
            self
              .problems
              .push("non-function item in an extern block".into());
            continue;
          };
          let name = f.sig.ident.to_string();
          let inputs = f.sig.inputs.iter().filter_map(|a| match a {
            syn::FnArg::Typed(p) => Some(&*p.ty),
            syn::FnArg::Receiver(_) => None,
          });
          match self.fn_ptr(inputs, &f.sig.output, f.sig.variadic.is_some()) {
            Ok(cty) => self.fns.push((name, cty)),
            Err(e) => self.problems.push(format!("fn {name}: {e}")),
          }
        }
      }
      _ => {}
    }
  }

  fn fn_ptr<'a, P>(
    &self,
    inputs: P,
    output: &syn::ReturnType,
    variadic: bool,
  ) -> Result<String, String>
  where
    P: IntoIterator<Item = &'a syn::Type>,
  {
    if variadic {
      return Err("variadic functions are not supported".into());
    }
    let ret = match output {
      syn::ReturnType::Default => "void".to_string(),
      syn::ReturnType::Type(_, ty) => self.cty(ty)?,
    };
    let mut args = vec![ret];
    for ty in inputs {
      args.push(self.cty(ty)?);
    }
    Ok(format!("qs_fnptr<{}>", args.join(", ")))
  }

  /// The C++ spelling of a Rust FFI type (using the helpers in `sigcheck_source`).
  fn cty(&self, ty: &syn::Type) -> Result<String, String> {
    Ok(match ty {
      syn::Type::Path(p) => {
        let seg = p.path.segments.last().unwrap();
        let ident = seg.ident.to_string();
        if ident == "Option" {
          // Option<fn> is a nullable function pointer: same ABI.
          let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
            return Err("bare Option".into());
          };
          let Some(syn::GenericArgument::Type(inner)) = args.args.first() else {
            return Err("Option without a type".into());
          };
          return self.cty(inner);
        }
        match ident.as_str() {
          "c_char" => "char",
          "c_int" => "int",
          "c_uint" => "unsigned int",
          "c_long" => "long",
          "c_ulong" => "unsigned long",
          "c_void" => "void",
          "c_double" => "double",
          "i8" => "int8_t",
          "u8" => "uint8_t",
          "i16" => "int16_t",
          "u16" => "uint16_t",
          "i32" => "int32_t",
          "u32" => "uint32_t",
          "i64" => "int64_t",
          "u64" => "uint64_t",
          "isize" => "ptrdiff_t",
          "usize" => "size_t",
          "f32" => "float",
          "f64" => "double",
          "bool" => "bool",
          other if other.starts_with("Qs") || other.starts_with("qs_") => other,
          other => return Err(format!("unknown type `{other}`")),
        }
        .to_string()
      }
      syn::Type::Ptr(p) => {
        let inner = self.cty(&p.elem)?;
        if p.const_token.is_some() {
          format!("{inner} const *")
        } else {
          format!("{inner} *")
        }
      }
      syn::Type::BareFn(f) => {
        if f
          .abi
          .as_ref()
          .and_then(|a| a.name.as_ref())
          .map(|n| n.value())
          != Some("C".into())
        {
          return Err("function pointer without extern \"C\"".into());
        }
        self.fn_ptr(
          f.inputs.iter().map(|a| &a.ty),
          &f.output,
          f.variadic.is_some(),
        )?
      }
      syn::Type::Array(a) => {
        let len = a.len.to_token_stream().to_string();
        format!("qs_arr<{}, {len}>", self.cty(&a.elem)?)
      }
      syn::Type::Tuple(t) if t.elems.is_empty() => "void".into(),
      syn::Type::Paren(p) => self.cty(&p.elem)?,
      other => return Err(format!("unsupported type `{}`", other.to_token_stream())),
    })
  }

  fn sigcheck_source(&self) -> String {
    let mut s = String::new();
    s.push_str(
      "// Generated by qtshim-sys-systest/build.rs: the Rust declarations of qtshim-sys as C++\n\
       // types, compared with the header's. A failure names both types in the instantiation.\n\
       #include \"qtshim.h\"\n\
       #include <stddef.h>\n\
       #include <stdint.h>\n\
       #include <type_traits>\n\n\
       template <class R, class... A> using qs_fnptr = R (*)(A...);\n\
       template <class T, size_t N> using qs_arr = T[N];\n\
       template <int Line, class Header, class Rust> struct Same {\n\
       \x20 static_assert(std::is_same<Header, Rust>::value, \"Rust declaration differs from qtshim.h\");\n\
       };\n\n",
    );
    for name in &self.opaque {
      writeln!(s, "// opaque {name}\nusing check_opaque_{name} = {name} *;").unwrap();
    }
    for (name, cty) in &self.callbacks {
      writeln!(
        s,
        "// callback {name}\ntemplate struct Same<__LINE__, {name}, {cty}>;"
      )
      .unwrap();
    }
    for (st, field, cty) in &self.fields {
      writeln!(
        s,
        "// field {st}::{field}\ntemplate struct Same<__LINE__, decltype({st}::{field}), {cty}>;"
      )
      .unwrap();
    }
    for (name, cty) in &self.fns {
      writeln!(
        s,
        "// fn {name}\ntemplate struct Same<__LINE__, decltype(&{name}), {cty}>;"
      )
      .unwrap();
    }
    s
  }
}

fn has_repr_c(attrs: &[syn::Attribute]) -> bool {
  attrs
    .iter()
    .any(|a| a.path().is_ident("repr") && a.meta.to_token_stream().to_string().contains('C'))
}

// ---------------------------------------------------------------------------------------------
// header side (a small parser for qtshim.h's flat declaration style)

struct HeaderItems {
  enums: Vec<Vec<String>>,
  fns: Vec<String>,
  callbacks: Vec<String>,
}

impl HeaderItems {
  fn parse(src: &str) -> Self {
    // Drop comments and preprocessor lines, then the `extern "C" {` wrapper.
    let mut text = String::new();
    for line in src.lines() {
      let line = line.split("//").next().unwrap();
      if line.trim_start().starts_with('#') {
        continue;
      }
      text.push_str(line);
      text.push('\n');
    }
    let text = text.replacen("extern \"C\" {", "", 1);
    let text = match text.rfind('}') {
      // the closing brace of `extern "C"` is the last one and is followed only by whitespace
      Some(i) if text[i + 1..].trim().is_empty() => text[..i].to_string(),
      _ => text,
    };

    let mut enums = Vec::new();
    let mut fns = Vec::new();
    let mut callbacks = Vec::new();
    for stmt in split_top_level(&text) {
      let stmt = stmt.trim();
      if stmt.starts_with("enum") {
        let body = &stmt[stmt.find('{').unwrap() + 1..stmt.rfind('}').unwrap()];
        let names = body
          .split(',')
          .filter_map(|e| e.split('=').next().map(str::trim))
          .filter(|n| !n.is_empty())
          .map(String::from)
          .collect();
        enums.push(names);
      } else if stmt.starts_with("typedef") {
        if let Some(i) = stmt.find("(*") {
          let rest = &stmt[i + 2..];
          callbacks.push(rest[..rest.find(')').unwrap()].trim().to_string());
        }
      } else if let Some(i) = stmt.find('(') {
        let before = stmt[..i].trim_end();
        let name = before
          .rsplit(|c: char| !(c.is_alphanumeric() || c == '_'))
          .next()
          .unwrap();
        fns.push(name.to_string());
      }
    }
    let unique: BTreeSet<_> = fns.iter().collect();
    assert_eq!(
      unique.len(),
      fns.len(),
      "duplicate function declaration in qtshim.h"
    );
    Self {
      enums,
      fns,
      callbacks,
    }
  }
}

/// Splits on `;` outside braces.
fn split_top_level(text: &str) -> Vec<String> {
  let mut out = Vec::new();
  let mut depth = 0i32;
  let mut cur = String::new();
  for ch in text.chars() {
    match ch {
      '{' => depth += 1,
      '}' => depth -= 1,
      ';' if depth == 0 => {
        out.push(std::mem::take(&mut cur));
        continue;
      }
      _ => {}
    }
    cur.push(ch);
  }
  out
}
