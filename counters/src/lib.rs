#[macro_use]
#[cfg(feature = "serialization")]
extern crate serde;

#[cfg(not(feature="noop"))] mod counters;
#[cfg(not(feature="noop"))] pub use crate::counters::*;

#[cfg(feature="noop")] mod noop;
#[cfg(feature="noop")] pub use crate::noop::*;

mod filter;
pub use filter::*;

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

    counters.retain(Select(|key, _| key == "meh"));

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
