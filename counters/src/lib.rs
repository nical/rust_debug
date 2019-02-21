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

    pub fn event(&self, name: &'static str) {
        *self.events.borrow_mut().entry(name).or_insert(0) += 1
    }
    pub fn reset(&self, name: &'static str) {
        self.events.borrow_mut().insert(name, 0);
    }
    pub fn reset_all(&self) {
        self.events.borrow_mut().clear();
    }

    pub fn get(&self, key: &'static str) -> u64 {
        self.events.borrow().get(key).cloned().unwrap_or(0)
    }

    pub fn accumulate(&self, substr: &'static str) -> u64 {
        let mut n = 0;

        for (key, value) in self.events.borrow().iter() {
            if key.contains(substr) {
                n += value
            }
        }

        n
    }

    pub fn print(&self, substr: Option<&'static str>) {
        for (key, value) in self.events.borrow().iter() {
            if substr.map(|s| key.contains(s)).unwrap_or(true) {
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
