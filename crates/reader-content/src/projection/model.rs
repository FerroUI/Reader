//! Safe native presentation data. No executable markup crosses this boundary.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Span {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
    pub href: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Image {
    pub url: String,
    pub alt: String,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub link: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cell {
    pub spans: Vec<Span>,
    pub heading: bool,
    pub colspan: usize,
    pub rowspan: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Table {
    pub rows: Vec<Vec<Cell>>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Block {
    pub spans: Vec<Span>,
    pub heading: u8,
    pub quote: bool,
    pub pre: bool,
    pub list_depth: usize,
    pub image: Option<Image>,
    pub table: Option<Table>,
    pub rule: bool,
    pub notice: bool,
}

/// Parsing limits bound pathological documents, while normal long articles keep
/// their complete body. A visible final notice reports any exceeded budget.
pub(super) const MAX_INPUT: usize = 2 * 1024 * 1024;
pub(super) const MAX_BLOCKS: usize = 2048;
pub(super) const MAX_SPANS: usize = 16384;
pub(super) const MAX_IMAGES: usize = 128;
pub(super) const MAX_TABLE_CELLS: usize = 4096;
pub(super) const MAX_DEPTH: usize = 64;
