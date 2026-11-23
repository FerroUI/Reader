use ferroui_base::media::FontWeight;
use ferroui_view::prelude::*;
use reader_controls::{action, action_row, paragraph, separator};
use reader_state::AppViewModel;
use std::rc::Rc;

pub fn view(vm: Rc<AppViewModel>) -> Element {
    let import = vm.clone();
    let export = vm.clone();
    let backup = vm.clone();
    let restore = vm.clone();
    rsx! {StackPanel {spacing:18.0,
        TextBlock {text:"Your subscriptions",font_weight:FontWeight::Bold,font_size:14.0}
        {paragraph("Import subscriptions from an OPML file, including an export from Google Reader or another feed reader. Folder names are preserved.")}
        {action_row([action("Import subscriptions...",move||import.request_import_opml()),action("Export subscriptions",move||export.request_export_opml())],10.0)}
        {separator()}
        TextBlock {text:"Your complete library",font_weight:FontWeight::Bold,font_size:14.0}
        {paragraph("A complete backup includes cached articles, read state, stars, tags, notes, subscriptions and preferences.")}
        {action_row([action("Download backup",move||backup.request_export_json()),action("Restore backup...",move||restore.request_import_json())],10.0)}
    }}
}
