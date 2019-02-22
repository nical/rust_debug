
#[macro_use]
#[cfg(feature = "serialization")]
extern crate serde;

#[cfg(not(feature = "noop"))]
use std::collections::HashMap;
#[cfg(not(feature = "noop"))]
use std::cell::RefCell;

#[derive(Clone, Debug)]
#[cfg(not(feature = "noop"))]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Counters {
    events: RefCell<HashMap<String, u64>>
}

#[cfg(not(feature = "noop"))]
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

    /// Reset the counter for the provided event key to zero.
    pub fn reset_event(&self, key: &str) {
        self.events.borrow_mut().insert(key.into(), 0);
    }

    /// Reset some of the counters to zero.
    pub fn reset_events<F: Filter>(&self, filter: F) {
        self.events.borrow_mut().retain(|key, _| !filter.apply(key));
    }

    /// Reset all counters to zero.
    pub fn reset_all(&self) {
        self.events.borrow_mut().clear();
    }

    /// Keep some of the counters and throw away the rest.
    pub fn retain<F: Filter>(&self, filter: F) {
        self.events.borrow_mut().retain(|key, _| filter.apply(key));
    }

    /// Get the value of the counter or zero if it does not exist.
    pub fn get(&self, key: &str) -> u64 {
        self.events.borrow().get(key).cloned().unwrap_or(0)
    }

    /// Return the sum of all counters with keys containing the provided string.
    pub fn accumulate<F: Filter>(&self, filter: F) -> u64 {
        let mut n = 0;

        for (key, value) in self.events.borrow().iter() {
            if filter.apply(key) {
                n += value
            }
        }

        n
    }

    /// Print the counters to stdout.
    ///
    /// If a string is passed, only counters with keys containing that string will be printed.
    pub fn print_to_stdout<F: Filter>(&self, filter: F) {
        for (key, value) in self.events.borrow().iter() {
            if filter.apply(key) {
                println!("{}: {}", key, value);
            }
        }
    }

    /// Moves all the elements of `other` into `self`, leaving `other` empty.
    pub fn append(&self, other: &Counters) {
        for (key, value) in other.events.borrow_mut().drain() {
            *self.events.borrow_mut().entry(key).or_insert(0) += value;
        }
    } 
}

pub trait Filter {
    fn apply(&self, key: &str) -> bool;
}

impl<'l> Filter for &'l str {
    fn apply(&self, key: &str) -> bool {
        key.contains(self)
    }
}

impl Filter for Fn(&str) -> bool {
    fn apply(&self, key: &str) -> bool {
        self(key)
    }
}

pub struct All;
impl Filter for All {
    fn apply(&self, _: &str) -> bool { true }
}

pub struct StartsWith<'l>(pub &'l str);
impl<'l> Filter for StartsWith<'l> {
    fn apply(&self, key: &str) -> bool {
        key.starts_with(self.0)
    }
}

pub struct EndsWith<'l>(pub &'l str);
impl<'l> Filter for EndsWith<'l> {
    fn apply(&self, key: &str) -> bool {
        key.ends_with(self.0)
    }
}

#[cfg(feature = "noop")]
pub struct Counters;

#[cfg(feature = "noop")]
impl Counters {
    pub fn new() -> Self { Counters }
    pub fn event(&self, _key: &str) {}
    pub fn reset_event(&self, _key: &str) {}
    pub fn reset_events<F: Filter>(&self, _filter: F) {}
    pub fn reset_all(&self) {}
    pub fn retain<F: Filter>(&self, _filter: F) {}
    pub fn get(&self, _key: &str) -> u64 { 0 }
    pub fn accumulate<F: Filter>(&self, _filter: F) -> u64 { 0 }
    pub fn print_to_stdout<F: Filter>(&self, _filter: F) {}
}

#[test]
#[cfg(not(feature = "noop"))]
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

    counters.reset_events(EndsWith("bar"));

    assert_eq!(counters.get("foo::bar"), 0);
    assert_eq!(counters.get("foo::baz"), 1);
    assert_eq!(counters.accumulate("foo::"), 1);

    counters.reset_all();

    assert_eq!(counters.get("foo::bar"), 0);
    assert_eq!(counters.get("foo::baz"), 0);
    assert_eq!(counters.accumulate("foo::"), 0);
}

#[test]
#[cfg(feature = "noop")]
fn noop() {
    let counters = Counters::new();

    counters.event("foo::bar");
    counters.event("foo::bar");

    counters.event("foo::baz");

    counters.event("meh");
    counters.event("fooo");

    assert_eq!(counters.get("foo::bar"), 0);
    assert_eq!(counters.get("foo::baz"), 0);
    assert_eq!(counters.accumulate("foo::"), 0);

    counters.reset_events(EndsWith("bar"));

    assert_eq!(counters.get("foo::bar"), 0);
    assert_eq!(counters.get("foo::baz"), 0);
    assert_eq!(counters.accumulate("foo::"), 0);

    counters.reset_all();

    assert_eq!(counters.get("foo::bar"), 0);
    assert_eq!(counters.get("foo::baz"), 0);
    assert_eq!(counters.accumulate("foo::"), 0);    
}
