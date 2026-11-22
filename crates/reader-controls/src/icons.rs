use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::Geometry;
use ferroui_base::Ref;
use ferroui_view::prelude::*;
use std::{cell::RefCell, collections::HashMap};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Icon {
    Home,
    Star,
    Starred,
    Feed,
    Folder,
    ChevronDown,
    ChevronRight,
    ChevronLeft,
    ChevronUp,
    Refresh,
    List,
    Expanded,
    Search,
    Settings,
    Close,
    Menu,
    Trends,
    Explore,
    Share,
    Tag,
    Mail,
    External,
    Check,
}
ferroui_base::ferro_thread_local! {
    static CACHE: RefCell<HashMap<Icon, Ref<Geometry>>> = RefCell::new(HashMap::new());
}
pub fn icon(kind: Icon, size: f64, color: &str) -> Element {
    let geometry = CACHE.with(|cache| {
        cache
            .borrow_mut()
            .entry(kind)
            .or_insert_with(|| Geometry::parse(path(kind)).expect("embedded vector icon"))
            .clone()
    });
    path_icon()
        .data(Some(geometry))
        .width(size)
        .height(size)
        .foreground(color)
        .vertical_alignment(V::Center)
        .horizontal_alignment(H::Center)
        .is_hit_test_visible(false)
        .automation_properties_is_control_element_override(Some(false))
        .into()
}
fn path(kind: Icon) -> &'static str {
    match kind {
    Icon::Home=>"M2,10 L12,2 L22,10 L20,12 L18,10 L18,22 L14,22 L14,15 L10,15 L10,22 L6,22 L6,10 L4,12 Z",
    Icon::Star=>"M12,1 L15.4,8 L23,9 L17.5,14.4 L19,22 L12,18.2 L5,22 L6.5,14.4 L1,9 L8.6,8 Z M12,5.6 L10,10 L5.4,10.6 L8.7,13.8 L7.9,18.4 L12,16.2 L16.1,18.4 L15.3,13.8 L18.6,10.6 L14,10 Z",
    Icon::Starred=>"M12,1 L15.4,8 L23,9 L17.5,14.4 L19,22 L12,18.2 L5,22 L6.5,14.4 L1,9 L8.6,8 Z",
    Icon::Feed=>"M3,2 C13.5,2 22,10.5 22,21 L18,21 C18,12.7 11.3,6 3,6 Z M3,9 C9.6,9 15,14.4 15,21 L11,21 C11,16.6 7.4,13 3,13 Z M5,17 A2.5,2.5 0 1 1 5,22 A2.5,2.5 0 1 1 5,17 Z",
    Icon::Folder=>"M2,5 L10,5 L12,8 L22,8 L22,21 L2,21 Z",
    Icon::ChevronDown=>"M4,8 L12,16 L20,8 Z",
    Icon::ChevronRight=>"M8,4 L16,12 L8,20 Z",
    Icon::ChevronLeft=>"M16,4 L8,12 L16,20 Z",
    Icon::ChevronUp=>"M4,16 L12,8 L20,16 Z",
    Icon::Refresh=>"M19,4 L19,1 L24,7 L17,9 L18,6 C13,1 4,5 4,12 C4,19 13,23 18,18 L20,20 C13,27 1,22 1,12 C1,3 12,-2 19,4 Z",
    Icon::List=>"M1,3 L4,3 L4,6 L1,6 Z M7,3 L23,3 L23,6 L7,6 Z M1,10 L4,10 L4,13 L1,13 Z M7,10 L23,10 L23,13 L7,13 Z M1,17 L4,17 L4,20 L1,20 Z M7,17 L23,17 L23,20 L7,20 Z",
    Icon::Expanded=>"M2,3 L22,3 L22,6 L2,6 Z M2,9 L22,9 L22,20 L2,20 Z M5,12 L5,17 L19,17 L19,12 Z",
    Icon::Search=>"M10,1 A9,9 0 1 1 10,19 A9,9 0 1 1 10,1 Z M10,4 A6,6 0 1 0 10,16 A6,6 0 1 0 10,4 Z M16,14 L24,22 L22,24 L14,16 Z",
    Icon::Settings=>"M9,0 L15,0 L16,4 L19,5 L23,4 L26,9 L23,12 L23,15 L26,18 L23,23 L19,22 L16,24 L15,28 L9,28 L8,24 L5,22 L1,23 L-2,18 L1,15 L1,12 L-2,9 L1,4 L5,5 L8,4 Z M12,8 A6,6 0 1 0 12,20 A6,6 0 1 0 12,8 Z",
    Icon::Close=>"M4,2 L12,10 L20,2 L22,4 L14,12 L22,20 L20,22 L12,14 L4,22 L2,20 L10,12 L2,4 Z",
    Icon::Menu=>"M2,4 L22,4 L22,7 L2,7 Z M2,11 L22,11 L22,14 L2,14 Z M2,18 L22,18 L22,21 L2,21 Z",
    Icon::Trends=>"M2,12 L7,12 L7,22 L2,22 Z M10,6 L15,6 L15,22 L10,22 Z M18,1 L23,1 L23,22 L18,22 Z",
    Icon::Explore=>"M12,1 A11,11 0 1 0 12,23 A11,11 0 1 0 12,1 Z M18,6 L14,14 L6,18 L10,10 Z",
    Icon::Share=>"M18,1 A4,4 0 1 0 18,9 A4,4 0 1 0 18,1 Z M5,8 A4,4 0 1 0 5,16 A4,4 0 1 0 5,8 Z M18,15 A4,4 0 1 0 18,23 A4,4 0 1 0 18,15 Z M7,10 L17,4 L18,6 L8,12 Z M7,12 L18,18 L17,20 L7,14 Z",
    Icon::Tag=>"M2,2 L14,2 L24,12 L12,24 L2,14 Z M7,5 A2,2 0 1 0 7,9 A2,2 0 1 0 7,5 Z",
    Icon::Mail=>"M1,4 L23,4 L23,20 L1,20 Z M3,6 L12,13 L21,6 Z M3,9 L3,18 L21,18 L21,9 L12,16 Z",
    Icon::External=>"M13,1 L23,1 L23,11 L20,11 L20,6 L11,15 L9,13 L18,4 L13,4 Z M2,5 L10,5 L10,8 L5,8 L5,20 L17,20 L17,14 L20,14 L20,23 L2,23 Z",
    Icon::Check=>"M2,12 L5,9 L10,14 L20,3 L23,6 L10,21 Z",
}
}
