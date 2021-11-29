//! Utilities to easily count events for debuging puposes
//!
//! This can be useful to gather statistics about how often different code
//! paths are run.
//!
//! # Overhead
//!
//! The implementation is very simple and not particularly fast. Using counters
//! will affect perfomance measurements.
//!
//! # Optimizing out
//!
//! If the feature flag `noop` is enabled, the counters compile into an empty struct
//! with empty methods. This way the code for counting events can be kept while opting
//! out of its overhead in shipping and profiling build configurations.
//!
//! # Example
//!
//! In the example below we have a function `do_the_thing` which we determined to
//! be expensive (using a profiler). We would like to get some insight into how
//! often the function is run and how often we take the slow and fast paths.
//!
//! ```rust
//! use counters::Counters;
//! use counters::filters::*;
//!
//! struct Foo {
//!     counters: Counters,
//! }
//!
//! impl Foo {
//!     // This method is not mutable (&self), however we can still update
//!     // the counters because they use internal mutability.
//!     fn do_the_thing(&self, n: u32) -> u32 {
//!         self.counters.event("do_the_thing");
//!         if n % 17 == 0 {
//!             self.counters.event("fast path A");
//!             return self.foo();
//!         }
//!
//!         if n % 56 == 0 {
//!             self.counters.event("fast path B");
//!             return self.bar();
//!         }
//!
//!         self.counters.event("slow path");
//!         return self.baz();
//!     }
//!
//!     fn do_all_of_the_things(&mut self) {
//!         self.counters.reset_all();
//!
//!         for i in 0..100 {
//!             self.do_the_thing(i);
//!         }
//!
//!         // We can use filters to accumulate the values of several counters.
//!         let total_fast_path = self.counters.accumulate(Contains("fast path"));
//!         let slow_path = self.counters.get("do_the_thing") - total_fast_path;
//!
//!         // Set the value of a counter.
//!         self.counters.set("slow path", slow_path);
//!
//!         // This prints the following to stdout:
//!         // slow path: 93
//!         // fast path A: 6
//!         // fast path B: 1
//!         // do_the_thing: 100
//!         self.counters.print_to_stdout(All);
//!     }
//!
//!     // Let's pretend the methods below do interesting things...
//!     fn foo(&self) -> u32 { 0 }
//!     fn bar(&self) -> u32 { 0 }
//!     fn baz(&self) -> u32 { 0 }
//! }
//!
//! ```

#[macro_use]
#[cfg(feature = "serialization")]
extern crate serde;

#[cfg(not(feature = "noop"))]
mod counters;
#[cfg(not(feature = "noop"))]
pub use crate::counters::*;

#[cfg(not(feature = "noop"))]
mod table;
#[cfg(not(feature = "noop"))]
pub use crate::table::*;

#[cfg(feature = "noop")]
mod noop;
#[cfg(feature = "noop")]
pub use crate::noop::*;

pub mod filters;

#[test]
#[cfg(not(feature = "noop"))]
fn it_works() {
    use crate::filters::*;

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
    use crate::filters::*;

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
