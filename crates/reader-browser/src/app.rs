use ferroui_base::{ferro_class, ferro_impl_classes, instantiate, FerroObjectImpl, Ref};
use ferroui_controls::{Application, ApplicationImpl, ApplicationImplExt, NewApplication};
use reader_domain::AppState;
use reader_state::{AppViewModel, HostServices};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

ferroui_base::ferro_thread_local! {
    static VIEW_MODEL: RefCell<Option<Rc<AppViewModel>>> = const { RefCell::new(None) };
    static DEMO: Cell<bool> = const { Cell::new(false) };
}
pub fn set_demo(value: bool) {
    DEMO.with(|d| d.set(value));
}
pub fn view_model() -> Option<Rc<AppViewModel>> {
    VIEW_MODEL.with(|vm| vm.borrow().clone())
}

#[repr(C)]
pub struct App {
    base: Application,
}
ferro_class!(App: Application);
ferro_impl_classes!(App: FerroObjectImpl);
impl NewApplication for App {
    fn new_application() -> Ref<Self> {
        instantiate(Self {
            base: Application::construct(),
        })
    }
}
impl ApplicationImpl for App {
    fn initialize(this: &Self) {
        Self::parent_initialize(this);
        reader_controls::install_theme(this);
    }
    fn on_framework_initialization_completed(this: &Self) {
        let now = crate::host::now() as i64;
        let (state, error) = read_state(now);
        let vm = AppViewModel::new(
            state,
            now,
            Rc::new(|json| {
                if DEMO.with(|d| d.get()) {
                    return Ok(());
                }
                crate::host::save(json).map_err(crate::host::error_message)
            }),
        );
        vm.set_viewport(crate::host::width());
        vm.set_services(HostServices {
            request: Rc::new(|request| {
                if let Ok(json) = serde_json::to_string(&request) {
                    crate::host::request(&json);
                }
            }),
        });
        VIEW_MODEL.with(|slot| *slot.borrow_mut() = Some(vm.clone()));
        if let Some(error) = error {
            vm.report_error(error);
        }
        if let Some(lifetime) = this
            .application_lifetime()
            .as_ref()
            .and_then(|l| l.as_single_view_application_lifetime())
        {
            lifetime.set_main_view(Some(reader_ui::create_view(vm)));
        }
        Self::parent_on_framework_initialization_completed(this);
    }
}
fn read_state(now: i64) -> (AppState, Option<String>) {
    if DEMO.with(|d| d.get()) {
        return (AppState::sample(now), None);
    }
    match crate::host::load() {
        Ok(json) if json.trim().is_empty() => (AppState::sample(now), None),
        Ok(json) => match AppState::from_json(&json) {
            Ok(state) => (state, None),
            Err(error) => {
                let recovery = if crate::host::preserve_recovery(&json) {
                    "A recovery copy was preserved."
                } else {
                    "The original saved data is protected from overwriting. Export a backup before reloading."
                };
                (
                    AppState::new(),
                    Some(format!(
                        "Your saved library could not be read. {recovery} {error}"
                    )),
                )
            }
        },
        Err(error) => (AppState::new(), Some(crate::host::error_message(error))),
    }
}
