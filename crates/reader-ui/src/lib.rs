//! RSX components over a retained MVVM model and virtualized article projection.
mod article;
mod content;
mod dialogs;
mod drag;
mod format;
mod header;
mod home;
mod images;
mod keyboard;
mod menus;
mod reading;
mod rows;
mod session;
mod settings;
mod shell;
mod sidebar;
mod toolbar;
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
