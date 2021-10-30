use std::fmt;

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

    pub fn formatter<'a, Cf, Rf, Tf, Cs, Rs, Ts>(
        &'a self,
        col_fmt: Cf,
        row_fmt: Rf,
        data_fmt: Tf,
    ) -> impl fmt::Display + 'a
    where
        Cf: Fn(&'a C) -> Cs + 'a,
        Rf: Fn(&'a R) -> Rs + 'a,
        Tf: Fn(&'a T) -> Ts + 'a,
        Cs: fmt::Display + 'a,
        Rs: fmt::Display + 'a,
        Ts: fmt::Display + 'a,
    {
        TableFormatter {
            col_fmt,
            row_fmt,
            data_fmt,
            table: self,
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

struct TableFormatter<'a, C, R, T, Cf, Rf, Tf> {
    col_fmt: Cf,
    row_fmt: Rf,
    data_fmt: Tf,
    table: &'a Table<C, R, T>,
}

impl<'a, C, R, T, Cf, Rf, Tf, Cs, Rs, Ts> fmt::Display for TableFormatter<'a, C, R, T, Cf, Rf, Tf>
where
    Cf: Fn(&'a C) -> Cs,
    Rf: Fn(&'a R) -> Rs,
    Tf: Fn(&'a T) -> Ts,
    Cs: fmt::Display + 'a,
    Rs: fmt::Display + 'a,
    Ts: fmt::Display + 'a,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "—")?;
        for header in self.table.col_headers() {
            write!(f, "| {}", (self.col_fmt)(header))?;
        }
        writeln!(f)?;

        write!(f, "---")?;
        for _ in self.table.col_headers() {
            write!(f, "|---")?;
        }
        writeln!(f)?;

        for (header, row) in self.table.row_headers().iter().zip(self.table.rows()) {
            write!(f, "**{}**", (self.row_fmt)(header))?;
            for cell in row {
                write!(f, " | {}", (self.data_fmt)(cell))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
