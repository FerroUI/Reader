//! The footer and Shift+T use the same native menu and MVVM command.
mod anchor;

use crate::{
    rows::Row,
    session::{Panel, Session},
};
use ferroui_base::input::ContextRequestedEventArgs;
use ferroui_base::layout::{Orientation, VerticalAlignment as V};
use ferroui_base::Rect;
use ferroui_controls::{Control, MenuBase, PlacementMode};
use ferroui_view::prelude::*;
use reader_domain::Id;
use reader_state::AppViewModel;
use std::rc::Rc;

pub(crate) use anchor::popup;

pub(crate) fn open(vm: &AppViewModel, ui: &Session, id: Id) {
    if vm.state().article(id).is_none() {
        return;
    }
    let enabled = vm.state().settings.send_to.iter().any(|item| item.enabled);
    ui.show(if enabled {
        Panel::SendToMenu(id)
    } else {
        Panel::SendToSettings
    });
}

pub(crate) fn button(vm: Rc<AppViewModel>, ui: Rc<Session>, row: &Row) -> Element {
    if !vm.state().settings.send_to.iter().any(|item| item.enabled) {
        return Element::Empty;
    }
    let id = row.article.id;
    let font_size = 13.0 * row.scale.factor();
    ferroui_view::prelude::button().key(format!("send-to:{id}:{}", row.send_to_revision))
        .name("ArticleSendTo").tag(Some(Rc::new(id) as ferroui_base::BoxedValue))
        .class("reader-link").font_size(font_size).foreground("#444444")
        .padding((8.0,0.0)).height(crate::article::action_height(row))
        .min_width(if row.touch {44.0}else{0.0}).automation_properties_name("Send to")
        .context_menu(menu(vm,ui,id,false))
        .on_click(move |event|{
            event.set_handled(true);
            if let Some(control)=event.source().and_then(|source|source.cast::<Control>()){
                request_at(&control,Rect::from_size(control.bounds().size()));
            }
        })
        .child(rsx!{StackPanel {orientation:Orientation::Horizontal,spacing:5.0,vertical_alignment:V::Center,
            TextBlock {text:"Send to",font_size,foreground:"#444444",vertical_alignment:V::Center}
            {reader_controls::icon(reader_controls::Icon::ChevronDown,8.0,"#777777")}
        }}).into()
}

/// An unpositioned context request uses centered keyboard placement. Supply the
/// full local anchor while retaining native opening, focus and dismissal events.
fn request_at(control: &Control, rect: Rect) {
    let request = ContextRequestedEventArgs::new();
    request.set_placement_rect(control, rect);
    control.raise_event(&request);
}

fn menu(vm: Rc<AppViewModel>, ui: Rc<Session>, id: Id, close_panel: bool) -> Element {
    let generation = vm.library_generation();
    let source_url = vm
        .state()
        .article(id)
        .map(|article| article.url.clone())
        .unwrap_or_default();
    let destinations = vm.state().settings.send_to.clone();
    let width = (vm.viewport() - 16.0).max(1.0);
    let close = Rc::downgrade(&ui);
    let mut menu = context_menu()
        .name("ArticleSendToMenu")
        .class(if crate::responsive::touch(&vm) {
            "reader-touch"
        } else {
            "reader-desktop"
        })
        .placement(PlacementMode::BottomEdgeAlignedLeft)
        .min_width(200.0_f64.min(width))
        .max_width(width)
        .max_height((vm.viewport_height() - 16.0).max(1.0))
        .with(move |menu| {
            crate::transient::attach_menu(menu);
            if close_panel {
                let close = close.clone();
                let token = menu.closed(move |_, _| {
                    if let Some(ui) = close
                        .upgrade()
                        .filter(|ui| ui.panel.get_untracked() == Panel::SendToMenu(id))
                    {
                        ui.close();
                    }
                });
                let weak = menu.downgrade();
                on_cleanup(move || {
                    if let Some(menu) = weak.upgrade() {
                        menu.remove_handler(MenuBase::closed_event(), token);
                    }
                });
            }
        });
    for destination in destinations.into_iter().filter(|item| item.enabled) {
        let label = destination.name.clone();
        let target = vm.clone();
        let url = source_url.clone();
        menu = menu.child(crate::menus::item(&label, move || {
            if let Err(error) = target.send_article_to(id, &url, generation, &destination) {
                target.report_error(error);
            }
        }));
    }
    menu.child(crate::menus::separator())
        .child(crate::menus::item("Customize Send to", move || {
            ui.show(Panel::SendToSettings)
        }))
        .into()
}
