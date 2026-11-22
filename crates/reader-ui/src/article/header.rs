use crate::rows::Row;
use ferroui_base::layout::{HorizontalAlignment as H, Orientation, VerticalAlignment as V};
use ferroui_base::media::{FontWeight, TextWrapping};
use ferroui_view::prelude::*;
use reader_controls::{icon, Icon};
use reader_state::AppViewModel;
use std::rc::Rc;

pub(super) fn view(vm: Rc<AppViewModel>, row: &Row, padding: f64) -> Element {
    if row.touch {
        return touch_heading(vm, row, padding);
    }
    let id = row.article.id;
    let feed_id = row.article.feed_id;
    let open = vm.clone();
    let external = vm.clone();
    let star = vm.clone();
    let source = vm.clone();
    let title_width = if row.narrow {
        (vm.viewport() - 2.0 * padding - 60.0).max(130.0)
    } else {
        610.0
    };
    let title = row.article.title.clone();
    let mut heading = grid()
        .column_definitions("*,Auto")
        .margin((padding, padding, padding, 8.0));
    heading=heading.child(rsx!{
        StackPanel{max_width:650.0,horizontal_alignment:H::Left,spacing:4.0,
            WrapPanel{orientation:Orientation::Horizontal,
                Button{class:"reader-link",max_width:title_width,horizontal_content_alignment:H::Left,automation_properties_name:format!("Open {}",title),on_click:move |_|open.open_article(id),
                    TextBlock{text:title,font_size:18.0,font_weight:FontWeight::Bold,foreground:if row.article.read&&!row.selected{"#444444"}else{"#1155CC"},text_wrapping:TextWrapping::Wrap}
                }
                Button{class:"reader-flat",width:18.0,height:18.0,vertical_alignment:V::Bottom,margin:(2.0,0.0,0.0,2.0),automation_properties_name:"View original article",tool_tip_tip:"View original article (v)",on_click:move |_|external.open_article(id),
                    {icon(Icon::External,13.0,"#999999")}
                }
                Button{class:"reader-flat",width:18.0,height:18.0,vertical_alignment:V::Bottom,margin:(2.0,0.0,0.0,2.0),automation_properties_name:if row.article.starred{"Remove star"}else{"Add star"},on_click:move |event|{event.set_handled(true);star.toggle_star(id);},
                    {icon(if row.article.starred{Icon::Starred}else{Icon::Star},14.0,if row.article.starred{"#E9BC36"}else{"#B5B5B5"})}
                }
            }
            WrapPanel{orientation:Orientation::Horizontal,
                if !row.single_source {
                    TextBlock{text:"from ",font_size:13.0,foreground:"#666666",vertical_alignment:V::Center}
                    {reader_controls::link(&row.source,move||source.navigate(reader_domain::View::Feed(feed_id)))}
                }
                if !row.article.author.is_empty(){TextBlock{text:format!(" by {}",row.article.author),font_size:13.0,foreground:"#666666",vertical_alignment:V::Center}}
            }
        }
    });
    if !row.narrow {
        heading = heading.child(
            text_block()
                .grid_column(1)
                .text(super::full_date(row.article.published))
                .font_size(11.0)
                .foreground("#666666")
                .margin((12.0, 2.0, 0.0, 0.0))
                .vertical_alignment(V::Top),
        );
    }
    heading.into()
}

/// Phone/tablet actions retain the list's inline expansion and use real touch targets.
fn touch_heading(vm: Rc<AppViewModel>, row: &Row, padding: f64) -> Element {
    let id = row.article.id;
    let feed_id = row.article.feed_id;
    let collapse = vm.state().settings.view_mode == reader_domain::ViewMode::List;
    let title_action = vm.clone();
    let star = vm.clone();
    let original = vm.clone();
    let close = vm.clone();
    let feed = vm.clone();
    let title = row.article.title.clone();
    rsx! {StackPanel{margin:(padding,padding,padding,4.0),spacing:4.0,max_width:650.0,horizontal_alignment:H::Stretch,
        Grid{column_definitions:"*,44",column_spacing:6.0,
            Button{class:"reader-link",horizontal_content_alignment:H::Stretch,automation_properties_name:if collapse{format!("Collapse {}",title)}else{format!("Open {}",title)},on_click:move |_|{if collapse{title_action.select(None);}else{title_action.open_article(id);}},
                TextBlock{text:title,font_size:18.0,font_weight:FontWeight::Bold,foreground:if row.article.read&&!row.selected{"#444444"}else{"#1155CC"},text_wrapping:TextWrapping::Wrap}
            }
            Button{Grid.column:1,class:"reader-flat",width:44.0,height:44.0,vertical_alignment:V::Top,automation_properties_name:if row.article.starred{"Remove star"}else{"Add star"},on_click:move |event|{event.set_handled(true);star.toggle_star(id);},
                {icon(if row.article.starred{Icon::Starred}else{Icon::Star},18.0,if row.article.starred{"#E9BC36"}else{"#B5B5B5"})}
            }
        }
        if !row.single_source {
            Button{class:"reader-link",horizontal_content_alignment:H::Stretch,automation_properties_name:format!("From {}",row.source),on_click:move |_|feed.navigate(reader_domain::View::Feed(feed_id)),
                TextBlock{text:format!("from {}",row.source),font_size:13.0,foreground:"#1155CC",text_wrapping:TextWrapping::Wrap}
            }
        }
        if !row.article.author.is_empty(){TextBlock{text:format!("by {}",row.article.author),font_size:13.0,foreground:"#666666",text_wrapping:TextWrapping::Wrap}}
        TextBlock{text:super::full_date(row.article.published),font_size:12.0,foreground:"#777777",text_wrapping:TextWrapping::Wrap}
        WrapPanel{orientation:Orientation::Horizontal,
            Button{class:"reader-link",content:"View original",height:44.0,padding:(0.0,0.0,16.0,0.0),automation_properties_name:"View original article",on_click:move |_|original.open_article(id)}
            if collapse{Button{class:"reader-link",content:"Collapse item",height:44.0,automation_properties_name:"Collapse item",on_click:move |_|close.select(None)}}
        }
    }}
}
