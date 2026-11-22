use crate::session::{Panel, Session};
use ferroui_base::input::{InputElement, Key, KeyEventArgs, KeyModifiers};
use ferroui_base::interactivity::{RoutedEventHandlerToken, RoutingStrategies};
use ferroui_base::threading::{Dispatcher, DispatcherPriority};
use ferroui_base::{Ref, WeakRef};
use ferroui_controls::{Control, Grid, ScrollViewer, TextBox, TopLevel};
use ferroui_view::prelude::*;
use reader_domain::{View, ViewMode};
use reader_state::AppViewModel;
use std::{cell::RefCell, rc::Rc};

pub fn attach(root: &Ref<Grid>, vm: Rc<AppViewModel>, ui: Rc<Session>) {
    let route = Rc::new(RefCell::new(
        None::<(WeakRef<TopLevel>, RoutedEventHandlerToken)>,
    ));
    let connect: Rc<dyn Fn()> = {
        let route = route.clone();
        let root = root.downgrade();
        let vm = Rc::downgrade(&vm);
        let ui = Rc::downgrade(&ui);
        Rc::new(move || {
            clear(&route);
            let Some(root) = root.upgrade().filter(|r| r.is_attached_to_visual_tree()) else {
                return;
            };
            let Some(top) = TopLevel::get_top_level(Some(&root)) else {
                return;
            };
            let model = vm.clone();
            let session = ui.clone();
            let token = top.add_handler_with(
                InputElement::key_down_event(),
                move |_, event| {
                    if let (Some(vm), Some(ui)) = (model.upgrade(), session.upgrade()) {
                        handle(&vm, &ui, event);
                    }
                },
                RoutingStrategies::BUBBLE,
                false,
            );
            *route.borrow_mut() = Some((top.downgrade(), token));
        })
    };
    let mounted = connect.clone();
    let attached = root.attached_to_visual_tree(move |_| mounted());
    let detached_route = route.clone();
    let detached = root.detached_from_visual_tree(move |_| clear(&detached_route));
    connect();
    on_cleanup(move || {
        attached.dispose();
        detached.dispose();
        clear(&route);
    });
}
fn clear(route: &RefCell<Option<(WeakRef<TopLevel>, RoutedEventHandlerToken)>>) {
    if let Some((top, token)) = route.borrow_mut().take() {
        if let Some(top) = top.upgrade() {
            top.remove_handler(InputElement::key_down_event(), token);
        }
    }
}
fn handle(vm: &AppViewModel, ui: &Session, event: &KeyEventArgs) {
    if event.key == Key::Escape {
        ui.close();
        if vm.sidebar_open() {
            vm.set_sidebar_open(false);
        } else {
            vm.select(None);
        }
        event.set_handled(true);
        return;
    }
    if ui.panel.get_untracked() != Panel::None {
        return;
    }
    let modifiers = event.key_modifiers;
    let ctrl = modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::META);
    let shift = modifiers.contains(KeyModifiers::SHIFT);
    let editing = event.source().is_some_and(|source| {
        source.cast::<TextBox>().is_some()
            || source.cast::<Control>().is_some_and(|c| {
                c.get_visual_ancestors()
                    .any(|a| a.cast::<TextBox>().is_some())
            })
    });
    if editing {
        return;
    }
    if ctrl {
        if event.key == Key::Z {
            vm.undo();
            event.set_handled(true);
        }
        return;
    }
    if ui.prefix.replace(false)
        && ui
            .prefix_at
            .get()
            .is_some_and(|at| at.elapsed().as_secs_f64() < 1.5)
    {
        let view = match event.key {
            Key::H => Some(View::Home),
            Key::A => Some(View::All),
            Key::S => Some(View::Starred),
            Key::T => Some(View::Trends),
            Key::D => Some(View::Explore),
            _ => None,
        };
        if let Some(view) = view {
            vm.navigate(view);
            event.set_handled(true);
            return;
        }
    }
    match event.key {
        Key::G => {
            ui.prefix.set(true);
            ui.prefix_at.set(Some(std::time::Instant::now()));
        }
        Key::J if !shift => vm.next_article(1),
        Key::N if !shift => vm.scan_article(1),
        Key::K if !shift => vm.next_article(-1),
        Key::P if !shift => vm.scan_article(-1),
        Key::N | Key::P if shift => {
            let feeds = vm.feeds();
            if feeds.is_empty() {
                return;
            }
            let current = feeds
                .iter()
                .position(|f| vm.view() == View::Feed(f.id))
                .unwrap_or(0);
            let index = if event.key == Key::N {
                (current + 1).min(feeds.len() - 1)
            } else {
                current.saturating_sub(1)
            };
            vm.navigate(View::Feed(feeds[index].id));
        }
        Key::S if shift => {
            if let Some(id) = vm.selected().or(vm.cursor()) {
                vm.share_article(id);
            }
        }
        Key::T if shift => {
            if let Some(id) = vm.selected().or(vm.cursor()) {
                vm.share_article(id);
            }
        }
        Key::S => {
            if let Some(id) = vm.selected() {
                vm.toggle_star(id);
            }
        }
        Key::M => {
            if let Some(id) = vm.selected() {
                vm.toggle_read(id);
            }
        }
        Key::V => {
            if let Some(id) = vm.selected() {
                vm.open_article(id);
            }
        }
        Key::E => {
            if let Some(id) = vm.selected() {
                vm.email_article(id);
            }
        }
        Key::T => {
            if let Some(id) = vm.selected() {
                ui.show(Panel::Tags(id));
            }
        }
        Key::R => vm.refresh(),
        Key::A if shift => {
            if vm.state().settings.confirm_mark_all {
                ui.show(Panel::MarkAll);
            } else {
                vm.mark_all_read(None);
            }
        }
        Key::A => ui.show(Panel::Subscribe),
        Key::D1 => vm.set_view_mode(ViewMode::Expanded),
        Key::D2 => vm.set_view_mode(ViewMode::List),
        Key::U => {
            if vm.viewport() < 700.0 {
                vm.toggle_sidebar();
            } else {
                ui.desktop_sidebar.update(|v| *v = !*v);
            }
        }
        Key::F => ui.focus_mode.update(|v| *v = !*v),
        Key::O | Key::Return => {
            if vm.selected().is_some() && vm.selected() == vm.cursor() {
                vm.select(None);
            } else if let Some(id) = vm.cursor() {
                vm.select(Some(id));
            } else {
                vm.next_article(1);
            }
        }
        Key::OemQuestion if shift => ui.show(Panel::Shortcuts),
        Key::OemQuestion => {
            ui.focus_mode.set(false);
            let node = ui.search_input;
            Dispatcher::ui_thread().post_local(
                move || {
                    if let Some(input) = node.get_untracked() {
                        input.focus();
                        input.select_all();
                    }
                },
                DispatcherPriority::BACKGROUND,
            );
        }
        Key::Space => {
            if let Some(list) = ui.article_list.get_untracked() {
                if let Some(scroll) = list
                    .get_visual_descendants()
                    .find_map(|v| v.cast::<ScrollViewer>())
                {
                    if shift {
                        scroll.page_up();
                    } else {
                        scroll.page_down();
                    }
                }
            }
        }
        _ => return,
    }
    event.set_handled(true);
}
