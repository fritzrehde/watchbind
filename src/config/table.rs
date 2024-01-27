use std::fmt;
use tabled::{
    builder::Builder,
    settings::{
        peaker::PriorityMax, themes::ColumnNames, Margin, Padding, Style, Width as TabledWidth,
    },
    Table as TabledTable,
};

/// A formatted table. Columns will never overflow if an appropriate maximum
/// screen/widget width was specified.
pub struct Table<'a, RowsIter, RowIter, TableItem>
where
    TableItem: Into<String>,
    RowIter: IntoIterator<Item = TableItem>,
    RowsIter: IntoIterator<Item = RowIter>,
{
    data: RowsIter,
    width: Option<usize>,
    left_margin: Option<usize>,
    border: bool,
    header: Option<&'a [String]>,
}

impl<'a, RowsIter, RowIter, TableItem> Table<'a, RowsIter, RowIter, TableItem>
where
    TableItem: Into<String>,
    RowIter: IntoIterator<Item = TableItem>,
    RowsIter: IntoIterator<Item = RowIter>,
{
    pub fn new(data: RowsIter) -> Self {
        Table {
            data,
            width: None,
            left_margin: None,
            border: false,
            header: None,
        }
    }

    /// Add a left margin.
    pub fn left_margin(mut self, left_margin: usize) -> Self {
        self.left_margin = Some(left_margin);
        self
    }

    /// Set the maximum width for string/display output.
    pub fn width<U>(mut self, width: Option<U>) -> Self
    where
        usize: From<U>,
    {
        self.width = width.map_or(self.width, |w| Some(usize::from(w)));
        self
    }

    /// Add a border.
    pub fn border(mut self) -> Self {
        self.border = true;
        self
    }

    /// Use the first input row as a header.
    pub fn header(mut self, column_names: &'a [String]) -> Self {
        self.header = Some(column_names);
        self
    }

    /// Return the table as something that can be displayed (e.g. be printed
    /// to stdout).
    pub fn displayable(self) -> impl fmt::Display {
        self.create_table()
    }

    /// Return the table as a string.
    pub fn make_string(self) -> String {
        self.create_table().to_string()
    }

    fn create_table(self) -> TabledTable {
        let left_margin = self.left_margin.unwrap_or(0);
        let mut table = Builder::from_iter(self.data.into_iter().map(RowIter::into_iter)).build();

        if self.border {
            table.with(Style::modern());
        } else {
            table
                .with(Style::blank())
                // Remove left padding.
                .with(Padding::new(0, 1, 0, 0));
        }

        // Add left margin for indent.
        table.with(Margin::new(left_margin, 0, 0, 0));

        if let Some(column_names) = self.header {
            // Set the header to the column names.
            table.with(ColumnNames::new(column_names));
        }

        if let Some(width) = self.width {
            // Set table width.
            table
                .with(
                    TabledWidth::wrap(width)
                        .priority::<PriorityMax>()
                        .keep_words(),
                )
                .with(TabledWidth::increase(width));
        }

        table
    }
}
