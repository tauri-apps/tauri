// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Conversions between Tauri runtime types and Qt values.

use qtshim::app::{self, Screen};
use tauri_runtime::{
  Error, ResizeDirection,
  dpi::{
    LogicalPosition, LogicalSize, PhysicalPosition, PhysicalRect, PhysicalSize, PixelUnit,
    Position, Size,
  },
  monitor::Monitor,
  window::CursorIcon,
};

/// An error for APIs that have no Qt equivalent.
#[derive(Debug)]
pub struct Unsupported(pub &'static str);

impl std::fmt::Display for Unsupported {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "`{}` is not available with the Qt runtime", self.0)
  }
}

impl std::error::Error for Unsupported {}

pub fn unsupported(what: &'static str) -> Error {
  Error::CreateWebview(Box::new(Unsupported(what)))
}

pub fn round(v: f64) -> i32 {
  v.round() as i32
}

pub fn size_to_logical(size: Size, scale: f64) -> (i32, i32) {
  let s: LogicalSize<f64> = size.to_logical(scale);
  (round(s.width), round(s.height))
}

pub fn position_to_logical(pos: Position, scale: f64) -> (i32, i32) {
  let p: LogicalPosition<f64> = pos.to_logical(scale);
  (round(p.x), round(p.y))
}

pub fn unit_to_logical(unit: PixelUnit, scale: f64) -> f64 {
  match unit {
    PixelUnit::Logical(l) => l.0,
    PixelUnit::Physical(p) => p.0 as f64 / scale,
  }
}

pub fn physical_size(w: i32, h: i32, scale: f64) -> PhysicalSize<u32> {
  PhysicalSize::new(
    (w.max(0) as f64 * scale).round() as u32,
    (h.max(0) as f64 * scale).round() as u32,
  )
}

pub fn physical_position(x: i32, y: i32, scale: f64) -> PhysicalPosition<i32> {
  PhysicalPosition::new(round(x as f64 * scale), round(y as f64 * scale))
}

pub fn monitor_from_screen(s: &Screen) -> Monitor {
  Monitor {
    name: Some(s.name.clone()),
    size: physical_size(s.width, s.height, s.scale),
    position: physical_position(s.x, s.y, s.scale),
    work_area: PhysicalRect {
      position: physical_position(s.available_x, s.available_y, s.scale),
      size: physical_size(s.available_width, s.available_height, s.scale),
    },
    scale_factor: s.scale,
  }
}

/// Screen containing a physical point.
pub fn screen_at_physical(x: f64, y: f64) -> Option<Screen> {
  app::screens().into_iter().find(|s| {
    let (sx, sy) = (s.x as f64 * s.scale, s.y as f64 * s.scale);
    let (sw, sh) = (s.width as f64 * s.scale, s.height as f64 * s.scale);
    x >= sx && x < sx + sw && y >= sy && y < sy + sh
  })
}

/// Maps to `Qt::CursorShape`.
pub fn cursor_shape(icon: CursorIcon) -> i32 {
  use CursorIcon::*;
  match icon {
    Default | Arrow => 0,                          // ArrowCursor
    Crosshair => 2,                                // CrossCursor
    Wait => 3,                                     // WaitCursor
    Text | VerticalText => 4,                      // IBeamCursor
    NsResize | NResize | SResize | RowResize => 5, // SizeVerCursor
    EwResize | EResize | WResize | ColResize => 6, // SizeHorCursor
    NeswResize | NeResize | SwResize => 7,         // SizeBDiagCursor
    NwseResize | NwResize | SeResize => 8,         // SizeFDiagCursor
    Move | AllScroll => 9,                         // SizeAllCursor
    Hand => 13,                                    // PointingHandCursor
    NotAllowed | NoDrop => 14,                     // ForbiddenCursor
    Help | ContextMenu => 15,                      // WhatsThisCursor
    Progress => 16,                                // BusyCursor
    Grab => 17,                                    // OpenHandCursor
    Grabbing => 18,                                // ClosedHandCursor
    Copy => 19,                                    // DragCopyCursor
    Alias => 21,                                   // DragLinkCursor
    Cell | ZoomIn | ZoomOut => 2,
    _ => 0,
  }
}

/// Maps to a `Qt::Edges` bitmask.
pub fn resize_edges(dir: ResizeDirection) -> u32 {
  const TOP: u32 = 1;
  const LEFT: u32 = 2;
  const RIGHT: u32 = 4;
  const BOTTOM: u32 = 8;
  match dir {
    ResizeDirection::East => RIGHT,
    ResizeDirection::North => TOP,
    ResizeDirection::NorthEast => TOP | RIGHT,
    ResizeDirection::NorthWest => TOP | LEFT,
    ResizeDirection::South => BOTTOM,
    ResizeDirection::SouthEast => BOTTOM | RIGHT,
    ResizeDirection::SouthWest => BOTTOM | LEFT,
    ResizeDirection::West => LEFT,
  }
}

/// Does `host` match a cookie `domain` (RFC 6265 domain matching, leading dot ignored)?
pub fn domain_matches(host: &str, domain: &str) -> bool {
  let domain = domain.trim_start_matches('.').to_ascii_lowercase();
  let host = host.to_ascii_lowercase();
  host == domain || host.ends_with(&format!(".{domain}"))
}

pub fn path_matches(request_path: &str, cookie_path: &str) -> bool {
  if cookie_path.is_empty() || cookie_path == "/" || request_path == cookie_path {
    return true;
  }
  request_path.starts_with(cookie_path)
    && (cookie_path.ends_with('/') || request_path[cookie_path.len()..].starts_with('/'))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn cookie_matching() {
    assert!(domain_matches("api.example.com", ".example.com"));
    assert!(domain_matches("example.com", "example.com"));
    assert!(!domain_matches("badexample.com", "example.com"));
    assert!(path_matches("/a/b", "/a"));
    assert!(!path_matches("/ab", "/a"));
    assert!(path_matches("/anything", "/"));
  }

  #[test]
  fn cookie_matching_edge_cases() {
    // Case-insensitive, and a leading dot is ignored.
    assert!(domain_matches("API.Example.COM", "example.com"));
    assert!(domain_matches("example.com", ".EXAMPLE.com"));
    // A parent domain doesn't match a subdomain cookie.
    assert!(!domain_matches("example.com", "api.example.com"));
    assert!(path_matches("/a/b", "/a/"));
    assert!(path_matches("/a", "/a"));
    assert!(path_matches("/x", ""));
    assert!(!path_matches("/", "/a"));
    assert!(!path_matches("/b/a", "/a"));
  }

  #[test]
  fn conversions_round_to_whole_pixels() {
    assert_eq!(round(1.5), 2);
    assert_eq!(round(-1.4), -1);
    assert_eq!(
      size_to_logical(Size::Physical(PhysicalSize::new(300, 201)), 2.0),
      (150, 101)
    );
    assert_eq!(
      size_to_logical(Size::Logical(LogicalSize::new(10.4, 10.6)), 3.0),
      (10, 11)
    );
    assert_eq!(
      position_to_logical(Position::Physical(PhysicalPosition::new(-30, 45)), 1.5),
      (-20, 30)
    );
    assert_eq!(
      position_to_logical(Position::Logical(LogicalPosition::new(7.0, 8.0)), 2.0),
      (7, 8)
    );
    assert_eq!(unit_to_logical(PixelUnit::Logical(12.5.into()), 2.0), 12.5);
    assert_eq!(unit_to_logical(PixelUnit::Physical(30.into()), 2.0), 15.0);
    assert_eq!(physical_size(100, 50, 1.25), PhysicalSize::new(125, 63));
    // Negative sizes clamp to zero.
    assert_eq!(physical_size(-5, 10, 1.0), PhysicalSize::new(0, 10));
    assert_eq!(
      physical_position(-10, 3, 2.0),
      PhysicalPosition::new(-20, 6)
    );
  }

  #[test]
  fn monitor_from_screen_scales_to_physical() {
    let screen = Screen {
      name: "DP-1".into(),
      x: 100,
      y: 0,
      width: 1280,
      height: 720,
      available_x: 100,
      available_y: 30,
      available_width: 1280,
      available_height: 690,
      scale: 2.0,
      primary: true,
    };
    let m = monitor_from_screen(&screen);
    assert_eq!(m.name.as_deref(), Some("DP-1"));
    assert_eq!(m.size, PhysicalSize::new(2560, 1440));
    assert_eq!(m.position, PhysicalPosition::new(200, 0));
    assert_eq!(m.work_area.position, PhysicalPosition::new(200, 60));
    assert_eq!(m.work_area.size, PhysicalSize::new(2560, 1380));
    assert_eq!(m.scale_factor, 2.0);
  }

  #[test]
  fn cursor_shapes_map_to_qt() {
    use CursorIcon::*;
    let cases = [
      (Default, 0),
      (Arrow, 0),
      (Crosshair, 2),
      (Wait, 3),
      (Text, 4),
      (VerticalText, 4),
      (NsResize, 5),
      (RowResize, 5),
      (EwResize, 6),
      (ColResize, 6),
      (NeswResize, 7),
      (NwseResize, 8),
      (Move, 9),
      (AllScroll, 9),
      (Hand, 13),
      (NotAllowed, 14),
      (NoDrop, 14),
      (Help, 15),
      (ContextMenu, 15),
      (Progress, 16),
      (Grab, 17),
      (Grabbing, 18),
      (Copy, 19),
      (Alias, 21),
      (Cell, 2),
      (ZoomIn, 2),
      (ZoomOut, 2),
    ];
    for (icon, shape) in cases {
      assert_eq!(cursor_shape(icon), shape, "{icon:?}");
    }
  }

  #[test]
  fn resize_directions_map_to_qt_edges() {
    use ResizeDirection::*;
    assert_eq!(resize_edges(North), 1);
    assert_eq!(resize_edges(West), 2);
    assert_eq!(resize_edges(East), 4);
    assert_eq!(resize_edges(South), 8);
    assert_eq!(resize_edges(NorthEast), 1 | 4);
    assert_eq!(resize_edges(NorthWest), 1 | 2);
    assert_eq!(resize_edges(SouthEast), 8 | 4);
    assert_eq!(resize_edges(SouthWest), 8 | 2);
  }

  #[test]
  fn unsupported_names_the_api() {
    let e = unsupported("gtk_window");
    assert!(
      e.to_string()
        .contains("`gtk_window` is not available with the Qt runtime"),
      "{e}"
    );
  }
}
