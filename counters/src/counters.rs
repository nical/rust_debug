use std::io;
use std::collections::HashMap;
use std::cell::RefCell;

use crate::filter::Filter;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Counters {
    events: RefCell<HashMap<String, u64>>
}

impl Counters {
    pub fn new() -> Self {
        Counters {
            events: RefCell::new(HashMap::default()),
        }
    }

    /// Increment the counter for the provided event key.
    pub fn event(&self, key: &str) {
        *self.events.borrow_mut().entry(key.into()).or_insert(0) += 1
    }

    /// Set the value of the counter for a given event key.
    pub fn set(&self, key: &str, value: u64) {
        self.events.borrow_mut().insert(key.into(), value);
    }

    /// Reset the counter for the provided event key to zero.
    pub fn reset_event(&self, key: &str) {
        self.events.borrow_mut().insert(key.into(), 0);
    }

    /// Reset some of the counters to zero.
    pub fn reset_events<F: Filter>(&self, mut filter: F) {
        self.events.borrow_mut().retain(|key, val| !filter.apply(key, *val));
    }

    /// Reset all counters to zero.
    pub fn reset_all(&self) {
        self.events.borrow_mut().clear();
    }

    /// Keep some of the counters and throw away the rest.
    pub fn retain<F: Filter>(&self, mut filter: F) {
        self.events.borrow_mut().retain(|key, val| filter.apply(key, *val));
    }

    /// Get the value of the counter or zero if it does not exist.
    pub fn get(&self, key: &str) -> u64 {
        self.events.borrow().get(key).cloned().unwrap_or(0)
    }

    /// Return the sum of all counters with keys containing the provided filter.
    pub fn accumulate<F: Filter>(&self, mut filter: F) -> u64 {
        let mut n = 0;

        for (key, value) in self.events.borrow().iter() {
            if filter.apply(key, *value) {
                n += value
            }
        }

        n
    }

    /// Print the counters to an io stream.
    pub fn print<F: Filter>(&self, mut filter: F, to: &mut io::Write) -> io::Result<()> {
        for (key, value) in self.events.borrow().iter() {
            if filter.apply(key, *value) {
                writeln!(to, "{}: {}", key, value)?;
            }
        }

        Ok(())
    }


    /// Print the counters to stdout.
    pub fn print_to_stdout<F: Filter>(&self, filter: F) {
        let stdout = io::stdout();
        let mut to = stdout.lock();
        self.print(filter, &mut to).unwrap();
    }

    /// Moves all the elements of `other` into `self`, leaving `other` empty.
    pub fn append(&self, other: &Counters) {
        for (key, value) in other.events.borrow_mut().drain() {
            *self.events.borrow_mut().entry(key).or_insert(0) += value;
        }
    } 
}
