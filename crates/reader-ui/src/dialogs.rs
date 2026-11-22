mod shortcuts;
use crate::{
    session::{Panel, Session},
    settings,
};
use ferroui_base::input::{Key, KeyboardNavigationMode};
use ferroui_base::layout::{HorizontalAlignment as H, Orientation, VerticalAlignment as V};
use ferroui_base::media::TextWrapping;
use ferroui_controls::primitives::ScrollBarVisibility;
use ferroui_view::prelude::*;
use reader_controls::{action, icon_action, paragraph, primary, Icon};
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn view(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let panel = ui.panel.get();
    if panel == Panel::None {
        return Element::Empty;
    }
    let close = ui.clone();
    let close_key = ui.clone();
    let title = match panel {
        Panel::Subscribe => "Add a subscription",
        Panel::Shortcuts => "Keyboard shortcuts",
        Panel::Preferences => "Reader settings",
        Panel::Subscriptions => "Subscriptions",
        Panel::ImportExport => "Import/Export",
        Panel::Tags(_) => "Edit tags",
        Panel::Note(_) => "Add a note",
        Panel::Rename(_) => "Rename subscription",
        Panel::Move(_) => "Change folders",
        Panel::Unsubscribe(_) => "Unsubscribe",
        Panel::NewFolder => "Create a new folder",
        Panel::MarkAll => "Mark all as read?",
        Panel::About => "About Reader",
        Panel::None => "",
    };
    let width: f64 = if matches!(
        panel,
        Panel::Shortcuts | Panel::Subscriptions | Panel::Preferences
    ) {
        760.0
    } else {
        510.0
    };
    let content = match panel.clone() {
        Panel::Shortcuts => shortcuts::view(),
        Panel::Subscriptions => settings::subscriptions(vm.clone(), ui.clone()),
        Panel::Preferences => settings::preferences(vm.clone(), ui.clone()),
        Panel::ImportExport => settings::import_export(vm.clone()),
        Panel::About => about(),
        Panel::MarkAll | Panel::Unsubscribe(_) => {
            confirmation(vm.clone(), ui.clone(), panel.clone())
        }
        _ => form(vm.clone(), ui.clone(), panel.clone()),
    };
    rsx! {
        Grid {name:"DialogOverlay",background:"#66000000",keyboard_navigation_tab_navigation:KeyboardNavigationMode::Cycle,
            on_key_down:move |e|{if e.key==Key::Escape{close_key.close();e.set_handled(true);}},
            Border {name:"ReaderDialog",background:"White",border_brush:"#888888",border_thickness:1.0,width:width.min((vm.viewport()-32.0).max(280.0)),max_height:650.0,padding:26.0,horizontal_alignment:H::Center,vertical_alignment:V::Center,
                Grid {row_definitions:"Auto,*",row_spacing:20.0,
                    Grid {column_definitions:"*,Auto",
                        TextBlock {text:title,font_size:20.0,foreground:"#222222",vertical_alignment:V::Center}
                        Border {Grid.column:1,{icon_action(Icon::Close,"Close dialog",move||close.close())}}
                    }
                    ScrollViewer {Grid.row:1,max_height:520.0,horizontal_scroll_bar_visibility:ScrollBarVisibility::Disabled,vertical_scroll_bar_visibility:ScrollBarVisibility::Auto,
                        {content}
                    }
                }
            }
        }
    }
}
fn form(vm: Rc<AppViewModel>, ui: Rc<Session>, panel: Panel) -> Element {
    if ui.form_for.borrow().as_ref() != Some(&panel) {
        let value = match &panel {
            Panel::Rename(id) => vm.feed(*id).map(|f| f.title).unwrap_or_default(),
            Panel::Move(id) => vm.feed(*id).map(|f| f.folder).unwrap_or_default(),
            Panel::Tags(id) => vm
                .article(*id)
                .map(|a| a.tags.join(", "))
                .unwrap_or_default(),
            Panel::Note(id) => vm.article(*id).map(|a| a.note).unwrap_or_default(),
            _ => String::new(),
        };
        *ui.form_for.borrow_mut() = Some(panel.clone());
        ui.entry.set(value);
    }
    let label = match panel {
        Panel::Subscribe => "Enter a feed or website URL:",
        Panel::Move(_) => "Folder name (leave blank for no folder):",
        Panel::Tags(_) => "Separate tags with commas:",
        Panel::Note(_) => "Your note:",
        _ => "Name:",
    };
    let action_text = if panel == Panel::Subscribe {
        "Add"
    } else {
        "Save"
    };
    let save_vm = vm.clone();
    let save_ui = ui.clone();
    let save_panel = panel.clone();
    let save: Rc<dyn Fn()> = Rc::new(move || {
        let value = save_ui.entry.get_untracked().trim().to_string();
        if value.is_empty()
            && !matches!(save_panel, Panel::Move(_) | Panel::Tags(_) | Panel::Note(_))
        {
            save_vm.set_status("Please enter a value.".into());
            return;
        }
        save_ui.close();
        match save_panel {
            Panel::Subscribe => save_vm.subscribe(value),
            Panel::Rename(id) => save_vm.rename_feed(id, value),
            Panel::Move(id) => save_vm.move_feed(id, value),
            Panel::Tags(id) => save_vm.set_tags(
                id,
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect(),
            ),
            Panel::Note(id) => save_vm.set_note(id, value),
            Panel::NewFolder => save_vm.create_folder(value),
            _ => {}
        }
    });
    let enter = save.clone();
    let cancel = ui.clone();
    let field = ui.field;
    let mut input = reader_controls::input(label)
        .name("DialogInput")
        .node_ref(field)
        .text(ui.entry.two_way())
        .horizontal_alignment(H::Stretch)
        .on_loaded(move |_| {
            if let Some(field) = field.get_untracked() {
                field.focus();
                field.select_all();
            }
        });
    let note = matches!(panel, Panel::Note(_));
    if note {
        input = input
            .height(150.0)
            .accepts_return(true)
            .text_wrapping(TextWrapping::Wrap);
    } else {
        input = input.on_key_down(move |e| {
            if e.key == Key::Return {
                e.set_handled(true);
                enter();
            }
        });
    }
    let mut form = stack_panel()
        .spacing(14.0)
        .child(paragraph(label))
        .child(input);
    if matches!(panel, Panel::Subscribe) {
        form=form.child(paragraph("Follow your favorite websites in one place. Paste an RSS or Atom feed address, or a website that advertises one."));
    }
    if matches!(panel, Panel::Move(_)) {
        let mut folders = wrap_panel().orientation(Orientation::Horizontal);
        for folder in vm.folders() {
            let ui = ui.clone();
            let value = folder.clone();
            folders = folders.child(reader_controls::link(&format!("{folder}  "), move || {
                ui.entry.set(value.clone())
            }));
        }
        form = form.child(folders);
    }
    form.child(
        stack_panel()
            .orientation(Orientation::Horizontal)
            .spacing(10.0)
            .margin((0.0, 12.0, 0.0, 0.0))
            .child(primary(action_text, move || save()))
            .child(action("Cancel", move || cancel.close())),
    )
    .into()
}
fn confirmation(vm: Rc<AppViewModel>, ui: Rc<Session>, panel: Panel) -> Element {
    let confirm = ui.clone();
    let cancel = ui.clone();
    let text = if let Panel::Unsubscribe(id) = panel {
        format!("Are you sure you want to unsubscribe from {}? Starred and tagged items will remain in your library.",vm.feed(id).map(|f|f.title).unwrap_or_default())
    } else {
        format!("Mark all items in {} as read?", vm.title())
    };
    rsx! {StackPanel {spacing:22.0,
        {paragraph(text)}
        StackPanel {orientation:Orientation::Horizontal,spacing:10.0,
            {primary(if matches!(panel,Panel::Unsubscribe(_)){"Unsubscribe"}else{"Mark all as read"},move||{confirm.close();if let Panel::Unsubscribe(id)=panel{vm.unsubscribe(id);}else{vm.mark_all_read(None);}})}
            {action("Cancel",move||cancel.close())}
        }
    }}
}
fn about() -> Element {
    rsx! {StackPanel {spacing:16.0,
        {paragraph("FerroUI Reader recreates the familiar Google Reader reading experience in Rust. Its interface is based on the final 2013 desktop design.")}
        {paragraph("Subscriptions, articles, tags, stars and reading history are saved in this browser. Use Import/Export to move your library or keep a backup.")}
        {paragraph("This independent project is not affiliated with Google. Google's original Reader and Google+ services were discontinued; this app uses public RSS and Atom feeds.")}
        {paragraph("Built with FerroUI, Material control templates, ReactiveUI and DynamicData.")}
    }}
}
