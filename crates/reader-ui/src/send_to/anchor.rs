//! A keyboard menu has its own tiny native anchor, so it need not expand an
//! unread list item or scroll a long article to its footer to open Send to.
use super::{menu, request_at};
use crate::session::{Panel, Session};
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::{Point, Rect};
use ferroui_controls::Control;
use ferroui_view::prelude::*;
use reader_domain::Id;
use reader_state::AppViewModel;
use std::rc::Rc;

fn position(vm: &AppViewModel, ui: &Session, id: Id) -> Rect {
    if let Some(list) = ui.article_list.get_untracked() {
        if let Some(root) = list.visual_root() {
            let list_origin = list
                .translate_point(Point::new(0.0, 0.0), &root)
                .unwrap_or_default();
            let viewport = Rect::new(
                list_origin.x,
                list_origin.y,
                list.bounds().width,
                list.bounds().height,
            );
            let controls: Vec<_> = list
                .get_visual_descendants()
                .filter_map(|visual| visual.cast::<Control>())
                .filter(|control| {
                    control
                        .tag()
                        .as_ref()
                        .and_then(|tag| tag.downcast_ref::<Id>())
                        .copied()
                        == Some(id)
                })
                .collect();
            for name in ["ArticleSendTo", "ArticleRow", "ArticleEntry"] {
                for control in controls
                    .iter()
                    .filter(|control| control.name().as_deref() == Some(name))
                {
                    let Some(origin) = control.translate_point(Point::new(0.0, 0.0), &root) else {
                        continue;
                    };
                    let bounds = control.bounds();
                    if origin.y + bounds.height > viewport.y && origin.y < viewport.bottom() {
                        if name == "ArticleSendTo"
                            && origin.y >= viewport.y
                            && origin.y + bounds.height <= viewport.bottom()
                        {
                            return Rect::new(origin.x, origin.y, bounds.width, bounds.height);
                        }
                        if name != "ArticleSendTo" {
                            return Rect::new(
                                (origin.x + 12.0).max(viewport.x),
                                origin.y.max(viewport.y) + 4.0,
                                0.0,
                                24.0,
                            );
                        }
                    }
                }
            }
            return Rect::new(viewport.x + 12.0, viewport.y + 4.0, 0.0, 24.0);
        }
    }
    Rect::new(
        if vm.viewport() < 700.0 { 12.0 } else { 280.0 },
        140.0,
        0.0,
        24.0,
    )
}

pub(crate) fn popup(vm: Rc<AppViewModel>, ui: Rc<Session>, id: Id) -> Element {
    let rect = position(&vm, &ui, id);
    let anchor_ui = Rc::downgrade(&ui);
    border()
        .key(format!("send-to-keyboard:{id}"))
        .name("SendToKeyboardAnchor")
        .background("Transparent")
        .width(rect.width.max(1.0))
        .height(rect.height.max(1.0))
        .horizontal_alignment(H::Left)
        .vertical_alignment(V::Top)
        .margin((
            rect.x.clamp(0.0, (vm.viewport() - 1.0).max(0.0)),
            rect.y.clamp(0.0, (vm.viewport_height() - 1.0).max(0.0)),
            0.0,
            0.0,
        ))
        .context_menu(menu(vm, ui, id, true))
        .on_loaded(move |event| {
            if anchor_ui
                .upgrade()
                .is_some_and(|ui| ui.panel.get_untracked() == Panel::SendToMenu(id))
            {
                if let Some(control) = event.source().and_then(|source| source.cast::<Control>()) {
                    // Loaded can precede the first arrange; use the dimensions
                    // assigned to this anchor rather than its initial bounds.
                    request_at(
                        &control,
                        Rect::new(0.0, 0.0, rect.width.max(1.0), rect.height.max(1.0)),
                    );
                }
            }
        })
        .into()
}
