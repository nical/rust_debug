use std::io;
use crate::filters::Filter;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Counters;

impl Counters {
    pub fn new() -> Self { Counters }
    pub fn event(&self, _key: &str) {}
    pub fn reset_event(&self, _key: &str) {}
    pub fn reset_events<F: Filter>(&self, _filter: F) {}
    pub fn reset_all(&self) {}
    pub fn retain<F: Filter>(&self, _filter: F) {}
    pub fn get(&self, _key: &str) -> u64 { 0 }
    pub fn accumulate<F: Filter>(&self, _filter: F) -> u64 { 0 }
    pub fn print<F: Filter>(&self, _filter: F, _out: &mut io::Write) -> io::Result<()> { Ok(()) }
    pub fn print_to_stdout<F: Filter>(&self, _filter: F) {}
}

#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Table;

impl Table {
    pub fn new<Label>(_labels: &[Label]) -> Self where Label: ToString { Table }
    pub fn add_row(&self, _row: &Counters) -> usize { 0 }
    pub fn print(&self, _to: &mut io::Write) -> io::Result<()> { Ok(()) }
    pub fn print_to_stdout(&self) {}
}
