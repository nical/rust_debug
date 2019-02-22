use Counters;
use std::io;
use std::cell::RefCell;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Table {
    labels: Vec<String>,
    rows: RefCell<Vec<Counters>>,
}

impl Table {
    pub fn new<Iter, Label>(labels: Iter) -> Self
    where Iter: IntoIterator<Item=Label>, Label: Into<String> {
        Table {
            labels: labels.into_iter().map(|label| label.into()).collect(),
            rows: RefCell::new(Vec::new()),
        }
    }

    pub fn add_row(&self, row: Counters) -> usize {
        row.retain(Select(|key: &str| self.labels.iter().any(|label| label == key) ));
        self.rows.borrow_mut().push(row);

        self.rows.borrow().len()
    }

    pub fn print(&self, to: &mut io::Write) -> io::Result<()> {
        for (i, label) in self.labels.iter().enumerate() {
            if i != 0 {
                write!(to, ", ")?;
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

    pub fn print_to_stdout(&self) {
        self.print(&mut io::stdout()).unwrap();
    }
}
