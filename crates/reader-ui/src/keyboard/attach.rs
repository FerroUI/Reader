use crate::session::Session;
use ferroui_base::input::InputElement;
use ferroui_base::interactivity::{RoutedEventHandlerToken, RoutingStrategies};
use ferroui_base::{Ref, WeakRef};
use ferroui_controls::{Grid, TopLevel};
use ferroui_view::on_cleanup;
use reader_state::AppViewModel;
use std::{cell::RefCell, rc::Rc};

struct Route {
    root: WeakRef<TopLevel>,
    key: RoutedEventHandlerToken,
    remove_input: Box<dyn FnOnce()>,
}

pub fn attach(root: &Ref<Grid>, vm: Rc<AppViewModel>, ui: Rc<Session>) {
    let route = Rc::new(RefCell::new(None::<Route>));
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
            let remove_input = crate::gestures::attach(&top, ui.clone());
            let model = vm.clone();
            let session = ui.clone();
            let key = top.add_handler_with(
                InputElement::key_down_event(),
                move |_, event| {
                    if let (Some(vm), Some(ui)) = (model.upgrade(), session.upgrade()) {
                        super::handle(&vm, &ui, event);
                    }
                },
                RoutingStrategies::BUBBLE,
                false,
            );
            *route.borrow_mut() = Some(Route {
                root: top.downgrade(),
                key,
                remove_input,
            });
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
fn clear(route: &RefCell<Option<Route>>) {
    let old = route.borrow_mut().take();
    if let Some(old) = old {
        (old.remove_input)();
        if let Some(root) = old.root.upgrade() {
            root.remove_handler(InputElement::key_down_event(), old.key);
        }
    }
}
