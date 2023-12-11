use std::collections::BTreeMap;

/// A valid ACL number.
#[derive(Debug)]
pub enum Number {
  /// Represents an [`i64`].
  Int(i64),

  /// Represents a [`f64`].
  Float(f64),
}

impl From<i64> for Number {
  #[inline(always)]
  fn from(value: i64) -> Self {
    Self::Int(value)
  }
}

impl From<f64> for Number {
  #[inline(always)]
  fn from(value: f64) -> Self {
    Self::Float(value)
  }
}

/// All supported ACL values.
#[derive(Debug)]
pub enum Value {
  /// Represents a [`bool`].
  Bool(bool),

  /// Represents a valid ACL [`Number`].
  Number(Number),

  /// Represents a [`String`].
  String(String),

  /// Represents a list of other [`Value`]s.
  List(Vec<Value>),

  /// Represents a map of [`String`] keys to [`Value`]s.
  Map(BTreeMap<String, Value>),
}

impl From<bool> for Value {
  #[inline(always)]
  fn from(value: bool) -> Self {
    Self::Bool(value)
  }
}

impl<T: Into<Number>> From<T> for Value {
  #[inline(always)]
  fn from(value: T) -> Self {
    Self::Number(value.into())
  }
}

impl From<String> for Value {
  #[inline(always)]
  fn from(value: String) -> Self {
    Value::String(value)
  }
}

impl From<toml::Value> for Value {
  #[inline(always)]
  fn from(value: toml::Value) -> Self {
    use toml::Value as Toml;

    match value {
      Toml::String(s) => s.into(),
      Toml::Integer(i) => i.into(),
      Toml::Float(f) => f.into(),
      Toml::Boolean(b) => b.into(),
      Toml::Datetime(d) => d.to_string().into(),
      Toml::Array(a) => Value::List(a.into_iter().map(Value::from).collect()),
      Toml::Table(t) => Value::Map(t.into_iter().map(|(k, v)| (k, v.into())).collect()),
    }
  }
}
