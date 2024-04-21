use std::{collections::VecDeque, f32::NAN, ops::Range};

use crate::Color;

pub type CounterId = usize;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Format {
    Int,
    Float,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Orientation {
    Vertical,
    Horizontal,
}

pub struct Counter {
    pub(crate) current_value: f32,
    pub(crate) sum: f32,
    pub(crate) samples: f32,
    pub(crate) min: f32,
    pub(crate) max: f32,
    pub(crate) displayed_avg: f32,
    pub(crate) displayed_min: f32,
    pub(crate) displayed_max: f32,
    pub(crate) descriptor: CounterDescriptor,
    pub(crate) history: VecDeque<f32>,
}

impl Counter {
    pub fn new(descritpor: CounterDescriptor) -> Self {
        Counter {
            current_value: NAN,
            sum: 0.0,
            samples: 0.0,
            min: 0.0,
            max: 0.0,
            displayed_avg: NAN,
            displayed_min: NAN,
            displayed_max: NAN,
            descriptor: descritpor,
            history: VecDeque::new(),
        }
    }

    pub fn set(&mut self, value: impl Into<Option<f32>>) {
        self.current_value = value.into().unwrap_or(NAN);
    }

    pub fn update(&mut self, update_avg: bool) {
        if self.current_value.is_finite() {
            self.samples += 1.0;
            self.sum += self.current_value;
            self.min = self.min.min(self.current_value);
            self.max = self.max.max(self.current_value);
        }

        if !self.history.is_empty() {
            self.history.pop_front();
            self.history.push_back(self.current_value)
        }

        self.current_value = NAN;

        if update_avg {
            if self.samples > 0.0 {
                self.displayed_avg = self.sum / self.samples;
                self.displayed_max = self.max;
                self.displayed_min = self.min;
            } else {
                self.displayed_avg = NAN;
                self.displayed_max = NAN;
                self.displayed_min = NAN;
            }
            self.samples = 0.0;
            self.sum = 0.0;
            self.min = std::f32::MAX;
            self.max = std::f32::MIN;    
        }
    }

    pub fn name(&self) -> &'static str {
        self.descriptor.name
    }

    pub fn history(&self) -> Option<HistoryIter> {
        if self.history.is_empty() {
            return None;
        }

        Some(HistoryIter {
            inner: self.history.iter()
        })
    }

    pub fn enable_history(&mut self, samples: usize) {
        self.history.clear();
        self.history.reserve(samples);
        for _ in 0..samples {
            self.history.push_back(NAN);
        }
    }

    pub fn disable_history(&mut self) {
        self.history = VecDeque::new();
    }
}


#[derive(Clone, Debug)]
pub struct CounterDescriptor {
    pub name: &'static str,
    pub unit: &'static str,
    pub id: CounterId,
    pub format: Format,
    pub color: Color,
    pub safe_range: Option<Range<f32>>,
}

impl CounterDescriptor {
    pub const fn int(name: &'static str, unit: &'static str, id: CounterId) -> Self {
        CounterDescriptor {
            name,
            unit,
            id,
            format: Format::Int,
            color: (255, 255, 255, 255),
            safe_range: None,
        }
    }

    pub const fn float(name: &'static str, unit: &'static str, id: CounterId) -> Self {
        CounterDescriptor {
            name,
            unit,
            id,
            format: Format::Float,
            color: (255, 255, 255, 255),
            safe_range: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn safe_range(mut self, range: Range<f32>) -> Self {
        self.safe_range = Some(range);
        self
    }
}

pub struct HistoryIter<'l> {
    inner: std::collections::vec_deque::Iter<'l, f32>,
}

impl<'l> Iterator for HistoryIter<'l> {
    type Item = Option<f32>;
    fn next(&mut self) -> Option<Option<f32>> {
        self.inner.next().map(
            |val| if val.is_finite() { Some(*val) } else { None }
        )
    }
}

pub struct Counters {
    counters: Vec<Counter>,
    history_size: usize,
    counter_avg_window: u32,
    frame_idx: u32,
}

impl Counters {
    pub fn new(descriptors: &[CounterDescriptor], history_size: usize) -> Self {
        let mut counters = Vec::with_capacity(descriptors.len());
        let mut idx = 0;
        for desc in descriptors {
            assert_eq!(desc.id, idx);
            idx += 1;
            counters.push(Counter::new(desc.clone()));
        }

        Counters { counters, history_size, counter_avg_window: 30, frame_idx: 0 }
    }

    pub fn update(&mut self) {
        self.frame_idx += 1;
        let update_avg = self.frame_idx == self.counter_avg_window;
        for counter in &mut self.counters {
            counter.update(update_avg);
        }
        if update_avg {
            self.frame_idx = 0;
        }
    }

    pub fn set(&mut self, key: CounterId, val: impl Into<Option<f32>>) {
        self.counters[key].set(val);
    }

    pub fn enable_history(&mut self, key: CounterId) {
        self.counters[key].enable_history(self.history_size);
    }

    pub fn disable_history(&mut self, key: CounterId) {
        self.counters[key].disable_history();
    }

    pub fn select_counters<'b, 'a: 'b>(&'a self, keys: impl Iterator<Item = CounterId>, selection: &'b mut Vec<&'a Counter>) {
        for key in keys {
            selection.push(&self.counters[key]);
        }
    }

    pub fn get_counter(&self, key: CounterId) -> &Counter {
        &self.counters[key]
    }
}

#[test]
fn history() {
    let mut c = Counter::new(CounterDescriptor::float("foo", "", 0));
    c.enable_history(6);
    c.set(1.0);
    c.update(false);
    c.set(2.0);
    c.update(false);
    c.set(None);
    c.update(false);
    c.set(4.0);
    c.update(false);
    c.set(5.0);
    c.update(false);
    let samples: Vec<Option<f32>> = c.history().unwrap().collect();
    assert_eq!(&samples[..], &[None, Some(1.0), Some(2.0), None, Some(4.0), Some(5.0)]);
}
