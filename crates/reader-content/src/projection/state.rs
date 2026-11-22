use super::model::*;

#[derive(Default)]
pub(super) struct State {
    pub output: Vec<Block>,
    pub block: Block,
    pub bold: usize,
    pub italic: usize,
    pub code: usize,
    pub pre: usize,
    pub quote: usize,
    pub heading: u8,
    pub hrefs: Vec<Option<String>>,
    pub lists: Vec<Option<i64>>,
    pub table: Option<TableState>,
    pub span_count: usize,
    pub images: usize,
    pub limited: bool,
    pending_space: bool,
}

impl State {
    pub fn flush(&mut self) {
        self.pending_space = false;
        if self.block.spans.is_empty() && self.block.image.is_none() && !self.block.rule {
            return;
        }
        let block = std::mem::take(&mut self.block);
        if let Some(table) = &mut self.table {
            table.append(block);
        } else if self.output.len() < MAX_BLOCKS {
            self.output.push(block);
        } else {
            self.limited = true;
        }
    }

    pub fn push_span(&mut self, span: Span) {
        if span.text.is_empty() {
            return;
        }
        if let Some(previous) = self.block.spans.last_mut().filter(|p| {
            p.bold == span.bold
                && p.italic == span.italic
                && p.code == span.code
                && p.href == span.href
        }) {
            previous.text.push_str(&span.text);
            return;
        }
        if self.span_count >= MAX_SPANS {
            self.limited = true;
            return;
        }
        self.span_count += 1;
        self.block.spans.push(span);
    }

    pub fn text(&mut self, text: &str) {
        self.block.heading = self.heading;
        self.block.pre = self.pre > 0;
        self.block.quote = self.quote > 0;
        self.block.list_depth = self.lists.len().min(8);
        let decoded = super::html::decode(text);
        let mut text = String::with_capacity(decoded.len());
        if self.pre > 0 {
            text = decoded;
        } else {
            let mut pending = self.pending_space;
            let mut populated = !self.block.spans.is_empty();
            for character in decoded.chars() {
                if character.is_ascii_whitespace() {
                    pending = true;
                } else {
                    if pending && populated {
                        text.push(' ');
                    }
                    text.push(character);
                    populated = true;
                    pending = false;
                }
            }
            self.pending_space = pending;
        }
        self.push_span(Span {
            text,
            bold: self.bold > 0,
            italic: self.italic > 0,
            code: self.code > 0,
            href: self.hrefs.last().cloned().flatten(),
        });
    }

    pub fn newline(&mut self) {
        self.pending_space = false;
        self.push_span(Span {
            text: "\n".into(),
            ..Default::default()
        });
    }

    pub fn media(&mut self, label: &str, url: String) {
        self.flush();
        self.block.spans.push(Span {
            text: label.into(),
            href: Some(url),
            ..Default::default()
        });
        self.flush();
    }

    pub fn finish_table(&mut self) {
        self.flush();
        if let Some(mut table) = self.table.take() {
            table.end_row();
            self.limited |= table.limited;
            if !table.table.rows.is_empty() && self.output.len() < MAX_BLOCKS {
                self.output.push(Block {
                    table: Some(table.table),
                    ..Default::default()
                });
            }
        }
    }

    pub fn finish(mut self) -> Vec<Block> {
        self.finish_table();
        self.flush();
        if self.limited {
            self.output.push(Block {
                notice: true,
                spans: vec![Span {
                    text: "This article exceeds the reading view's content limit. View the original to read the remainder.".into(),
                    ..Default::default()
                }],
                ..Default::default()
            });
        }
        self.output
    }
}

#[derive(Default)]
pub(super) struct TableState {
    table: Table,
    row: Vec<Cell>,
    cell: Option<Cell>,
    cells: usize,
    pub limited: bool,
    pub nesting: usize,
}

impl TableState {
    pub fn append(&mut self, block: Block) {
        if self.cell.is_none() {
            self.begin_cell(Cell::default());
        }
        let Some(cell) = &mut self.cell else {
            return;
        };
        if !cell.spans.is_empty() && !block.spans.is_empty() {
            cell.spans.push(Span {
                text: "\n".into(),
                ..Default::default()
            });
        }
        cell.spans.extend(block.spans);
        if let Some(image) = block.image {
            cell.spans.push(Span {
                text: if image.alt.is_empty() {
                    "View image".into()
                } else {
                    image.alt
                },
                href: Some(image.url),
                ..Default::default()
            });
        }
    }

    pub fn begin_cell(&mut self, mut cell: Cell) {
        self.end_cell();
        if self.cells >= MAX_TABLE_CELLS {
            self.limited = true;
            return;
        }
        self.cells += 1;
        cell.colspan = cell.colspan.clamp(1, 32);
        cell.rowspan = cell.rowspan.clamp(1, 512);
        self.cell = Some(cell);
    }

    pub fn end_cell(&mut self) {
        if let Some(cell) = self.cell.take() {
            self.row.push(cell);
        }
    }

    pub fn end_row(&mut self) {
        self.end_cell();
        if !self.row.is_empty() {
            self.table.rows.push(std::mem::take(&mut self.row));
        }
    }
}
