use ferroui_base::{ferro_class, ferro_impl_classes, instantiate, FerroObjectImpl, Ref};
use ferroui_controls::{Application, ApplicationImpl, ApplicationImplExt, NewApplication};
use reader_state::{AppViewModel, HostServices};
use std::{cell::RefCell, rc::Rc};

ferroui_base::ferro_thread_local! {
    static VIEW_MODEL: RefCell<Option<Rc<AppViewModel>>> = const { RefCell::new(None) };
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
        crate::input_hints::install();
    }
    fn on_framework_initialization_completed(this: &Self) {
        let now = crate::host::now() as i64;
        let loaded = crate::startup::read_state();
        let vm = AppViewModel::new(
            loaded.state,
            now,
            Rc::new(|json| crate::host::save(json).map_err(crate::host::error_message)),
        );
        vm.set_viewport_size(crate::host::width(), crate::host::height());
        vm.set_services(HostServices {
            request: Rc::new(|request| {
                if let Ok(json) = serde_json::to_string(&request) {
                    crate::host::request(&json);
                }
            }),
        });
        VIEW_MODEL.with(|slot| *slot.borrow_mut() = Some(vm.clone()));
        reader_ui::set_navigation_notifier(Rc::new(|| crate::host::navigation_changed()));
        let save_failed = loaded.changed && vm.persist_initial_state().is_err();
        if let Some(error) = loaded.error {
            vm.report_error(error);
        }
        if let Some(lifetime) = this
            .application_lifetime()
            .as_ref()
            .and_then(|l| l.as_single_view_application_lifetime())
        {
            lifetime.set_main_view(Some(reader_ui::create_view(vm.clone())));
        }
        Self::parent_on_framework_initialization_completed(this);
        if !save_failed {
            vm.refresh_initial_feeds();
        }
    }
}
