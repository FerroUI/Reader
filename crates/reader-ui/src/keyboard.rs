mod attach;
use crate::session::{Panel, Session};
pub use attach::attach;
use ferroui_base::input::{Key, KeyEventArgs, KeyModifiers};
use ferroui_base::threading::{Dispatcher, DispatcherPriority};
use ferroui_controls::{Control, ScrollViewer, TextBox};
use reader_domain::{View, ViewMode};
use reader_state::AppViewModel;

fn handle(vm: &AppViewModel, ui: &Session, event: &KeyEventArgs) {
    let modifiers = event.key_modifiers;
    let ctrl = modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::META);
    let shift = modifiers.contains(KeyModifiers::SHIFT);
    // Alt/AltGr belong to the platform and text input, including non-Latin layouts.
    if modifiers.contains(KeyModifiers::ALT) {
        ui.cancel_prefix();
        return;
    }
    if event.key == Key::Escape {
        if ui.panel.get_untracked() != Panel::None {
            ui.close();
            event.set_handled(true);
            return;
        }
        ui.cancel_prefix();
        if vm.sidebar_open() {
            vm.set_sidebar_open(false);
            ui.restore_article_focus();
        } else {
            vm.select(None);
        }
        event.set_handled(true);
        return;
    }
    if ui.panel.get_untracked() != Panel::None {
        ui.cancel_prefix();
        return;
    }
    if editing_source(event) {
        ui.cancel_prefix();
        return;
    }
    if vm.viewport() < 700.0 && vm.sidebar_open() {
        ui.cancel_prefix();
        if !ctrl && shift && tree_key(vm, ui, event.key) {
            event.set_handled(true);
            return;
        }
        if !ctrl && !shift && event.key == Key::U {
            vm.set_sidebar_open(false);
            ui.restore_article_focus();
            event.set_handled(true);
        }
        return;
    }
    if ctrl {
        ui.cancel_prefix();
        if event.key == Key::Z && !shift {
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
        if event.key == Key::T && !shift {
            ui.show(Panel::GoToTag);
            event.set_handled(true);
            return;
        }
        if event.key == Key::U {
            ui.show(Panel::GoToSubscription);
            event.set_handled(true);
            return;
        }
        let view = match event.key {
            Key::H => Some(View::Home),
            Key::A => Some(View::All),
            Key::S => Some(View::Starred),
            Key::T if shift => Some(View::Trends),
            Key::D | Key::E => Some(View::Explore),
            _ => None,
        };
        if let Some(view) = view {
            vm.navigate(view);
            event.set_handled(true);
            return;
        }
    }
    // A modified letter is a separate shortcut in Reader, not the unmodified
    // command with Shift ignored. Keep only the explicitly supported chords.
    if shift
        && !matches!(
            event.key,
            Key::N
                | Key::P
                | Key::J
                | Key::K
                | Key::O
                | Key::X
                | Key::S
                | Key::T
                | Key::A
                | Key::U
                | Key::OemQuestion
                | Key::Space
                | Key::OemPlus
                | Key::Add
        )
    {
        ui.cancel_prefix();
        return;
    }
    if !shift && matches!(event.key, Key::J | Key::K | Key::N | Key::P) {
        if let Some(list) = ui.article_list.get_untracked() {
            list.focus();
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
        Key::N | Key::P | Key::J | Key::K | Key::O | Key::X if shift => {
            tree_key(vm, ui, event.key);
        }
        Key::S if shift => {
            if let Some(id) = vm.cursor().or(vm.selected()) {
                vm.share_article(id);
            }
        }
        Key::T if shift => {
            if let Some(id) = vm.cursor().or(vm.selected()) {
                vm.share_article(id);
            }
        }
        Key::S => {
            if let Some(id) = vm.cursor().or(vm.selected()) {
                vm.toggle_star(id);
            }
        }
        Key::M => {
            if let Some(id) = vm.cursor().or(vm.selected()) {
                vm.toggle_read(id);
            }
        }
        Key::V => {
            if let Some(id) = vm.cursor().or(vm.selected()) {
                vm.open_article(id);
            }
        }
        Key::E => {
            if let Some(id) = vm.cursor().or(vm.selected()) {
                vm.email_article(id);
            }
        }
        Key::T => {
            if let Some(id) = vm.cursor().or(vm.selected()) {
                ui.show(Panel::Tags(id));
            }
        }
        Key::OemPlus | Key::Add | Key::OemMinus | Key::Subtract => {
            let mut settings = vm.state().settings.clone();
            let larger = matches!(event.key, Key::OemPlus | Key::Add);
            settings.font_size =
                (settings.font_size + if larger { 1.0 } else { -1.0 }).clamp(11.0, 24.0);
            vm.set_settings(settings);
        }
        Key::R => vm.refresh(),
        Key::A if shift => {
            if vm.should_confirm_mark_all() {
                ui.show(Panel::MarkAll(None));
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
            crate::reading::user_scroll();
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

pub(crate) fn editing_source(event: &KeyEventArgs) -> bool {
    event.source().is_some_and(|source| {
        source.cast::<TextBox>().is_some()
            || source.cast::<Control>().is_some_and(|control| {
                control
                    .get_visual_ancestors()
                    .any(|ancestor| ancestor.cast::<TextBox>().is_some())
            })
    })
}

fn tree_key(vm: &AppViewModel, ui: &Session, key: Key) -> bool {
    use crate::sidebar::cursor;
    match key {
        Key::N | Key::J => cursor::move_cursor(vm, ui, true),
        Key::P | Key::K => cursor::move_cursor(vm, ui, false),
        Key::O => cursor::open_cursor(vm, ui),
        Key::X => cursor::toggle(vm, ui),
        _ => return false,
    }
    true
}
