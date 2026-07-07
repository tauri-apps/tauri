use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::{MouseButton, MouseButtonState, TrayIconEvent, TrayIconId};

use super::menu::muda_to_ksni_menu_item;

pub struct Tray {
    id: TrayIconId,
    icon: Vec<ksni::Icon>,
    title: String,
    status: ksni::Status,
    tooltip: String,
    menu: Vec<Arc<ArcSwap<muda::CompatMenuItem>>>,
}

impl Tray {
    pub fn new(
        id: TrayIconId,
        icon: Option<ksni::Icon>,
        title: String,
        tooltip: String,
        menu: Vec<Arc<ArcSwap<muda::CompatMenuItem>>>,
    ) -> Self {
        Self {
            id,
            icon: icon.into_iter().collect(),
            title,
            status: ksni::Status::Active,
            tooltip,
            menu,
        }
    }

    pub fn set_icon(&mut self, icon: Option<ksni::Icon>) {
        self.icon = icon.into_iter().collect();
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn set_status(&mut self, status: ksni::Status) {
        self.status = status;
    }

    pub fn set_tooltip(&mut self, tooltip: String) {
        self.tooltip = tooltip;
    }

    pub fn set_menu(&mut self, menu: Vec<Arc<ArcSwap<muda::CompatMenuItem>>>) {
        self.menu = menu;
    }
}

impl ksni::Tray for Tray {
    fn id(&self) -> String {
        self.id.0.clone()
    }

    fn status(&self) -> ksni::Status {
        self.status
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        self.icon.clone()
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            icon_pixmap: self.icon.clone(),
            title: self.title.clone(),
            description: self.tooltip.clone(),
            ..Default::default()
        }
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        self.menu
            .iter()
            .cloned()
            .map(muda_to_ksni_menu_item)
            .collect()
    }

    fn activate(&mut self, x: i32, y: i32) {
        let event = TrayIconEvent::Click {
            id: self.id.clone(),
            position: muda::dpi::PhysicalPosition::new(x as f64, y as f64),
            rect: Default::default(),
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
        };
        TrayIconEvent::send(event);
    }

    fn secondary_activate(&mut self, x: i32, y: i32) {
        let event = TrayIconEvent::Click {
            id: self.id.clone(),
            position: muda::dpi::PhysicalPosition::new(x as f64, y as f64),
            rect: Default::default(),
            button: MouseButton::Middle,
            button_state: MouseButtonState::Up,
        };
        TrayIconEvent::send(event);
    }
}
