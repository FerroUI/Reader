//! Keep immutable projection data across selection, annotation and width changes.
use reader_content::projection::{self, Block};
use std::{
    cell::RefCell,
    collections::VecDeque,
    hash::{Hash, Hasher},
    rc::Rc,
};

const BYTES: usize = 8 * 1024 * 1024;
const DOCUMENTS: usize = 96;
struct Entry {
    fingerprint: u64,
    html: Rc<str>,
    base: Rc<str>,
    blocks: Rc<[Block]>,
    bytes: usize,
}
#[derive(Default)]
struct Cache {
    entries: VecDeque<Entry>,
    bytes: usize,
}
ferroui_base::ferro_thread_local! {
    static CACHE: RefCell<Cache> = RefCell::new(Cache::default());
}

pub(super) fn project(html: &str, base: &str) -> (u64, Rc<[Block]>) {
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    html.hash(&mut hash);
    base.hash(&mut hash);
    let fingerprint = hash.finish();
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(at) = cache.entries.iter().position(|entry| {
            entry.fingerprint == fingerprint
                && entry.html.as_ref() == html
                && entry.base.as_ref() == base
        }) {
            let entry = cache.entries.remove(at).unwrap();
            let blocks = entry.blocks.clone();
            cache.entries.push_back(entry);
            return (fingerprint, blocks);
        }
        let blocks: Rc<[Block]> = projection::parse(html, base).into();
        let bytes = html.len() + base.len() + blocks.iter().map(charge).sum::<usize>();
        if bytes <= BYTES {
            while cache.bytes + bytes > BYTES || cache.entries.len() >= DOCUMENTS {
                let Some(old) = cache.entries.pop_front() else {
                    break;
                };
                cache.bytes -= old.bytes;
            }
            cache.bytes += bytes;
            cache.entries.push_back(Entry {
                fingerprint,
                html: html.into(),
                base: base.into(),
                blocks: blocks.clone(),
                bytes,
            });
        }
        (fingerprint, blocks)
    })
}

fn charge(block: &Block) -> usize {
    let mut bytes =
        std::mem::size_of::<Block>() + block.spans.iter().map(span_charge).sum::<usize>();
    if let Some(image) = &block.image {
        bytes += image.url.len() + image.alt.len() + image.link.as_ref().map_or(0, String::len);
    }
    if let Some(table) = &block.table {
        bytes += table
            .rows
            .iter()
            .flat_map(|row| row.iter())
            .map(|cell| {
                std::mem::size_of_val(cell) + cell.spans.iter().map(span_charge).sum::<usize>()
            })
            .sum::<usize>();
    }
    bytes
}
fn span_charge(span: &projection::Span) -> usize {
    std::mem::size_of_val(span) + span.text.len() + span.href.as_ref().map_or(0, String::len)
}
