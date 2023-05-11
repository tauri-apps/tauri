// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{marker::PhantomData, sync::Arc};

use crate::assets::{Assets, EmbeddedAssets};

use self::isolation::Keys;

/// Handling the Tauri "Isolation" Pattern.
#[cfg(feature = "isolation")]
pub mod isolation;

/// An application pattern.
#[derive(Debug)]
pub enum Pattern<A: Assets = EmbeddedAssets> {
  /// The brownfield pattern.
  Brownfield(PhantomData<A>),
  /// Isolation pattern. Recommended for security purposes.
  #[cfg(feature = "isolation")]
  Isolation {
    /// The HTML served on `isolation://index.html`.
    assets: Arc<A>,

    /// The schema used for the isolation frames.
    schema: String,

    /// A random string used to ensure that the message went through the isolation frame.
    ///
    /// This should be regenerated at runtime.
    key: String,

    /// Cryptographically secure keys
    crypto_keys: Box<Keys>,
  },
}

impl<A: Assets> Clone for Pattern<A> {
  fn clone(&self) -> Self {
    match self {
      Self::Brownfield(a) => Self::Brownfield(*a),
      #[cfg(feature = "isolation")]
      Self::Isolation {
        assets,
        schema,
        key,
        crypto_keys,
      } => Self::Isolation {
        assets: assets.clone(),
        schema: schema.clone(),
        key: key.clone(),
        crypto_keys: crypto_keys.clone(),
      },
    }
  }
}
