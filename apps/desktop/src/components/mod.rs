//! Dormant GPUI component prototypes retained for planned desktop work.
//!
//! Keep this allowance at this module boundary only. Remove each exemption when
//! the corresponding component is wired into the desktop or delete the stale
//! prototype instead.
#![allow(dead_code, unused_imports)]

mod badge;
mod button;
mod context_menu;
mod label;
mod resizable;
mod separator;

pub(crate) use badge::{Badge, BadgeVariant};
pub(crate) use button::{Button, ButtonSize, ButtonVariant};
pub(crate) use context_menu::{
    ContextMenu, ContextMenuItem, ContextMenuItemVariant, context_menu_trigger,
};
pub(crate) use label::Label;
pub(crate) use resizable::{Orientation, ResizablePanelGroup};
pub(crate) use separator::Separator;
