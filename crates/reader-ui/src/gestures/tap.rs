use ferroui_base::input::{InputElement, MouseButton, PointerType};
use ferroui_base::interactivity::RoutingStrategies;
use ferroui_base::{FerroObject, Ref};
use ferroui_controls::{Button, Control, RichTextBlock, SelectableTextBlock, TextBox};
use ferroui_view::on_cleanup;
use reader_domain::Id;
use reader_state::AppViewModel;
use std::rc::Rc;

/// Mouse selection remains immediate. Touch/pen selection runs only after the
/// top-level contact guard and native scroll recognizers accept a release tap.
/// A mounted hook outlives its recycled row: resolve the current article tag at
/// input time instead of retaining the item that first created this control.
pub(crate) fn attach_article_tap(control: &Ref<Control>, vm: Rc<AppViewModel>) {
    attach_tap(control, vm, false);
}

/// The compact headline toggles list expansion, while a body tap only selects.
/// Resolve the article through the live visual tree after row recycling.
pub(crate) fn attach_headline_tap(control: &Ref<Control>, vm: Rc<AppViewModel>) {
    attach_tap(control, vm, true);
}

fn attach_tap(control: &Ref<Control>, vm: Rc<AppViewModel>, headline: bool) {
    let mouse_vm = vm.clone();
    let mouse_control = control.downgrade();
    let pressed = control.add_handler_with(
        InputElement::pointer_pressed_event(),
        move |_, event| {
            if event.pointer().type_() == PointerType::Mouse
                && event.properties().is_left_button_pressed
                && (!event.handled() || body_text_source(event.source()).is_some())
            {
                if let Some(id) = mouse_control.upgrade().as_ref().and_then(current_article) {
                    select(&mouse_vm, id, headline);
                }
            }
        },
        RoutingStrategies::BUBBLE,
        true,
    );
    let text_vm = vm.clone();
    let text_control = control.downgrade();
    // The top-level guard tunnels first and handles rejected contacts. Observe
    // only its accepted rich-text releases before RichTextBlock handles them
    // for native selection or hyperlink navigation. Never consume the event.
    let text_released = control.add_handler_with(
        InputElement::pointer_released_event(),
        move |_, event| {
            if event.pointer().type_() != PointerType::Mouse
                && event.pointer().is_primary()
                && event.initial_press_mouse_button() == MouseButton::Left
                && body_text_source(event.source())
                    .is_some_and(|text| text.cast::<RichTextBlock>().is_some())
            {
                if let Some(id) = text_control.upgrade().as_ref().and_then(current_article) {
                    select(&text_vm, id, headline);
                }
            }
        },
        RoutingStrategies::TUNNEL,
        false,
    );
    let touch_control = control.downgrade();
    let released = control.add_handler(InputElement::pointer_released_event(), move |_, event| {
        if event.pointer().type_() != PointerType::Mouse
            && event.pointer().is_primary()
            && event.initial_press_mouse_button() == MouseButton::Left
        {
            if let Some(id) = touch_control.upgrade().as_ref().and_then(current_article) {
                select(&vm, id, headline);
            }
        }
    });
    let weak = control.downgrade();
    on_cleanup(move || {
        if let Some(control) = weak.upgrade() {
            control.remove_handler(InputElement::pointer_pressed_event(), pressed);
            control.remove_handler(InputElement::pointer_released_event(), text_released);
            control.remove_handler(InputElement::pointer_released_event(), released);
        }
    });
}

fn select(vm: &AppViewModel, id: Id, headline: bool) {
    if headline
        && vm.state().settings.view_mode == reader_domain::ViewMode::List
        && vm.selected() == Some(id)
    {
        vm.select(None);
    } else {
        vm.select(Some(id));
    }
}

/// Native text controls handle their press to establish selection/capture.
/// Observe that completed native handling without replacing or consuming it.
/// Other handled sources, including editor and button descendants, stay local.
fn body_text_source(source: Option<Ref<FerroObject>>) -> Option<Ref<Control>> {
    let text = source?.cast::<Control>()?;
    (text.has_name("ArticleBodyText")
        && (text.cast::<SelectableTextBlock>().is_some() || text.cast::<RichTextBlock>().is_some())
        && !text.get_visual_ancestors().any(|ancestor| {
            ancestor.cast::<Button>().is_some() || ancestor.cast::<TextBox>().is_some()
        }))
    .then_some(text)
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
