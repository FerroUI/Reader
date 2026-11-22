mod background;
mod discovery;
mod files;
mod persistence;
mod services;
mod transport;
mod writer;

use chrono::Utc;
use ferroui_base::threading::{DispatcherPriority, DispatcherTimer};
use ferroui_controls::{AppBuilder, Application, Control, Window};
use ferroui_desktop::AppBuilderDesktopExtensions;
use ferroui_fonts_inter::AppBuilderExtension;
use reader_domain::AppState;
use reader_state::AppViewModel;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let demo = args.iter().any(|arg| arg == "--demo");
    AppBuilder::configure::<Application>()
        .use_platform_detect()
        .with_inter_font()
        .with(Rc::new(reader_fonts::font_options()))
        .configure_fonts(|manager| {
            reader_fonts::install_fonts(manager, reader_fonts::BUNDLED_FONTS)
                .expect("the bundled Reader fonts are valid");
        })
        .start(move |app, _| {
            reader_controls::install_theme(app);
            let now = Utc::now().timestamp();
            let directory = persistence::data_directory();
            let loaded = if demo {
                persistence::Loaded { state: AppState::sample(now), warning: None, writable: true }
            } else {
                persistence::load(&directory, now)
            };
            let writer = Rc::new(writer::Writer::new(directory.clone(), loaded.writable, demo));
            let saving = writer.clone();
            let vm = AppViewModel::new(loaded.state, now, Rc::new(move |json| saving.submit(json)));
            let window = Window::new();
            window.set_title(Some("Reader".into()));
            window.set_width(1360.0);
            window.set_height(860.0);
            window.set_min_width(360.0);
            window.set_min_height(420.0);
            services::install(&vm, &window, directory);
            if let Some(warning) = loaded.warning { vm.report_error(warning); }
            window.set_content(Some(Control::boxed(&reader_ui::create_view(vm.clone()))));
            let layout_vm = Rc::downgrade(&vm);
            let layout_window = window.downgrade();
            window.layout_updated(move || {
                if let (Some(vm), Some(window)) = (layout_vm.upgrade(), layout_window.upgrade()) {
                    vm.set_viewport(window.client_size().width);
                }
            }).detach();

            let closing_vm = Rc::downgrade(&vm);
            let closing_writer = writer.clone();
            let discard_on_next_close = Cell::new(false);
            window.closing(move |event| {
                let Some(vm) = closing_vm.upgrade() else { return; };
                let saved = vm.flush().and_then(|_| closing_writer.wait());
                if let Err(error) = saved {
                    if !discard_on_next_close.replace(true) {
                        event.set_cancel(true);
                        vm.report_error(format!("Could not save: {error}. Export a backup, or close again to exit without saving."));
                    }
                }
            }).detach();

            let ticking = Rc::downgrade(&vm);
            let ticking_writer = writer.clone();
            let reported_error = RefCell::new(None);
            let timer = DispatcherTimer::run(move || {
                let Some(vm) = ticking.upgrade() else { return false; };
                vm.set_now(Utc::now().timestamp());
                let error = ticking_writer.error();
                if error != *reported_error.borrow() {
                    if let Some(error) = &error {
                        vm.report_error(format!("Could not save your library: {error}. Export a backup before closing."));
                    }
                    *reported_error.borrow_mut() = error;
                }
                true
            }, Duration::from_secs(1), DispatcherPriority::BACKGROUND);
            let timer = RefCell::new(Some(timer));
            window.closed(move || {
                if let Some(timer) = timer.borrow_mut().take() { timer.dispose(); }
                writer.stop();
            }).detach();
            app.run_window(window);
        }, &args);
}
