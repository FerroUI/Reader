use ferroui_base::input::{InputElement, MouseButton, PointerType};
use ferroui_base::Ref;
use ferroui_controls::Control;
use ferroui_view::on_cleanup;
use reader_domain::Id;
use reader_state::AppViewModel;
use std::rc::Rc;

/// Mouse selection remains immediate. Touch/pen selection runs only after the
/// top-level contact guard and native scroll recognizers accept a release tap.
pub(crate) fn attach_article_tap(control: &Ref<Control>, vm: Rc<AppViewModel>, id: Id) {
    let mouse_vm = vm.clone();
    let pressed = control.add_handler(InputElement::pointer_pressed_event(), move |_, event| {
        if event.pointer().type_() == PointerType::Mouse
            && event.properties().is_left_button_pressed
        {
            mouse_vm.select(Some(id));
        }
    });
    let released = control.add_handler(InputElement::pointer_released_event(), move |_, event| {
        if event.pointer().type_() != PointerType::Mouse
            && event.pointer().is_primary()
            && event.initial_press_mouse_button() == MouseButton::Left
        {
            vm.select(Some(id));
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
