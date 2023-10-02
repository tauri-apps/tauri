// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(target_os = "macos")]
#![allow(deprecated)]

use crate::utils::config::WindowEffectsConfig;
use crate::window::{Effect, EffectState};
use cocoa::{
  appkit::{
    NSAppKitVersionNumber, NSAppKitVersionNumber10_10, NSAppKitVersionNumber10_11,
    NSAutoresizingMaskOptions, NSView, NSViewHeightSizable, NSViewWidthSizable, NSWindow,
    NSWindowOrderingMode,
  },
  base::{id, nil, BOOL},
  foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize},
};
use objc::{class, msg_send, sel, sel_impl};

pub fn apply_effects(window: id, effects: WindowEffectsConfig) {
  let WindowEffectsConfig {
    effects,
    radius,
    state,
    ..
  } = effects;
  let mut appearance: NSVisualEffectMaterial = if let Some(effect) = effects.into_iter().find(|e| {
    matches!(
      e,
      Effect::AppearanceBased
        | Effect::Light
        | Effect::Dark
        | Effect::MediumLight
        | Effect::UltraDark
        | Effect::Titlebar
        | Effect::Selection
        | Effect::Menu
        | Effect::Popover
        | Effect::Sidebar
        | Effect::HeaderView
        | Effect::Sheet
        | Effect::WindowBackground
        | Effect::HudWindow
        | Effect::FullScreenUI
        | Effect::Tooltip
        | Effect::ContentBackground
        | Effect::UnderWindowBackground
        | Effect::UnderPageBackground
    )
  }) {
    effect.into()
  } else {
    return;
  };

  unsafe {
    if NSAppKitVersionNumber < NSAppKitVersionNumber10_10 {
      return;
    }

    if !msg_send![class!(NSThread), isMainThread] {
      return;
    }

    if appearance as u32 > 4 && NSAppKitVersionNumber < NSAppKitVersionNumber10_11 {
      appearance = NSVisualEffectMaterial::AppearanceBased;
    }

    if appearance as u32 > 9 && NSAppKitVersionNumber < NSAppKitVersionNumber10_14 {
      appearance = NSVisualEffectMaterial::AppearanceBased;
    }

    let ns_view: id = window.contentView();
    let bounds = NSView::bounds(ns_view);

    let blurred_view = NSVisualEffectView::initWithFrame_(NSVisualEffectView::alloc(nil), bounds);
    blurred_view.autorelease();

    blurred_view.setMaterial_(appearance);
    blurred_view.setCornerRadius_(radius.unwrap_or(0.0));
    blurred_view.setBlendingMode_(NSVisualEffectBlendingMode::BehindWindow);
    blurred_view.setState_(
      state
        .map(Into::into)
        .unwrap_or(NSVisualEffectState::FollowsWindowActiveState),
    );
    NSVisualEffectView::setAutoresizingMask_(
      blurred_view,
      NSViewWidthSizable | NSViewHeightSizable,
    );

    let _: () = msg_send![ns_view, addSubview: blurred_view positioned: NSWindowOrderingMode::NSWindowBelow relativeTo: 0];
  }
}

#[allow(non_upper_case_globals)]
const NSAppKitVersionNumber10_14: f64 = 1671.0;

// https://developer.apple.com/documentation/appkit/nsvisualeffectview/blendingmode
#[allow(dead_code)]
#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum NSVisualEffectBlendingMode {
  BehindWindow = 0,
  WithinWindow = 1,
}

// macos 10.10+
// https://developer.apple.com/documentation/appkit/nsvisualeffectview
#[allow(non_snake_case)]
trait NSVisualEffectView: Sized {
  unsafe fn alloc(_: Self) -> id {
    msg_send![class!(NSVisualEffectView), alloc]
  }

  unsafe fn init(self) -> id;
  unsafe fn initWithFrame_(self, frameRect: NSRect) -> id;
  unsafe fn bounds(self) -> NSRect;
  unsafe fn frame(self) -> NSRect;
  unsafe fn setFrameSize(self, frameSize: NSSize);
  unsafe fn setFrameOrigin(self, frameOrigin: NSPoint);

  unsafe fn superview(self) -> id;
  unsafe fn removeFromSuperview(self);
  unsafe fn setAutoresizingMask_(self, autoresizingMask: NSAutoresizingMaskOptions);

  // API_AVAILABLE(macos(10.12));
  unsafe fn isEmphasized(self) -> BOOL;
  // API_AVAILABLE(macos(10.12));
  unsafe fn setEmphasized_(self, emphasized: BOOL);

  unsafe fn setMaterial_(self, material: NSVisualEffectMaterial);
  unsafe fn setCornerRadius_(self, radius: f64);
  unsafe fn setState_(self, state: NSVisualEffectState);
  unsafe fn setBlendingMode_(self, mode: NSVisualEffectBlendingMode);
}

#[allow(non_snake_case)]
impl NSVisualEffectView for id {
  unsafe fn init(self) -> id {
    msg_send![self, init]
  }

  unsafe fn initWithFrame_(self, frameRect: NSRect) -> id {
    msg_send![self, initWithFrame: frameRect]
  }

  unsafe fn bounds(self) -> NSRect {
    msg_send![self, bounds]
  }

  unsafe fn frame(self) -> NSRect {
    msg_send![self, frame]
  }

  unsafe fn setFrameSize(self, frameSize: NSSize) {
    msg_send![self, setFrameSize: frameSize]
  }

  unsafe fn setFrameOrigin(self, frameOrigin: NSPoint) {
    msg_send![self, setFrameOrigin: frameOrigin]
  }

  unsafe fn superview(self) -> id {
    msg_send![self, superview]
  }

  unsafe fn removeFromSuperview(self) {
    msg_send![self, removeFromSuperview]
  }

  unsafe fn setAutoresizingMask_(self, autoresizingMask: NSAutoresizingMaskOptions) {
    msg_send![self, setAutoresizingMask: autoresizingMask]
  }

  // API_AVAILABLE(macos(10.12));
  unsafe fn isEmphasized(self) -> BOOL {
    msg_send![self, isEmphasized]
  }

  // API_AVAILABLE(macos(10.12));
  unsafe fn setEmphasized_(self, emphasized: BOOL) {
    msg_send![self, setEmphasized: emphasized]
  }

  unsafe fn setMaterial_(self, material: NSVisualEffectMaterial) {
    msg_send![self, setMaterial: material]
  }

  unsafe fn setCornerRadius_(self, radius: f64) {
    msg_send![self, setCornerRadius: radius]
  }

  unsafe fn setState_(self, state: NSVisualEffectState) {
    msg_send![self, setState: state]
  }

  unsafe fn setBlendingMode_(self, mode: NSVisualEffectBlendingMode) {
    msg_send![self, setBlendingMode: mode]
  }
}

/// <https://developer.apple.com/documentation/appkit/nsvisualeffectview/material>
#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NSVisualEffectMaterial {
  #[deprecated = "Since macOS 10.14 a default material appropriate for the view's effectiveAppearance. You should instead choose an appropriate semantic material."]
  AppearanceBased = 0,
  #[deprecated = "Since macOS 10.14 use a semantic material instead."]
  Light = 1,
  #[deprecated = "Since macOS 10.14 use a semantic material instead."]
  Dark = 2,
  #[deprecated = "Since macOS 10.14 use a semantic material instead."]
  MediumLight = 8,
  #[deprecated = "Since macOS 10.14 use a semantic material instead."]
  UltraDark = 9,

  /// macOS 10.10+
  Titlebar = 3,
  /// macOS 10.10+
  Selection = 4,

  /// macOS 10.11+
  Menu = 5,
  /// macOS 10.11+
  Popover = 6,
  /// macOS 10.11+
  Sidebar = 7,

  /// macOS 10.14+
  HeaderView = 10,
  /// macOS 10.14+
  Sheet = 11,
  /// macOS 10.14+
  WindowBackground = 12,
  /// macOS 10.14+
  HudWindow = 13,
  /// macOS 10.14+
  FullScreenUI = 15,
  /// macOS 10.14+
  Tooltip = 17,
  /// macOS 10.14+
  ContentBackground = 18,
  /// macOS 10.14+
  UnderWindowBackground = 21,
  /// macOS 10.14+
  UnderPageBackground = 22,
}

/// <https://developer.apple.com/documentation/appkit/nsvisualeffectview/state>
#[allow(dead_code)]
#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NSVisualEffectState {
  /// Make window vibrancy state follow the window's active state
  FollowsWindowActiveState = 0,
  /// Make window vibrancy state always active
  Active = 1,
  /// Make window vibrancy state always inactive
  Inactive = 2,
}

impl From<crate::window::Effect> for NSVisualEffectMaterial {
  fn from(value: crate::window::Effect) -> Self {
    match value {
      Effect::AppearanceBased => NSVisualEffectMaterial::AppearanceBased,
      Effect::Light => NSVisualEffectMaterial::Light,
      Effect::Dark => NSVisualEffectMaterial::Dark,
      Effect::MediumLight => NSVisualEffectMaterial::MediumLight,
      Effect::UltraDark => NSVisualEffectMaterial::UltraDark,
      Effect::Titlebar => NSVisualEffectMaterial::Titlebar,
      Effect::Selection => NSVisualEffectMaterial::Selection,
      Effect::Menu => NSVisualEffectMaterial::Menu,
      Effect::Popover => NSVisualEffectMaterial::Popover,
      Effect::Sidebar => NSVisualEffectMaterial::Sidebar,
      Effect::HeaderView => NSVisualEffectMaterial::HeaderView,
      Effect::Sheet => NSVisualEffectMaterial::Sheet,
      Effect::WindowBackground => NSVisualEffectMaterial::WindowBackground,
      Effect::HudWindow => NSVisualEffectMaterial::HudWindow,
      Effect::FullScreenUI => NSVisualEffectMaterial::FullScreenUI,
      Effect::Tooltip => NSVisualEffectMaterial::Tooltip,
      Effect::ContentBackground => NSVisualEffectMaterial::ContentBackground,
      Effect::UnderWindowBackground => NSVisualEffectMaterial::UnderWindowBackground,
      Effect::UnderPageBackground => NSVisualEffectMaterial::UnderPageBackground,
      _ => unreachable!(),
    }
  }
}

impl From<crate::window::EffectState> for NSVisualEffectState {
  fn from(value: crate::window::EffectState) -> Self {
    match value {
      EffectState::FollowsWindowActiveState => NSVisualEffectState::FollowsWindowActiveState,
      EffectState::Active => NSVisualEffectState::Active,
      EffectState::Inactive => NSVisualEffectState::Inactive,
    }
  }
}
