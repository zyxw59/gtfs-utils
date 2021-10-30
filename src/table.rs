/// A table, arranged in column-major order.
#[derive(Debug)]
pub struct Table<C, R, T> {
    col_headers: Vec<C>,
    row_headers: Vec<R>,
    data: Vec<T>,
}

impl<C, R, T> Table<C, R, T> {
    pub fn new(row_headers: Vec<R>) -> Self {
        Table {
            col_headers: Vec::new(),
            row_headers,
            data: Vec::new(),
        }
    }

    /// Adds a new column, returning a reference to the new column.
    pub fn add_column(&mut self, header: C, default: T) -> &mut [T]
    where
        T: Clone,
    {
        self.col_headers.push(header);
        let start = self.data.len();
        let end = start + self.row_headers.len();
        self.data.resize(end, default);
        &mut self.data[start..end]
    }

    pub fn col_headers(&self) -> &[C] {
        &self.col_headers
    }

    pub fn row_headers(&self) -> &[R] {
        &self.row_headers
    }

    /// Produces an iterator of iterators, in row-major order.
    pub fn rows(&self) -> impl Iterator<Item = impl Iterator<Item = &T>> {
        RowsIter {
            data: &self.data,
            cursor: 0,
            rows: self.row_headers.len(),
        }
    }
}

struct RowsIter<'a, T> {
    data: &'a [T],
    cursor: usize,
    rows: usize,
}

impl<'a, T> Iterator for RowsIter<'a, T> {
    type Item = std::iter::StepBy<std::slice::Iter<'a, T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor == self.rows {
            None
        } else {
            let iter = self.data[self.cursor..].iter().step_by(self.rows);
            self.cursor += 1;
            Some(iter)
        }
    }
}
