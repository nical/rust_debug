use crate::filters::Select;
use crate::Counters;
use std::cell::RefCell;
use std::io;

/// Helper to print counters as a table in csv format.
///
/// # Example
///
/// ```
/// use counters::*;
///
/// let counters = Counters::new();
/// let table = Table::new(&["foo", "bar", "meh"]);
///
/// for _ in 0..5 {
///     counters.event("bar");
/// }
/// counters.event("foo");
///
/// // "baz" isn't in the table labels, it will be ignored.
/// counters.event("baz");
///
/// table.add_row(&counters);
///
/// // Start a second row...
/// counters.reset_all();
///
/// counters.event("foo");
///
/// table.add_row(&counters);
///
/// // This prints the following to stdout:
/// // foo,bar,meh
/// // 1,5,0
/// // 1,0,0
/// table.print_to_stdout();
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Table {
    labels: Vec<String>,
    rows: RefCell<Vec<Counters>>,
}

impl Table {
    pub fn new<Label>(labels: &[Label]) -> Self
    where
        Label: ToString,
    {
        Table {
            labels: labels.iter().map(|label| label.to_string()).collect(),
            rows: RefCell::new(Vec::new()),
        }
    }

    /// Add collected counters as a row, preserving only the counters that match this table's labels.
    pub fn add_row(&self, row: &Counters) -> usize {
        let row = row.clone();
        row.retain(Select(|key, _| {
            self.labels.iter().any(|label| label == key)
        }));
        self.rows.borrow_mut().push(row);

        self.rows.borrow().len()
    }

    /// Print in csv format to an io stream.
    pub fn print(&self, to: &mut io::Write) -> io::Result<()> {
        for (i, label) in self.labels.iter().enumerate() {
            if i != 0 {
                write!(to, ",")?;
            }
            write!(to, "{}", label)?;
        }
        writeln!(to, "")?;
        for row in self.rows.borrow().iter() {
            for (i, label) in self.labels.iter().enumerate() {
                if i != 0 {
                    write!(to, ", ")?;
                }
                write!(to, "{}", row.get(label))?;
            }
            writeln!(to, "")?;
        }

        Ok(())
    }

    /// Print in csv format to stdout.
    pub fn print_to_stdout(&self) {
        self.print(&mut io::stdout()).unwrap();
    }
}
