use ferroui_base::input::{InputElement, Key, KeyModifiers};
use ferroui_base::interactivity::RoutingStrategies;
use ferroui_base::Ref;
use ferroui_controls::Button;
use ferroui_view::on_cleanup;

/// Browser platform defaults only register Apps. Make the standard keyboard
/// context shortcuts available on subscription rows on every host, with the
/// same item focus and pointer suppression used by toolbar dropdowns.
pub(crate) fn attach_context_keys(button: &Ref<Button>) {
    let weak = button.downgrade();
    let key = button.add_handler_with(
        InputElement::key_up_event(),
        move |_, event| {
            let context = (event.key == Key::F10 && event.key_modifiers == KeyModifiers::SHIFT)
                || (event.key == Key::Apps && event.key_modifiers.is_empty());
            if !context {
                return;
            }
            let Some(button) = weak.upgrade().filter(|button| {
                button.is_focused() && event.is_source(button) && button.context_menu().is_some()
            }) else {
                return;
            };
            // Intercept key-up before Control's native bubble handler. Hosts
            // that already register Shift+F10 must not open the menu twice.
            event.set_handled(true);
            super::opening::with_selection(&button, true);
        },
        RoutingStrategies::TUNNEL,
        false,
    );
    let weak = button.downgrade();
    on_cleanup(move || {
        if let Some(button) = weak.upgrade() {
            button.remove_handler(InputElement::key_up_event(), key);
        }
    });
}
