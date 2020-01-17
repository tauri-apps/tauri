use std::borrow::Cow;

#[derive(Clone, PartialEq, Debug)]
pub struct LinkValue {
  /// Target IRI: `link-value`.
  link: Cow<'static, str>,

  /// Forward Relation Types: `rel`.
  rel: Option<Vec<RelationType>>,
}

impl LinkValue {
  pub fn new<T>(uri: T) -> LinkValue
  where
    T: Into<Cow<'static, str>>,
  {
    LinkValue {
      link: uri.into(),
      rel: None,
    }
  }

  pub fn rel(&self) -> Option<&[RelationType]> {
    self.rel.as_ref().map(AsRef::as_ref)
  }
}

#[derive(Clone, PartialEq, Debug)]
pub enum RelationType {
  /// next.
  Next,
  /// ext-rel-type.
  ExtRelType(String),
}
