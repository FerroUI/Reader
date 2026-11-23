mod editor_focus;
mod folder_confirmation;
mod folders;
mod form;
mod navigate;
mod placement;
mod shortcuts;
mod subscribe;
mod tags;
use crate::{
    session::{Panel, Session},
    settings,
};
use ferroui_base::input::{Key, KeyboardNavigationMode};
use ferroui_base::layout::{HorizontalAlignment as H, VerticalAlignment as V};
use ferroui_base::media::{BoxShadows, TextTrimming};
use ferroui_controls::primitives::ScrollBarVisibility;
use ferroui_view::prelude::*;
use reader_controls::{action, action_row, paragraph, primary, Icon};
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn view(vm: Rc<AppViewModel>, ui: Rc<Session>) -> Element {
    let panel = ui.panel.get();
    if panel == Panel::None {
        return Element::Empty;
    }
    if panel == Panel::Shortcuts {
        return shortcuts::overlay(ui, &vm);
    }
    if let Panel::SendToMenu(id) = panel {
        return crate::send_to::popup(vm, ui, id);
    }
    let popup = desktop_popup(&vm, &ui);
    if panel == Panel::Subscribe && popup {
        return subscribe::popup(vm, ui);
    }
    if popup {
        if let Panel::Tags(id) = panel {
            if let Some(anchor) =
                tags::anchor(&ui, &vm, id).or_else(|| tags::existing_anchor(&ui, id))
            {
                return tags::popup(vm, ui, id, anchor);
            }
        }
    }
    let focus_dialog = matches!(
        panel,
        Panel::Preferences
            | Panel::SendToSettings
            | Panel::Subscriptions
            | Panel::ImportExport
            | Panel::About
            | Panel::MarkAll(_)
            | Panel::Unsubscribe(_)
            | Panel::DeleteFolder(_)
            | Panel::UnsubscribeFolder(_)
            | Panel::Move(_)
    );
    let dialog_key = match &panel {
        Panel::DeleteFolder(target) => format!("dialog:delete-folder:{}", target.name),
        Panel::UnsubscribeFolder(target) => format!("dialog:unsubscribe-folder:{}", target.name),
        _ => format!("dialog:{panel:?}"),
    };
    let close = ui.clone();
    let close_key = ui.clone();
    let title = match panel {
        Panel::Subscribe => "Add a subscription",
        Panel::Shortcuts => "Keyboard shortcuts",
        Panel::Preferences => "Reader settings",
        Panel::Subscriptions => "Subscriptions",
        Panel::GoToSubscription => "Go to a subscription",
        Panel::GoToTag => "Go to a tag or folder",
        Panel::ImportExport => "Import/Export",
        Panel::SendToSettings => "Send to",
        Panel::EditSendTo(_) => "Custom Send to link",
        Panel::SendToMenu(_) => "Send to",
        Panel::Tags(_) => "Edit tags",
        Panel::Note(_) => "Add a note",
        Panel::Rename(_) => "Rename subscription",
        Panel::RenameFolder(_) => "Rename folder",
        Panel::DeleteFolder(_) => "Delete folder?",
        Panel::UnsubscribeFolder(_) => "Unsubscribe from all?",
        Panel::Move(_) => "Change folders",
        Panel::Unsubscribe(_) => "Unsubscribe",
        Panel::NewFolder => "Create a new folder",
        Panel::MarkAll(_) => "Mark all as read?",
        Panel::About => "About Reader",
        Panel::None => "",
    };
    let width: f64 = if matches!(
        panel,
        Panel::Shortcuts | Panel::Subscriptions | Panel::Preferences | Panel::SendToSettings
    ) {
        760.0
    } else {
        510.0
    };
    let metrics = crate::responsive::DialogMetrics::new(&vm, width, 650.0);
    let content = match panel.clone() {
        Panel::Shortcuts => unreachable!(),
        Panel::Subscriptions => settings::subscriptions(vm.clone(), ui.clone()),
        Panel::Preferences => settings::preferences(vm.clone(), ui.clone()),
        Panel::ImportExport => settings::import_export(vm.clone()),
        Panel::SendToSettings => settings::send_to(vm.clone(), ui.clone()),
        Panel::EditSendTo(id) => settings::edit_send_to(vm.clone(), ui.clone(), id),
        Panel::About => about(),
        Panel::GoToSubscription => navigate::view(vm.clone(), ui.clone(), false),
        Panel::GoToTag => navigate::view(vm.clone(), ui.clone(), true),
        Panel::Move(id) => folders::view(vm.clone(), ui.clone(), id),
        Panel::DeleteFolder(target) => {
            folder_confirmation::view(vm.clone(), ui.clone(), target, false)
        }
        Panel::UnsubscribeFolder(target) => {
            folder_confirmation::view(vm.clone(), ui.clone(), target, true)
        }
        Panel::MarkAll(_) | Panel::Unsubscribe(_) => {
            confirmation(vm.clone(), ui.clone(), panel.clone())
        }
        _ => form::view(vm.clone(), ui.clone(), panel.clone()),
    };
    rsx! {
        Grid {key:dialog_key,name:"DialogOverlay",background:"#80FFFFFF",keyboard_navigation_tab_navigation:KeyboardNavigationMode::Cycle,
            on_key_down:move |e|{if e.key==Key::Escape{close_key.close();e.set_handled(true);}},
            Border {name:"ReaderDialog",focusable:true,is_tab_stop:false,on_loaded:move |event|{if focus_dialog{if let Some(control)=event.source().and_then(|source|source.cast::<ferroui_controls::Control>()){control.focus();}}},background:"White",border_brush:"#ABABAB",border_thickness:1.0,box_shadow:BoxShadows::parse("0 4 16 #33000000").unwrap_or_default(),width:metrics.width,max_height:metrics.height,horizontal_alignment:H::Center,vertical_alignment:V::Center,
                Grid {
                    Border {padding:metrics.padding,
                        Grid {row_definitions:"Auto,*",row_spacing:metrics.gap,
                            TextBlock {text:title,font_size:16.0,line_height:24.0,min_height:24.0,margin:(0.0,0.0,18.0,0.0),foreground:"#000000",text_trimming:<dyn TextTrimming>::character_ellipsis(),vertical_alignment:V::Center}
                            ScrollViewer {name:"DialogScroll",Grid.row:1,max_height:metrics.body_height.min(520.0),horizontal_scroll_bar_visibility:ScrollBarVisibility::Disabled,vertical_scroll_bar_visibility:ScrollBarVisibility::Auto,
                                {content}
                            }
                        }
                    }
                    Button {class:"reader-flat",width:45.0,height:45.0,horizontal_alignment:H::Right,vertical_alignment:V::Top,automation_properties_name:"Close dialog",on_click:move |_|close.close(),{reader_controls::icon(Icon::Close,11.0,"#777777")}}
                }
            }
        }
    }
}

/// Keep the active native editor mounted while the input method owns a preedit.
/// A completed edit can switch presentation on the next responsive update.
fn desktop_popup(vm: &AppViewModel, ui: &Session) -> bool {
    let Some(field) = ui
        .field
        .get_untracked()
        .filter(|field| field.is_attached_to_visual_tree() && field.is_composing_text())
    else {
        return !crate::responsive::touch(vm);
    };
    field
        .find_ancestor_of_type_where::<ferroui_controls::Control>(false, |control| {
            matches!(
                control.name().as_deref(),
                Some("SubscribePopup" | "ArticleTagsPopup")
            )
        })
        .is_some()
}

fn confirmation(vm: Rc<AppViewModel>, ui: Rc<Session>, panel: Panel) -> Element {
    let confirm = ui.clone();
    let cancel = ui.clone();
    let skip = ui.skip_mark_all_confirmation;
    let generation = vm.library_generation();
    let route = vm.view();
    let text = match &panel {
        Panel::Unsubscribe(id) => format!("Are you sure you want to unsubscribe from {}? Starred, tagged and noted items will remain in your library; other items from this subscription will be removed. You can undo this change.", vm.feed(*id).map(|f| f.title).unwrap_or_default()),
        Panel::MarkAll(Some(cutoff)) => {
            let date = chrono::DateTime::from_timestamp(*cutoff, 0)
                .map(|date| date.format("%b %-d, %Y at %H:%M UTC").to_string())
                .unwrap_or_else(|| cutoff.to_string());
            format!("Mark all items in {} published before {date} as read?", vm.title())
        }
        _ => format!("Mark all items in {} as read?", vm.title()),
    };
    rsx! {StackPanel {spacing:16.0,
        {paragraph(text)}
        if matches!(panel,Panel::MarkAll(_)) {CheckBox {class:"reader-check",content:"Don't ask me again",automation_properties_name:"Don't ask me again",font_size:13.0,min_height:28.0,padding:0.0,is_checked:Some(skip.get()),on_click:move |_|skip.update(|value|*value=!*value)}}
        {action_row([
            primary(if matches!(panel,Panel::Unsubscribe(_)){"Unsubscribe"}else{"Mark all as read"},move||{
                if confirm.panel.get_untracked() != panel { return; }
                let suppress_confirmation = skip.get_untracked();
                confirm.close();
                if vm.library_generation() != generation || vm.view() != route { return; }
                match panel {
                    Panel::Unsubscribe(id) => {
                        vm.unsubscribe(id);
                        confirm.removed_subscriptions(&[id]);
                    }
                    Panel::MarkAll(cutoff) => {
                        if suppress_confirmation {
                            let mut settings=vm.state().settings.clone();
                            settings.confirm_mark_all=false;
                            vm.set_settings(settings);
                        }
                        vm.mark_all_read(cutoff);
                    }
                    _ => {}
                }
            }),
            action("Cancel",move||cancel.close())
        ],16.0)}
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
