use super::controls::{check, choice, choices, heading, help};
use crate::session::{Panel, Session};
use ferroui_view::prelude::*;
use reader_controls::{action, separator};
use reader_domain::{Density, View};
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn view(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let settings = vm.state().settings.clone();
    let touch = crate::responsive::touch(&vm);
    let mut reading = stack_panel().spacing(4.0).child(heading("Reading"));
    for (name, label, current, field) in [
        (
            "MarkReadOnScroll",
            "Mark items as read when scrolling in expanded view",
            settings.mark_read_on_scroll,
            0,
        ),
        (
            "ConfirmMarkAll",
            "Confirm before marking a stream with more than 50 unread items as read",
            settings.confirm_mark_all,
            1,
        ),
        (
            "ShowAllSubscriptions",
            "Show all subscriptions, including those with no unread items",
            settings.show_all_subscriptions,
            2,
        ),
    ] {
        let model = vm.clone();
        reading = reading.child(check(label, name, current, touch, move |value| {
            let mut settings = model.state().settings.clone();
            let setting = match field {
                0 => &mut settings.mark_read_on_scroll,
                1 => &mut settings.confirm_mark_all,
                _ => &mut settings.show_all_subscriptions,
            };
            if *setting != value {
                *setting = value;
                model.set_settings(settings);
            }
        }));
    }
    let mut starts = vec![
        ("Home".to_owned(), View::Home),
        ("All items".to_owned(), View::All),
        ("Starred items".to_owned(), View::Starred),
        ("Trends".to_owned(), View::Trends),
        ("Explore".to_owned(), View::Explore),
    ];
    starts.extend(
        vm.folders()
            .into_iter()
            .map(|name| (name.clone(), View::Folder(name))),
    );
    starts.extend(
        vm.feeds()
            .into_iter()
            .map(|feed| (feed.title, View::Feed(feed.id))),
    );
    let start_label = starts
        .iter()
        .find(|(_, view)| view == &settings.start_page)
        .map(|(label, _)| label.clone())
        .unwrap_or_else(|| "Home".into());
    let start_items = starts
        .into_iter()
        .map(|(label, view)| {
            let model = vm.clone();
            let selected = settings.start_page == view;
            crate::menus::checked(&label, selected, move || {
                let mut settings = model.state().settings.clone();
                settings.start_page = view.clone();
                model.set_settings(settings);
            })
        })
        .collect();
    let densities = [
        ("Comfortable", Density::Comfortable),
        ("Cozy", Density::Cozy),
        ("Compact", Density::Compact),
    ]
    .into_iter()
    .map(|(label, density)| {
        let model = vm.clone();
        choice(
            "density",
            label,
            settings.density == density,
            touch,
            move || {
                if model.state().settings.density != density {
                    model.set_density(density);
                }
            },
        )
    })
    .collect();
    let mut fonts: Vec<Element> = [
        ("Small", 13.0),
        ("Medium", 14.0),
        ("Large", 16.0),
        ("Extra large", 18.0),
    ]
    .into_iter()
    .map(|(label, size)| {
        let model = vm.clone();
        choice(
            "font",
            label,
            (settings.font_size - size).abs() < 0.1,
            touch,
            move || {
                let mut settings = model.state().settings.clone();
                if (settings.font_size - size).abs() >= 0.1 {
                    settings.font_size = size;
                    model.set_settings(settings);
                }
            },
        )
    })
    .collect();
    if ![13.0, 14.0, 16.0, 18.0]
        .into_iter()
        .any(|size| (settings.font_size - size).abs() < 0.1)
    {
        let model = vm.clone();
        let size = settings.font_size;
        fonts.push(choice(
            "font",
            &format!("Custom ({size} px)"),
            true,
            touch,
            move || {
                let mut settings = model.state().settings.clone();
                if (settings.font_size - size).abs() >= 0.1 {
                    settings.font_size = size;
                    model.set_settings(settings);
                }
            },
        ));
    }
    let customize = ui.clone();
    rsx! {StackPanel {name:"ReaderPreferences",spacing:16.0,
        {help("Reading and display preferences are saved automatically.")}
        {reading}
        StackPanel {spacing:8.0,
            {heading("Start page")}
            {crate::menus::button(&vm,&start_label,start_items)}
        }
        {separator()}
        StackPanel {spacing:8.0,{heading("Display density")}{choices("density",densities)}}
        StackPanel {spacing:8.0,{heading("Article text size")}{choices("font",fonts)}}
        {separator()}
        StackPanel {spacing:8.0,
            {heading("Send to")}
            {help("Choose the services shown in an article's Send to menu.")}
            {action("Customize Send to",move||customize.show(Panel::SendToSettings))}
        }
        {separator()}
        {super::connection::view(vm,ui)}
    }}
}
