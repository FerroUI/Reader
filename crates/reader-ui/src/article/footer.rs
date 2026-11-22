//! Article actions and navigable tags share the same wrapping native control row.
use crate::{
    rows::Row,
    session::{Panel, Session},
};
use ferroui_base::layout::{HorizontalAlignment as H, Orientation, VerticalAlignment as V};
use ferroui_base::media::TextWrapping;
use ferroui_view::prelude::*;
use reader_controls::{icon, Icon};
use reader_domain::View;
use reader_state::AppViewModel;
use std::rc::Rc;

pub(super) fn view(vm: Rc<AppViewModel>, ui: Rc<Session>, row: &Row) -> El<Border> {
    let id = row.article.id;
    let star = vm.clone();
    let read = vm.clone();
    let was_read = row.article.read;
    let share = vm.clone();
    let email = vm.clone();
    let tags = ui.clone();
    let note = ui;
    let print = vm.clone();
    let touch = row.touch;
    let height = if touch { 44.0 } else { 26.0 };
    let mut actions = wrap_panel()
        .orientation(Orientation::Horizontal)
        .child(
            action_control(touch, if row.article.starred { "Unstar" } else { "Add star" }, move || star.toggle_star(id))
                .child(rsx! {StackPanel{orientation:Orientation::Horizontal,spacing:4.0,vertical_alignment:V::Center,
                    {icon(if row.article.starred{Icon::Starred}else{Icon::Star},12.0,if row.article.starred{"#E9BC36"}else{"#777777"})}
                    TextBlock{text:if row.article.starred{"Unstar"}else{"Add star"},font_size:13.0,foreground:"#444444",vertical_alignment:V::Center}
                }}),
        )
        .child(action(touch, if was_read { "Keep unread" } else { "Mark as read" }, move || {
            if was_read { read.keep_unread(id); } else { read.set_read(id, true); }
        }))
        .child(action(touch, "Email", move || email.email_article(id)))
        .child(action(touch, "Send to", move || share.share_article(id)))
        .child(
            action(touch, if row.article.tags.is_empty() { "Add tags" } else { "Edit tags:" }, move || tags.show(Panel::Tags(id)))
                .name("ArticleTags")
                .tag(Some(Rc::new(id) as ferroui_base::BoxedValue))
                .automation_properties_name(if row.article.tags.is_empty() { "Add tags" } else { "Edit tags" }),
        );

    for (index, tag) in row.article.tags.iter().enumerate() {
        let navigate = vm.clone();
        let target = tag.clone();
        let label = if index + 1 == row.article.tags.len() {
            tag.clone()
        } else {
            format!("{tag},")
        };
        actions = actions.child(
            button()
                .key(format!("article-tag:{id}:{tag}"))
                .name("ArticleTag")
                .class("reader-link")
                .automation_properties_name(format!("Show items tagged {tag}"))
                .tool_tip_tip(tag.clone())
                .min_height(height)
                .min_width(if touch { 44.0 } else { 0.0 })
                .max_width(if row.narrow {
                    (vm.viewport() - 72.0).max(1.0)
                } else {
                    650.0
                })
                .padding((if touch { 8.0 } else { 2.0 }, 0.0))
                .horizontal_content_alignment(H::Left)
                .on_click(move |event| {
                    event.set_handled(true);
                    navigate.navigate(View::Tag(target.clone()));
                })
                .child(
                    text_block()
                        .text(label)
                        .font_size(13.0)
                        .foreground(if row.selected { "#444444" } else { "#666666" })
                        .text_wrapping(TextWrapping::Wrap),
                ),
        );
    }
    actions = actions
        .child(action(touch, "Add note", move || {
            note.show(Panel::Note(id))
        }))
        .child(action(touch, "Print", move || print.print_article(id)));

    border()
        .background("#FAFAFA")
        .border_brush("#EBEBEB")
        .border_thickness((0.0, 1.0, 0.0, 0.0))
        .padding((if row.narrow { 4.0 } else { 12.0 }, 2.0))
        .child(actions)
}

fn action(touch: bool, label: &str, invoke: impl Fn() + 'static) -> El<Button> {
    action_control(touch, label, invoke).content(label)
}

fn action_control(touch: bool, label: &str, invoke: impl Fn() + 'static) -> El<Button> {
    button()
        .class("reader-link")
        .font_size(13.0)
        .foreground("#444444")
        .padding((8.0, 0.0))
        .height(if touch { 44.0 } else { 26.0 })
        .min_width(if touch { 44.0 } else { 0.0 })
        .automation_properties_name(label)
        .on_click(move |event| {
            event.set_handled(true);
            invoke();
        })
}
