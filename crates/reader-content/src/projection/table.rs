use super::{Cell, Table};

const MAX_COLUMNS: usize = 32;

pub struct CellPlacement<'a> {
    pub row: usize,
    pub column: usize,
    pub colspan: usize,
    pub rowspan: usize,
    pub cell: &'a Cell,
}

/// Bounded Grid geometry. Unusually wide rows retain their remaining cells as
/// a readable fallback instead of silently losing data or creating huge grids.
pub struct TableLayout<'a> {
    pub cells: Vec<CellPlacement<'a>>,
    pub overflow: Vec<(usize, &'a Cell)>,
    pub columns: usize,
    pub rows: usize,
}

impl Table {
    pub fn layout(&self) -> TableLayout<'_> {
        let mut layout = TableLayout {
            cells: Vec::new(),
            overflow: Vec::new(),
            columns: 1,
            rows: self.rows.len(),
        };
        let mut occupied = [0; MAX_COLUMNS];
        for (row, items) in self.rows.iter().enumerate() {
            let mut column = 0;
            for cell in items {
                let colspan = cell.colspan.clamp(1, MAX_COLUMNS);
                while column + colspan <= MAX_COLUMNS
                    && occupied[column..column + colspan]
                        .iter()
                        .any(|until| *until > row)
                {
                    column += 1;
                }
                if column + colspan > MAX_COLUMNS {
                    layout.overflow.push((row, cell));
                    continue;
                }
                let rowspan = cell.rowspan.clamp(1, 512);
                occupied[column..column + colspan].fill(row + rowspan);
                layout.cells.push(CellPlacement {
                    row,
                    column,
                    colspan,
                    rowspan,
                    cell,
                });
                column += colspan;
                layout.columns = layout.columns.max(column);
                layout.rows = layout.rows.max(row + rowspan);
            }
        }
        layout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colspan_avoids_the_entire_occupied_rowspan_range() {
        let table = Table {
            rows: vec![
                vec![
                    Cell::default(),
                    Cell {
                        rowspan: 2,
                        ..Cell::default()
                    },
                ],
                vec![Cell {
                    colspan: 2,
                    ..Cell::default()
                }],
            ],
        };
        let layout = table.layout();
        assert_eq!(layout.cells[2].column, 2);
        assert_eq!(layout.columns, 4);
    }

    #[test]
    fn extra_wide_tables_preserve_every_cell_in_the_readable_fallback() {
        let table = Table {
            rows: vec![vec![Cell::default(); 40]],
        };
        let layout = table.layout();
        assert_eq!(layout.cells.len(), MAX_COLUMNS);
        assert_eq!(layout.overflow.len(), 8);
        assert_eq!(layout.columns, MAX_COLUMNS);
    }
}
