use crate::{menus, session::Session};
use ferroui_base::input::text_input::{TextInputContentType, TextInputReturnKeyType};
use ferroui_base::input::Key;
use ferroui_base::layout::{HorizontalAlignment as H, Orientation, VerticalAlignment as V};
use ferroui_base::media::{FontFamily, FontWeight};
use ferroui_view::prelude::*;
use reader_controls::{icon, Icon};
use reader_domain::{Density, View};
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn view(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let narrow = vm.viewport() < 700.0;
    let scoped = vm.viewport() >= 830.0;
    let touch = crate::responsive::touch(&vm);
    let tiny = vm.viewport() < 360.0;
    let density = vm.state().settings.density;
    let left = if narrow {
        if vm.viewport() < 300.0 {
            44.0
        } else if tiny {
            88.0
        } else {
            132.0
        }
    } else {
        crate::sidebar::width(density)
    };
    let inset = if narrow {
        if tiny {
            8.0
        } else {
            16.0
        }
    } else {
        match density {
            Density::Comfortable => 44.0,
            Density::Cozy => 28.0,
            Density::Compact => 16.0,
        }
    };
    let submit_vm = vm.clone();
    let submit_ui = ui.clone();
    let enter_vm = vm.clone();
    let enter_ui = ui.clone();
    let home = vm.clone();
    let mut header = grid()
        .key("search-header")
        .name("SearchHeader")
        .column_definitions(&format!("{left},*"))
        .height(60.0)
        .background("White")
        .margin(0.0)
        .child(
            button()
                .class("reader-flat")
                .margin((inset, 0.0, 0.0, 0.0))
                .vertical_alignment(V::Center)
                .automation_properties_name("Google Reader home")
                .on_click(move |_| home.navigate(View::Home))
                .child(if vm.viewport() < 300.0 {
                    icon(Icon::Home, 20.0, "#777777")
                } else {
                    logo(narrow, !tiny)
                }),
        );
    let mut search = grid()
        .grid_column(1)
        .column_definitions(if scoped { "*,156,72" } else { "*,44,44" })
        .column_spacing(if scoped { 18.0 } else { 4.0 })
        .width(
            (vm.viewport() - left - 16.0)
                .min(if scoped { 532.0 } else { 500.0 })
                .max(1.0),
        )
        .horizontal_alignment(H::Left)
        .vertical_alignment(V::Center)
        .child(
            reader_controls::input("Search Reader")
                .text_input_options_content_type(TextInputContentType::Search)
                .text_input_options_return_key_type(TextInputReturnKeyType::Search)
                .text_input_options_auto_capitalization(false)
                .text_input_options_show_suggestions(Some(false))
                .name("SearchReader")
                .node_ref(ui.search_input)
                .text(ui.search.two_way())
                .placeholder_text(if narrow { "Search" } else { "Search Reader" })
                .on_key_down(move |event| {
                    if event.key == Key::Return {
                        event.set_handled(true);
                        submit(&enter_vm, &enter_ui);
                    }
                }),
        );
    search = search.child(border().grid_column(1).child(scope(vm, ui, !scoped)));
    search = search.child(rsx!{Button {Grid.column:2,class:"reader-primary",name:"SearchButton",height:if touch{44.0}else{29.0},padding:8.0,
        automation_properties_name:"Search Reader",on_click:move |_|submit(&submit_vm,&submit_ui),
        {icon(Icon::Search,15.0,"White")}
    }});
    header = header.child(search);
    header.into()
}
fn submit(vm: &AppViewModel, ui: &Session) {
    let query = ui.search.get_untracked();
    let suffix = match effective_scope(vm, ui.search_scope.get_untracked()) {
        View::Starred => "is:starred".to_string(),
        View::Feed(id) => format!("feed-id:{id}"),
        View::Folder(name) => format!("folder:\"{}\"", name.replace('"', "")),
        View::Search(query) => query,
        _ => String::new(),
    };
    vm.navigate(View::Search(format!("{query} {suffix}").trim().to_string()));
}
fn scope(vm: Rc<AppViewModel>, ui: Rc<Session>, compact: bool) -> Element {
    let current = effective_scope(&vm, ui.search_scope.get());
    let mut choices = vec![
        ("All items".to_string(), View::All),
        ("Starred items".to_string(), View::Starred),
        (
            "Unread items".to_string(),
            View::Search("is:unread".to_string()),
        ),
    ];
    choices.extend(
        vm.folders()
            .into_iter()
            .map(|name| (name.clone(), View::Folder(name))),
    );
    choices.extend(
        vm.feeds()
            .into_iter()
            .map(|feed| (feed.title, View::Feed(feed.id))),
    );
    let label = choices
        .iter()
        .find(|(_, view)| *view == current)
        .map(|(title, _)| title.as_str())
        .unwrap_or("All items")
        .to_string();
    let mut items = Vec::new();
    for (label, view) in choices {
        let ui = ui.clone();
        let selected = view == current;
        items.push(menus::checked(&label, selected, move || {
            ui.search_scope.set(view.clone())
        }));
    }
    let control = if compact {
        reader_controls::toolbar_icon(Icon::ChevronDown, "Search scope")
            .width(44.0)
            .tool_tip_tip(format!("Search in {label}"))
    } else {
        reader_controls::dropdown(&label)
            .width(156.0)
            .max_width(156.0)
            .automation_properties_name("Search scope")
    };
    menus::attach(&vm, control, items)
}

/// A removed or archived source cannot leave an invisible filter behind the
/// "All items" label. Display and submission must resolve the same scope even
/// when a host/import/undo changes subscriptions outside the current dialog.
fn effective_scope(vm: &AppViewModel, scope: View) -> View {
    match scope {
        View::Feed(id) if vm.feed(id).is_some() => View::Feed(id),
        View::Folder(name) if vm.folders().contains(&name) => View::Folder(name),
        View::Starred => View::Starred,
        View::Search(query) if query == "is:unread" => View::Search(query),
        _ => View::All,
    }
}
fn logo(narrow: bool, reader: bool) -> Element {
    let mut word = stack_panel()
        .orientation(Orientation::Horizontal)
        .vertical_alignment(V::Center)
        .automation_properties_name("Google Reader");
    for (letter, color) in [
        ("G", "#3369E8"),
        ("o", "#D50F25"),
        ("o", "#EEB211"),
        ("g", "#3369E8"),
        ("l", "#009939"),
        ("e", "#D50F25"),
    ] {
        word = word.child(
            text_block()
                .text(letter)
                .font_family(FontFamily::new("fonts:Reader#Liberation Serif"))
                .font_size(if narrow { 18.0 } else { 27.0 })
                .font_weight(FontWeight::Normal)
                .foreground(color),
        );
    }
    if reader {
        word = word.child(
            text_block()
                .text("reader")
                .font_size(if narrow { 14.0 } else { 20.0 })
                .font_weight(FontWeight::Normal)
                .foreground("#777777")
                .margin((4.0, 3.0, 0.0, 0.0))
                .vertical_alignment(V::Center),
        );
    }
    word.into()
}
