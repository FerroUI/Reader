use crate::{
    dialogs, header, home, keyboard,
    rows::Rows,
    session::{Panel, Session},
    sidebar, toolbar,
};
use ferroui_base::input::KeyboardNavigationMode;
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::TextWrapping;
use ferroui_controls::primitives::ScrollBarVisibility;
use ferroui_view::prelude::*;
use reader_controls::{icon_action, Icon};
use reader_domain::View;
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn view(vm: Rc<AppViewModel>, ui: Rc<Session>, rows: Rc<Rows>) -> Element {
    retain_route_scroll(vm.clone(), ui.clone(), rows.clone());
    let narrow = vm.viewport() < 700.0;
    let touch = crate::responsive::touch(&vm);
    let article_stream = !matches!(
        vm.view(),
        View::Home | View::Trends | View::Explore | View::Settings
    );
    let focus = ui.focus_mode.get();
    let show_sidebar = !narrow && ui.desktop_sidebar.get() && !focus;
    let width = if show_sidebar {
        sidebar::width(vm.state().settings.density)
    } else {
        0.0
    };
    let keys_vm = vm.clone();
    let keys_ui = ui.clone();
    let close = vm.clone();
    let undo = vm.clone();
    let menu = vm.clone();
    let mut root = grid()
        .name("ReaderApp")
        .class(if touch {
            "reader-touch"
        } else {
            "reader-desktop"
        })
        .background("White")
        .row_definitions(if focus { "0,*" } else { "60,*" })
        .focusable(true)
        .is_tab_stop(false)
        .with(move |root| keyboard::attach(root, keys_vm.clone(), keys_ui.clone()));
    if !focus {
        root = root.child(header::view(vm.clone(), ui.clone()));
    }
    let mut layout = grid()
        .key("layout")
        .grid_row(1)
        .column_definitions(&format!("{width},*"));
    if show_sidebar {
        layout = layout.child(sidebar::view(vm.clone(), ui.clone()));
    }
    let mut main = grid()
        .key("main")
        .grid_column(1)
        .row_definitions("Auto,*,Auto,Auto");
    let mut top = grid()
        .key("viewer-header")
        .column_definitions(if narrow || !show_sidebar {
            if touch {
                "48,*"
            } else {
                "40,*"
            }
        } else {
            "0,*"
        });
    if narrow || !show_sidebar {
        let side = ui.clone();
        top = top.child(
            border()
                .vertical_alignment(V::Top)
                .height(if touch {
                    56.0
                } else if vm.state().settings.density == reader_domain::Density::Compact {
                    43.0
                } else {
                    57.0
                })
                .padding((4.0, if touch { 0.0 } else { 6.0 }, 0.0, 0.0))
                .child(icon_action(Icon::Menu, "Show subscriptions", move || {
                    if menu.viewport() < 700.0 {
                        menu.toggle_sidebar();
                    } else {
                        side.desktop_sidebar.update(|v| *v = !*v);
                        side.focus_mode.set(false);
                    }
                })),
        );
    }
    top = top.child(border().grid_column(1).child(
        if matches!(vm.view(), View::Home | View::Trends | View::Explore) {
            toolbar::page_header(vm.clone(), ui.clone())
        } else {
            toolbar::view(vm.clone(), ui.clone())
        },
    ));
    main = main.child(top);
    if matches!(vm.view(), View::Home | View::Trends | View::Explore) {
        main = main.child(
            grid()
                .key("overview")
                .grid_row(1)
                .child(home::view(vm.clone(), ui.clone())),
        );
    } else if vm.view() == View::Settings {
        main = main.child(
            grid().key("settings-view").grid_row(1).child(
                scroll_viewer()
                    .name("ReaderPageScroll")
                    .node_ref(ui.page_scroll)
                    .horizontal_scroll_bar_visibility(ScrollBarVisibility::Disabled)
                    .vertical_scroll_bar_visibility(ScrollBarVisibility::Auto)
                    .child(
                        border()
                            .padding(if narrow { 16.0 } else { 26.0 })
                            .child(crate::settings::preferences(vm.clone(), ui.clone())),
                    ),
            ),
        );
    } else {
        let reading_vm = vm.clone();
        let reading_ui = ui.clone();
        let mut stream = grid().key("stream").grid_row(1).clip_to_bounds(true).child(
            list_box()
                .name("ArticleList")
                .focusable(true)
                .is_tab_stop(false)
                .with(move |list| {
                    crate::reading::attach(list, reading_vm.clone(), reading_ui.clone())
                })
                .node_ref(ui.article_list)
                .class("reader-list")
                .automation_properties_name("Articles")
                .items_source(rows.source())
                .item_template(rows.template.clone())
                .border_thickness(0.0)
                .padding(0.0),
        );
        if rows.len() == 0 {
            stream = stream.child(crate::empty::view(vm.clone()));
        }
        main = main.child(stream);
    }
    if !vm.status().is_empty() {
        main=main.child(rsx!{
        Border{Grid.row:2,background:"#FFF8CE",border_brush:"#F0C36D",border_thickness:(0.0,1.0,0.0,0.0),padding:(14.0,7.0),
            Grid{column_definitions:if narrow{"*,Auto"}else{"*,Auto,Auto"},row_definitions:if narrow{"Auto,Auto"}else{"Auto"},column_spacing:12.0,
                ScrollViewer{Grid.column_span:if narrow{2}else{1},max_height:if narrow{80.0}else{110.0},horizontal_scroll_bar_visibility:ScrollBarVisibility::Disabled,vertical_scroll_bar_visibility:ScrollBarVisibility::Auto,
                    TextBlock{text:vm.status(),font_size:12.0,text_wrapping:TextWrapping::Wrap,vertical_alignment:V::Center}
                }
                if vm.can_undo(){Border{Grid.column:if narrow{0}else{1},Grid.row:if narrow{1}else{0},{reader_controls::link("Undo",move||undo.undo())}}}
                Border{Grid.column:if narrow{1}else{2},Grid.row:if narrow{1}else{0},{reader_controls::link("Dismiss",move||close.clear_status())}}
            }
        }
    });
    }
    if narrow && article_stream {
        main = main.child(
            border()
                .grid_row(3)
                .child(toolbar::reading_navigation(vm.clone())),
        );
    }
    layout = layout.child(main);
    root = root.child(layout);
    if narrow && vm.sidebar_open() {
        let dismiss = vm.clone();
        let restore = ui.clone();
        root=root.child(rsx!{Grid{key:"mobile-navigation",name:"SubscriptionsDrawer",Grid.row:1,keyboard_navigation_tab_navigation:KeyboardNavigationMode::Cycle,
        Button{class:"reader-flat",background:"#55000000",horizontal_alignment:H::Stretch,vertical_alignment:V::Stretch,is_tab_stop:false,automation_properties_name:"Dismiss subscriptions drawer",on_click:move |_|{dismiss.set_sidebar_open(false);restore.restore_article_focus();}}
        Border{width:(vm.viewport()-44.0).clamp(1.0,320.0),horizontal_alignment:H::Left,background:"White",{sidebar::view(vm.clone(),ui.clone())}}
    }});
    }
    if ui.panel.get() != Panel::None {
        root = root.child(
            grid()
                .key("dialog")
                .grid_row_span(2)
                .child(dialogs::view(vm, ui)),
        );
    }
    root.into()
}

/// Route changes start at the first entry, while property-only revisions retain the
/// same native ListBox, ScrollViewer and recycling collection (including its offset).
fn retain_route_scroll(vm: Rc<AppViewModel>, ui: Rc<Session>, rows: Rc<Rows>) {
    let route = vm.view();
    let generation_changed =
        ui.last_library_generation.replace(vm.library_generation()) != vm.library_generation();
    let route_changed = ui
        .last_view
        .borrow()
        .as_ref()
        .is_some_and(|previous| previous != &route);
    if generation_changed || route_changed {
        // This is shared by native and browser hosts. Async navigation must not
        // leave an editor or confirmation targeting a replaced stream/library.
        crate::transient::dismiss_menu();
        if ui.panel.get_untracked() != Panel::None {
            ui.close();
        }
        if generation_changed {
            ui.tree_cursor.set(None);
            *ui.tree_opened.borrow_mut() = None;
        }
    }
    if ui.last_view.borrow().as_ref() == Some(&route) {
        return;
    }
    *ui.last_view.borrow_mut() = Some(route.clone());
    ferroui_base::threading::Dispatcher::ui_thread().post_local(
        move || {
            if vm.view() != route {
                return;
            }
            let Some(list) = ui.article_list.get_untracked() else {
                return;
            };
            ui.programmatic_scroll.set(Some(std::time::Instant::now()));
            if let Some(index) = vm.cursor().or(vm.selected()).and_then(|id| rows.index(id)) {
                list.scroll_into_view(index);
            } else if let Some(scroll) = list
                .scroll()
                .and_then(|scroll| scroll.cast::<ferroui_controls::ScrollViewer>())
            {
                scroll.scroll_to_home();
            } else if rows.len() > 0 {
                list.scroll_into_view(0);
            }
        },
        ferroui_base::threading::DispatcherPriority::BACKGROUND,
    );
}
