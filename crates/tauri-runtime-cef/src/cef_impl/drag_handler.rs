use cef::{rc::*, *};

#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub use windows::BrowserDragHandlerData;

#[cfg(not(windows))]
pub struct BrowserDragHandlerData;

#[cfg(not(windows))]
impl BrowserDragHandlerData {
  pub fn new() -> Self {
    Self
  }
}

wrap_drag_handler! {
  pub struct BrowserDragHandler {
    data: BrowserDragHandlerData
  }

  impl DragHandler {
    fn on_draggable_regions_changed(
      &self,
      browser: Option<&mut Browser>,
      _frame: Option<&mut Frame>,
      regions: Option<&[cef::DraggableRegion]>
    ) {

      let Some(regions) = regions else { return; };
      let Some(browser) = browser else { return; };

      #[cfg(windows)]
      unsafe { windows::on_dragged_region_changed(&self.data, browser, regions) };
    }
  }
}
