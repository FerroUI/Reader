//! Bounded, non-executing HTML projection for native Reader controls.
mod html;
mod images;
mod model;
mod state;
mod table;
use html::{attr, skip_element, tag_end};
use model::*;
pub use model::{Block, Cell, Image, Span, Table};
use state::{State, TableState};
pub use table::{CellPlacement, TableLayout};

pub fn parse(input: &str, base: &str) -> Vec<Block> {
    let mut end = input.len().min(MAX_INPUT);
    while !input.is_char_boundary(end) {
        end -= 1;
    }
    let html = &input[..end];
    let truncated = end < input.len();
    let mut state = State::default();
    let mut at = 0;
    while at < html.len() && !state.limited {
        let rest = &html[at..];
        if rest.starts_with("<!--") {
            at += rest.find("-->").map_or(rest.len(), |n| n + 3);
            continue;
        }
        if let Some(cdata) = rest.strip_prefix("<![CDATA[") {
            let end = cdata.find("]]>").unwrap_or(cdata.len());
            state.text(&cdata[..end]);
            at += 9 + end + usize::from(end < cdata.len()) * 3;
            continue;
        }
        if rest.starts_with('<')
            && rest
                .as_bytes()
                .get(1)
                .is_some_and(|c| c.is_ascii_alphabetic() || matches!(*c, b'/' | b'!' | b'?'))
        {
            let Some(end) = tag_end(rest) else {
                state.text(rest);
                break;
            };
            let raw = rest[1..end].trim();
            at += end + 1;
            let closing = raw.starts_with('/');
            let name = raw
                .trim_start_matches('/')
                .split(|c: char| c.is_ascii_whitespace() || c == '/')
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();
            if !closing
                && matches!(
                    name.as_str(),
                    "script"
                        | "style"
                        | "head"
                        | "template"
                        | "noscript"
                        | "form"
                        | "object"
                        | "svg"
                )
            {
                at += skip_element(&html[at..], &name);
                continue;
            }
            if !closing
                && (attr(raw, "hidden").is_some()
                    || attr(raw, "aria-hidden").as_deref() == Some("true"))
            {
                if !raw.ends_with('/')
                    && !matches!(
                        name.as_str(),
                        "img" | "br" | "hr" | "input" | "source" | "embed" | "meta" | "link"
                    )
                {
                    at += skip_element(&html[at..], &name);
                }
                continue;
            }
            match name.as_str() {
                "b" | "strong" => depth(&mut state.bold, closing),
                "i" | "em" => depth(&mut state.italic, closing),
                "code" | "tt" | "kbd" | "samp" => depth(&mut state.code, closing),
                "a" => {
                    if closing {
                        state.hrefs.pop();
                    } else if state.hrefs.len() < MAX_DEPTH {
                        state.hrefs.push(
                            attr(raw, "href").and_then(|v| reader_domain::safe_url(&v, Some(base))),
                        );
                    }
                }
                "br" => state.newline(),
                "img" if !closing => images::append(&mut state, raw, base),
                "iframe" | "video" | "audio" | "embed" if !closing => {
                    if let Some(url) =
                        attr(raw, "src").and_then(|v| reader_domain::safe_url(&v, Some(base)))
                    {
                        state.media(
                            if name == "audio" {
                                "Listen to audio"
                            } else {
                                "View embedded media"
                            },
                            url,
                        );
                    }
                    if name == "iframe" {
                        at += skip_element(&html[at..], &name);
                    }
                }
                "source" if !closing => {
                    if let Some(url) =
                        attr(raw, "src").and_then(|v| reader_domain::safe_url(&v, Some(base)))
                    {
                        state.media("Open media", url);
                    }
                }
                "ol" | "ul" => {
                    state.flush();
                    if closing {
                        state.lists.pop();
                    } else if state.lists.len() < MAX_DEPTH {
                        state.lists.push(if name == "ol" {
                            Some(attr(raw, "start").and_then(|v| v.parse().ok()).unwrap_or(1))
                        } else {
                            None
                        });
                    }
                }
                "li" => {
                    state.flush();
                    if !closing {
                        let marker = if let Some(Some(number)) = state.lists.last_mut() {
                            if let Some(value) = attr(raw, "value").and_then(|v| v.parse().ok()) {
                                *number = value;
                            }
                            let marker = format!("{number}.  ");
                            *number = number.saturating_add(1);
                            marker
                        } else {
                            "•  ".into()
                        };
                        state.block.list_depth = state.lists.len().clamp(1, 8);
                        state.block.spans.push(Span {
                            text: marker,
                            ..Default::default()
                        });
                    }
                }
                "blockquote" | "pre" => {
                    state.flush();
                    depth(
                        if name == "pre" {
                            &mut state.pre
                        } else {
                            &mut state.quote
                        },
                        closing,
                    );
                }
                "table" => {
                    state.flush();
                    if closing {
                        if state.table.as_ref().is_some_and(|t| t.nesting > 0) {
                            state.table.as_mut().unwrap().nesting -= 1;
                        } else {
                            state.finish_table();
                        }
                    } else if let Some(table) = &mut state.table {
                        table.nesting += 1;
                    } else {
                        state.table = Some(TableState::default());
                    }
                }
                "tr" => {
                    state.flush();
                    if let Some(table) = &mut state.table {
                        if table.nesting == 0 {
                            table.end_row();
                        }
                    }
                }
                "td" | "th" => {
                    state.flush();
                    if let Some(table) = &mut state.table {
                        if table.nesting == 0 {
                            if closing {
                                table.end_cell();
                            } else {
                                table.begin_cell(Cell {
                                    heading: name == "th",
                                    colspan: attr(raw, "colspan")
                                        .and_then(|v| v.parse().ok())
                                        .unwrap_or(1),
                                    rowspan: attr(raw, "rowspan")
                                        .and_then(|v| v.parse().ok())
                                        .unwrap_or(1),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    state.flush();
                    state.heading = if closing {
                        0
                    } else {
                        name.as_bytes()[1] - b'0'
                    };
                }
                "hr" => {
                    state.flush();
                    state.block.rule = true;
                    state.flush();
                }
                "p" | "div" | "section" | "article" | "header" | "footer" | "figure"
                | "figcaption" | "dl" | "dt" | "dd" | "address" => state.flush(),
                _ => {}
            }
        } else {
            let end = if rest.starts_with('<') {
                1
            } else {
                rest.find('<').unwrap_or(rest.len())
            };
            state.text(&rest[..end]);
            at += end;
        }
    }
    state.limited |= truncated;
    state.finish()
}

fn depth(depth: &mut usize, closing: bool) {
    *depth = if closing {
        depth.saturating_sub(1)
    } else {
        depth.saturating_add(1).min(MAX_DEPTH)
    };
}

#[cfg(test)]
mod tests;
