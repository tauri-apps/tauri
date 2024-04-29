// Copyright 2018-2023 the Deno authors. All rights reserved. MIT license.
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// a modified version of https://github.com/denoland/deno/blob/0ae83847f498a2886ae32172e50fd5bdbab2f524/core/resources.rs#L220

pub(crate) mod plugin;

use std::{
  any::{type_name, Any, TypeId},
  borrow::Cow,
  collections::BTreeMap,
  sync::Arc,
};

/// Resources are Rust objects that are stored in [ResourceTable] and managed by tauri.
/// They are identified in JS by a numeric ID (the resource ID, or rid).
/// Resources can be created in commands. Resources can also be retrieved in commands by
/// their rid. Resources are thread-safe.
///
/// Resources are reference counted in Rust. This means that they can be
/// cloned and passed around. When the last reference is dropped, the resource
/// is automatically closed. As long as the resource exists in the resource
/// table, the reference count is at least 1.
pub trait Resource: Any + 'static + Send + Sync {
  /// Returns a string representation of the resource. The default implementation
  /// returns the Rust type name, but specific resource types may override this
  /// trait method.
  fn name(&self) -> Cow<'_, str> {
    type_name::<Self>().into()
  }

  /// Resources may implement the `close()` trait method if they need to do
  /// resource specific clean-ups, such as cancelling pending futures, after a
  /// resource has been removed from the resource table.
  fn close(self: Arc<Self>) {}
}

impl dyn Resource {
  #[inline(always)]
  fn is<T: Resource>(&self) -> bool {
    self.type_id() == TypeId::of::<T>()
  }

  #[inline(always)]
  pub(crate) fn downcast_arc<'a, T: Resource>(self: &'a Arc<Self>) -> Option<&'a Arc<T>> {
    if self.is::<T>() {
      // A resource is stored as `Arc<T>` in a BTreeMap
      // and is safe to cast to `Arc<T>` because of the runtime
      // check done in `self.is::<T>()`
      let ptr = self as *const Arc<_> as *const Arc<T>;
      Some(unsafe { &*ptr })
    } else {
      None
    }
  }
}

/// A `ResourceId` is an integer value referencing a resource. It could be
/// considered to be the tauri equivalent of a `file descriptor` in POSIX like
/// operating systems.
pub type ResourceId = u32;

/// Map-like data structure storing Tauri's resources (equivalent to file
/// descriptors).
///
/// Provides basic methods for element access. A resource can be of any type.
/// Different types of resources can be stored in the same map, and provided
/// with a name for description.
///
/// Each resource is identified through a _resource ID (rid)_, which acts as
/// the key in the map.
#[derive(Default)]
pub struct ResourceTable {
  index: BTreeMap<ResourceId, Arc<dyn Resource>>,
}

impl ResourceTable {
  fn new_random_rid() -> u32 {
    let mut bytes = [0_u8; 4];
    getrandom::getrandom(&mut bytes).expect("failed to get random bytes");
    u32::from_ne_bytes(bytes)
  }

  /// Inserts resource into the resource table, which takes ownership of it.
  ///
  /// The resource type is erased at runtime and must be statically known
  /// when retrieving it through `get()`.
  ///
  /// Returns a unique resource ID, which acts as a key for this resource.
  pub fn add<T: Resource>(&mut self, resource: T) -> ResourceId {
    self.add_arc(Arc::new(resource))
  }

  /// Inserts a `Arc`-wrapped resource into the resource table.
  ///
  /// The resource type is erased at runtime and must be statically known
  /// when retrieving it through `get()`.
  ///
  /// Returns a unique resource ID, which acts as a key for this resource.
  pub fn add_arc<T: Resource>(&mut self, resource: Arc<T>) -> ResourceId {
    let resource = resource as Arc<dyn Resource>;
    self.add_arc_dyn(resource)
  }

  /// Inserts a `Arc`-wrapped resource into the resource table.
  ///
  /// The resource type is erased at runtime and must be statically known
  /// when retrieving it through `get()`.
  ///
  /// Returns a unique resource ID, which acts as a key for this resource.
  pub fn add_arc_dyn(&mut self, resource: Arc<dyn Resource>) -> ResourceId {
    let rid = Self::new_random_rid();
    let removed_resource = self.index.insert(rid, resource);
    assert!(removed_resource.is_none());
    rid
  }

  /// Returns true if any resource with the given `rid` exists.
  pub fn has(&self, rid: ResourceId) -> bool {
    self.index.contains_key(&rid)
  }

  /// Returns a reference counted pointer to the resource of type `T` with the
  /// given `rid`. If `rid` is not present or has a type different than `T`,
  /// this function returns [`Error::BadResourceId`](crate::Error::BadResourceId).
  pub fn get<T: Resource>(&self, rid: ResourceId) -> crate::Result<Arc<T>> {
    self
      .index
      .get(&rid)
      .and_then(|rc| rc.downcast_arc::<T>())
      .cloned()
      .ok_or_else(|| crate::Error::BadResourceId(rid))
  }

  /// Returns a reference counted pointer to the resource of the given `rid`.
  /// If `rid` is not present, this function returns [`Error::BadResourceId`].
  pub fn get_any(&self, rid: ResourceId) -> crate::Result<Arc<dyn Resource>> {
    self
      .index
      .get(&rid)
      .ok_or_else(|| crate::Error::BadResourceId(rid))
      .cloned()
  }

  /// Replaces a resource with a new resource.
  ///
  /// Panics if the resource does not exist.
  pub fn replace<T: Resource>(&mut self, rid: ResourceId, resource: T) {
    let result = self
      .index
      .insert(rid, Arc::new(resource) as Arc<dyn Resource>);
    assert!(result.is_some());
  }

  /// Removes a resource of type `T` from the resource table and returns it.
  /// If a resource with the given `rid` exists but its type does not match `T`,
  /// it is not removed from the resource table. Note that the resource's
  /// `close()` method is *not* called.
  ///
  /// Also note that there might be a case where
  /// the returned `Arc<T>` is referenced by other variables. That is, we cannot
  /// assume that `Arc::strong_count(&returned_arc)` is always equal to 1 on success.
  /// In particular, be really careful when you want to extract the inner value of
  /// type `T` from `Arc<T>`.
  pub fn take<T: Resource>(&mut self, rid: ResourceId) -> crate::Result<Arc<T>> {
    let resource = self.get::<T>(rid)?;
    self.index.remove(&rid);
    Ok(resource)
  }

  /// Removes a resource from the resource table and returns it. Note that the
  /// resource's `close()` method is *not* called.
  ///
  /// Also note that there might be a
  /// case where the returned `Arc<T>` is referenced by other variables. That is,
  /// we cannot assume that `Arc::strong_count(&returned_arc)` is always equal to 1
  /// on success. In particular, be really careful when you want to extract the
  /// inner value of type `T` from `Arc<T>`.
  pub fn take_any(&mut self, rid: ResourceId) -> crate::Result<Arc<dyn Resource>> {
    self
      .index
      .remove(&rid)
      .ok_or_else(|| crate::Error::BadResourceId(rid))
  }

  /// Returns an iterator that yields a `(id, name)` pair for every resource
  /// that's currently in the resource table. This can be used for debugging
  /// purposes. Note that the order in
  /// which items appear is not specified.
  pub fn names(&self) -> impl Iterator<Item = (ResourceId, Cow<'_, str>)> {
    self
      .index
      .iter()
      .map(|(&id, resource)| (id, resource.name()))
  }

  /// Removes the resource with the given `rid` from the resource table. If the
  /// only reference to this resource existed in the resource table, this will
  /// cause the resource to be dropped. However, since resources are reference
  /// counted, therefore pending ops are not automatically cancelled. A resource
  /// may implement the `close()` method to perform clean-ups such as canceling
  /// ops.
  pub fn close(&mut self, rid: ResourceId) -> crate::Result<()> {
    self
      .index
      .remove(&rid)
      .ok_or_else(|| crate::Error::BadResourceId(rid))
      .map(|resource| resource.close())
  }

  /// Removes and frees all resources stored. Note that the
  /// resource's `close()` method is *not* called.
  pub(crate) fn clear(&mut self) {
    self.index.clear()
  }
}
