use crate::rows::Row;
use ferroui_base::layout::{HorizontalAlignment as H, Orientation, VerticalAlignment as V};
use ferroui_base::media::{FontWeight, TextWrapping};
use ferroui_view::prelude::*;
use reader_controls::{icon, Icon};
use reader_state::AppViewModel;
use std::rc::Rc;

pub(super) fn view(vm: Rc<AppViewModel>, row: &Row, padding: f64) -> Element {
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
                TextBlock{text:"from ",font_size:13.0,foreground:"#666666",vertical_alignment:V::Center}
                {reader_controls::link(&row.source,move||source.navigate(reader_domain::View::Feed(feed_id)))}
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
