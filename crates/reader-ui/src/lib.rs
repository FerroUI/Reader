//! RSX components over a retained MVVM model and virtualized article projection.
mod article;
mod content;
mod dialogs;
mod drag;
mod empty;
mod format;
mod gestures;
mod header;
mod home;
mod images;
mod keyboard;
mod menus;
mod reading;
mod responsive;
mod rows;
mod session;
mod settings;
mod shell;
mod sidebar;
mod toolbar;
mod transient;
use ferroui_base::Ref;
use ferroui_controls::Control;
use ferroui_reactive::ObservableExt;
use ferroui_reactiveui::IReactiveNotifyPropertyChanged;
use ferroui_view::prelude::*;
use reader_state::AppViewModel;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
ferroui_base::ferro_thread_local! {
    static PRESENTATION: RefCell<Option<(Weak<session::Session>,Weak<rows::Rows>)>> = const {RefCell::new(None)};
    static NAVIGATION_NOTIFIER: RefCell<Option<Rc<dyn Fn()>>> = const {RefCell::new(None)};
}
pub use images::register_image;

pub fn create_view(vm: Rc<AppViewModel>) -> Ref<Control> {
    ComponentHost::with_view(move || {
        images::initialize();
        let revision = signal(vm.revision());
        let session = Rc::new(session::Session::new());
        let rows = Rc::new(rows::Rows::new(vm.clone(), session.clone()));
        rows.synchronize(&vm);
        PRESENTATION
            .with(|p| *p.borrow_mut() = Some((Rc::downgrade(&session), Rc::downgrade(&rows))));
        let observed = vm.clone();
        let observed_rows = rows.clone();
        let subscription = vm.changed().subscribe_next(move |_| {
            observed_rows.synchronize(&observed);
            revision.set(observed.revision());
            notify_navigation();
        });
        on_cleanup(move || subscription.dispose());
        let vm = vm.clone();
        rerender(move || {
            revision.get();
            shell::view(vm.clone(), session.clone(), rows.clone())
        })
    })
    .upcast()
}

/// Browser history observes presentation changes without polling or owning UI state.
pub fn set_navigation_notifier(notifier: Rc<dyn Fn()>) {
    NAVIGATION_NOTIFIER.with(|current| *current.borrow_mut() = Some(notifier));
    notify_navigation();
}
pub(crate) fn notify_navigation() {
    let callback = NAVIGATION_NOTIFIER.with(|current| current.borrow().clone());
    if let Some(callback) = callback {
        callback();
    }
}
fn current_session() -> Option<Rc<session::Session>> {
    PRESENTATION.with(|presentation| presentation.borrow().as_ref()?.0.upgrade())
}
pub fn has_transient_ui() -> bool {
    transient::has_menu()
        || current_session()
            .is_some_and(|session| session.panel.get_untracked() != session::Panel::None)
}
pub fn dismiss_transient_ui() -> bool {
    if transient::dismiss_menu() {
        return true;
    }
    let Some(session) =
        current_session().filter(|session| session.panel.get_untracked() != session::Panel::None)
    else {
        return false;
    };
    session.close();
    true
}

/// Bring a virtualized article into view after keyboard or host navigation.
pub fn scroll_to_article(id: reader_domain::Id) {
    PRESENTATION.with(|p| {
        if let Some((session, rows)) = p.borrow().as_ref() {
            if let (Some(session), Some(rows)) = (session.upgrade(), rows.upgrade()) {
                if let (Some(list), Some(index)) =
                    (session.article_list.get_untracked(), rows.index(id))
                {
                    session
                        .programmatic_scroll
                        .set(Some(std::time::Instant::now()));
                    list.scroll_into_view(index);
                }
            }
        }
    });
}
pub fn focus_search() {
    PRESENTATION.with(|p| {
        if let Some((session, _)) = p.borrow().as_ref() {
            if let Some(session) = session.upgrade() {
                if let Some(field) = session.search_input.get_untracked() {
                    field.focus();
                    field.select_all();
                }
            }
        }
    });
}
