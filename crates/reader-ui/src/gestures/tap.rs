use ferroui_base::input::{InputElement, MouseButton, PointerType};
use ferroui_base::Ref;
use ferroui_controls::Control;
use ferroui_view::on_cleanup;
use reader_domain::Id;
use reader_state::AppViewModel;
use std::rc::Rc;

/// Mouse selection remains immediate. Touch/pen selection runs only after the
/// top-level contact guard and native scroll recognizers accept a release tap.
/// A mounted hook outlives its recycled row: resolve the current article tag at
/// input time instead of retaining the item that first created this control.
pub(crate) fn attach_article_tap(control: &Ref<Control>, vm: Rc<AppViewModel>) {
    let mouse_vm = vm.clone();
    let mouse_control = control.downgrade();
    let pressed = control.add_handler(InputElement::pointer_pressed_event(), move |_, event| {
        if event.pointer().type_() == PointerType::Mouse
            && event.properties().is_left_button_pressed
        {
            if let Some(id) = mouse_control.upgrade().as_ref().and_then(current_article) {
                mouse_vm.select(Some(id));
            }
        }
    });
    let touch_control = control.downgrade();
    let released = control.add_handler(InputElement::pointer_released_event(), move |_, event| {
        if event.pointer().type_() != PointerType::Mouse
            && event.pointer().is_primary()
            && event.initial_press_mouse_button() == MouseButton::Left
        {
            if let Some(id) = touch_control.upgrade().as_ref().and_then(current_article) {
                vm.select(Some(id));
            }
        }
    });
    let weak = control.downgrade();
    on_cleanup(move || {
        if let Some(control) = weak.upgrade() {
            control.remove_handler(InputElement::pointer_pressed_event(), pressed);
            control.remove_handler(InputElement::pointer_released_event(), released);
        }
    });
}

fn current_article(control: &Ref<Control>) -> Option<Id> {
    control.get_self_and_visual_ancestors().find_map(|visual| {
        let control = visual.cast::<Control>()?;
        if !control.has_name("ArticleRow") && !control.has_name("ArticleEntry") {
            return None;
        }
        control
            .tag()
            .and_then(|tag| tag.downcast_ref::<Id>().copied())
    })
}
