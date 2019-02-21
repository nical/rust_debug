use std::collections::HashMap;
use std::cell::RefCell;

pub struct Counters {
    events: RefCell<HashMap<&'static str, u64>>
}

impl Counters {
    pub fn new() -> Self {
        Counters {
            events: RefCell::new(HashMap::default()),
        }
    }

    /// Increment the counter for the provided event key.
    pub fn event(&self, key: &'static str) {
        *self.events.borrow_mut().entry(key).or_insert(0) += 1
    }

    /// Reset the counter for the provided event key to zero.
    pub fn reset(&self, key: &'static str) {
        self.events.borrow_mut().insert(key, 0);
    }

    /// Reset all counters to zero.
    pub fn reset_all(&self) {
        self.events.borrow_mut().clear();
    }

    /// Get the value of the counter or zero if it does not exist.
    pub fn get(&self, key: &'static str) -> u64 {
        self.events.borrow().get(key).cloned().unwrap_or(0)
    }

    /// Return the sum of all counters with keys containing the provided string.
    pub fn accumulate(&self, filter: &'static str) -> u64 {
        let mut n = 0;

        for (key, value) in self.events.borrow().iter() {
            if key.contains(filter) {
                n += value
            }
        }

        n
    }

    /// Print the counters to stdout.
    ///
    /// If a string is passed, only counters with keys containing that string will be printed.
    pub fn print(&self, filter: Option<&'static str>) {
        for (key, value) in self.events.borrow().iter() {
            if filter.map(|s| key.contains(s)).unwrap_or(true) {
                println!("{}: {}", key, value);
            }
        }
    }
}

#[test]
fn it_works() {
    let counters = Counters::new();

    counters.event("foo::bar");
    counters.event("foo::bar");

    counters.event("foo::baz");

    counters.event("meh");
    counters.event("fooo");

    assert_eq!(counters.get("foo::bar"), 2);
    assert_eq!(counters.get("foo::baz"), 1);
    assert_eq!(counters.accumulate("foo::"), 3);

    counters.reset_all();

    assert_eq!(counters.get("foo::bar"), 0);
    assert_eq!(counters.get("foo::baz"), 0);
    assert_eq!(counters.accumulate("foo::"), 0);    
}
