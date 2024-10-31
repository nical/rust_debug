use std::{collections::VecDeque, f32::NAN, ops::Range};

#[doc(hidden)]
#[macro_export]
macro_rules! _counter_format {
    (int) => {
        $crate::Format::Int
    };
    (float) => {
        $crate::Format::Float
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _declare_counter {
    ($name:path : $format:ident = $strname:literal) => {
        //CounterDescriptor::$format($strname, $unit, $name as usize)
        $crate::CounterDescriptor {
            name: $strname,
            id: $crate::CounterId($name as u16),
            format: $crate::_counter_format!($format),
            .. CounterDescriptor::_DEFAULT
        }
    };
    ($name:path : $format:ident = $strname:literal with { $( $prop:ident: $val:expr ),* } ) => {
        //CounterDescriptor::$format($strname, $unit, $name as usize)
        $crate::CounterDescriptor {
            name: $strname,
            id: $crate::CounterId($name as u16),
            format: $crate::_counter_format!($format),
            $( $prop:$val ),*,
            .. CounterDescriptor::_DEFAULT
        }
    };
}

#[macro_export]
macro_rules! declare_counters {
    ($group_name:ident = {
        $(
            $name:ident : $format:ident = $strname:literal $(with { $( $prop:ident : $val:expr ),* })*
        ),+
    }) => {
        pub mod $group_name {
            #![allow(non_camel_case_types)]
            #![allow(non_snake_case)]
            #![allow(non_upper_case_globals)]
            #![allow(unused)]

            use $crate::CounterDescriptor;
            #[repr(u16)]
            enum Indices {
                $($name),+
            }

            $(
                pub const $name: $crate::CounterIndex = $crate::CounterIndex(Indices::$name as u16);
            )+

            pub const DESCRIPTORS : &[CounterDescriptor] = &[$(
                $crate::_declare_counter!(Indices::$name : $format = $strname $(with { $($prop:$val),* })* )
            ),+];

            #[derive(Copy, Clone, Debug, PartialEq, Eq)]
            pub struct Ids(pub $crate::CounterGroup);

            impl Ids {
                pub fn group(self) -> $crate::CounterGroup { self.0 }
                $(
                pub fn $name(self) -> $crate::CounterId {
                    self.0.counter($name)
                }
                )*
                pub fn counter(self, idx: $crate::CounterIndex) -> $crate::CounterId {
                    self.0.counter(idx)
                }
                pub fn all(self) -> impl Iterator<Item=$crate::CounterId> {
                    self.0.all()
                }
            }

            pub fn register(name: &'static str, counters: &mut $crate::Counters) -> Ids {
                Ids(counters.register_group(name, DESCRIPTORS))
            }
        }
    }
}

use crate::Color;

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CounterId(pub u16);

impl CounterId {
    #[inline]
    fn index(self) -> usize {
        self.0 as usize
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CounterIndex(pub u16);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CounterGroup {
    start: u16,
    end: u16,
}

impl CounterGroup {
    pub fn counter(self, index: CounterIndex) -> CounterId {
        CounterId(self.start + index.0)
    }

    pub fn all(self) -> impl Iterator<Item = CounterId> {
        CounterIterator {
            indices: self.start..self.end,
        }
    }
}

pub struct CounterIterator {
    indices: Range<u16>,
}

impl Iterator for CounterIterator {
    type Item = CounterId;
    fn next(&mut self) -> Option<CounterId> {
        Some(CounterId(self.indices.next()?))
    }
}

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
    pub(crate) last_value: f32,
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
            last_value: NAN,
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
        if let Some(val) = value.into() {
            self.current_value = val;
            self.last_value = val;
        } else {
            self.current_value = NAN;
        }
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
            inner: self.history.iter(),
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
    #[doc(hidden)]
    pub const _DEFAULT: Self = CounterDescriptor {
        name: "",
        unit: "",
        id: CounterId(std::u16::MAX),
        format: Format::Int,
        color: (255, 255, 255, 255),
        safe_range: None,
    };

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
        self.inner
            .next()
            .map(|val| if val.is_finite() { Some(*val) } else { None })
    }
}

struct Group {
    name: &'static str,
    range: Range<u16>,
}

pub struct Counters {
    groups: Vec<Group>,
    counters: Vec<Counter>,
    history_size: usize,
    counter_avg_window: u32,
    frame_idx: u32,
}

impl Counters {
    pub fn new(history_size: usize) -> Self {
        Counters {
            counters: Vec::new(),
            groups: Vec::new(),
            history_size,
            counter_avg_window: 30,
            frame_idx: 0,
        }
    }

    pub fn register_group(
        &mut self,
        name: &'static str,
        descriptors: &[CounterDescriptor],
    ) -> CounterGroup {
        let start = self.counters.len();
        let end = start + descriptors.len();
        assert!(
            end < (std::u16::MAX as usize),
            "Registered too many counters"
        );
        let start = start as u16;
        let end = end as u16;

        self.groups.push(Group {
            name,
            range: start..end,
        });
        self.counters.reserve(descriptors.len());
        let mut idx = 0;
        for desc in descriptors {
            assert_eq!(desc.id.index(), idx);
            idx += 1;
            self.counters.push(Counter::new(desc.clone()));
        }

        CounterGroup { start, end }
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

    pub fn set(&mut self, id: CounterId, val: impl Into<Option<f32>>) {
        self.counters[id.index()].set(val);
    }

    pub fn enable_history(&mut self, id: CounterId) {
        self.counters[id.index()].enable_history(self.history_size);
    }

    pub fn disable_history(&mut self, id: CounterId) {
        self.counters[id.index()].disable_history();
    }

    pub fn select_counters<'b, 'a: 'b>(
        &'a self,
        ids: impl Iterator<Item = CounterId>,
        selection: &'b mut Vec<&'a Counter>,
    ) {
        for id in ids {
            selection.push(&self.counters[id.index()]);
        }
    }

    pub fn get_counter(&self, id: CounterId) -> &Counter {
        &self.counters[id.index()]
    }

    pub fn find_group_by_name(&self, group_name: &str) -> Option<CounterGroup> {
        let group = self.groups.iter().find(|g| g.name == group_name)?;

        return Some(CounterGroup {
            start: group.range.start,
            end: group.range.end,
        });
    }

    pub fn find_counter_by_name(&self, group_name: &str, counter_name: &str) -> Option<CounterId> {
        let range = self
            .groups
            .iter()
            .find(|g| g.name == group_name)?
            .range
            .clone();

        let usize_range = range.start as usize..range.end as usize;
        for (idx, c) in self.counters[usize_range.clone()].iter().enumerate() {
            if c.name() == counter_name {
                return Some(CounterId(idx as u16 + range.start));
            }
        }

        None
    }
}

#[test]
fn history() {
    let mut c = Counter::new(CounterDescriptor::float("foo", "", CounterId(0)));
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
    assert_eq!(
        &samples[..],
        &[None, Some(1.0), Some(2.0), None, Some(4.0), Some(5.0)]
    );
}

#[test]
fn decl_counters() {
    declare_counters!(wgpu_counters = {
        textures: int = "hal.textures",
        texture_memory: int = "hal.texture-memory" with { unit: "MB", color: (255, 0, 0, 255) },
        buffer_memory: int = "hal.buffer-memory" with { unit: "MB", color: (128, 128, 0, 255) }
    });

    declare_counters!(app_counters = {
        ENTITIES: int = "entities",
        _RETRIES: int = "retries",
        _WATER_LEVEL: int = "water-level" with { unit: "m" }
    });

    println!("TEXTURES: {:?}", wgpu_counters::textures);
    println!("TEXTURE_MEMORY: {:?}", wgpu_counters::texture_memory);
    println!("BUFFER_MEMORY: {:?}", wgpu_counters::buffer_memory);
    println!("{:#?}", wgpu_counters::DESCRIPTORS);

    let mut counters = Counters::new(60);
    let wgpu = counters.register_group("wgpu", wgpu_counters::DESCRIPTORS);
    let app = counters.register_group("app", app_counters::DESCRIPTORS);

    let entities = counters.find_counter_by_name("app", "entities").unwrap();
    let texture_mem = counters
        .find_counter_by_name("wgpu", "hal.texture-memory")
        .unwrap();

    assert_eq!(entities, app.counter(app_counters::ENTITIES));
    assert_eq!(texture_mem, wgpu.counter(wgpu_counters::texture_memory));

    assert_eq!(counters.get_counter(entities).name(), "entities");
    assert_eq!(
        counters.get_counter(texture_mem).name(),
        "hal.texture-memory"
    );
}

// Mirrors wgpu::InternalCounters
#[cfg(any(feature = "wgpu-core", feature = "wgpu"))]
crate::declare_counters!(wgpu_counters = {
    buffers: int = "buffers",
    textures: int = "textures",
    texture_views: int = "texture_views",
    bind_groups: int = "bind_groups",
    bind_group_layouts: int = "bind group layouts",
    render_pipelines: int = "render pipelines",
    compute_pipelines: int = "compute pipelines",
    pipeline_layouts: int = "pipeline layouts",
    samplers: int = "samplers",
    command_encoders: int = "command encoders",
    shader_modules: int = "shader modules",
    query_sets: int = "query sets",
    fences: int = "fences",
    buffer_memory: float = "buffer memory" with { unit: "MB" },
    texture_memory: float = "texture memory" with { unit: "MB" },
    memory_allocations: int = "memory allocations"
});

#[cfg(any(feature = "wgpu-core", feature = "wgpu"))]
pub fn update_wgpu_internal_counters(
    counters: &mut Counters,
    ids: wgpu_counters::Ids,
    values: &wgpu::InternalCounters,
) {
    pub fn mb(bytes: isize) -> f32 {
        bytes as f32 / 1_000_000.0
    }
    counters.set(ids.buffers(), values.hal.buffers.read() as f32);
    counters.set(ids.textures(), values.hal.textures.read() as f32);
    counters.set(ids.texture_views(), values.hal.texture_views.read() as f32);
    counters.set(ids.bind_groups(), values.hal.bind_groups.read() as f32);
    counters.set(
        ids.bind_group_layouts(),
        values.hal.bind_group_layouts.read() as f32,
    );
    counters.set(
        ids.render_pipelines(),
        values.hal.render_pipelines.read() as f32,
    );
    counters.set(
        ids.compute_pipelines(),
        values.hal.compute_pipelines.read() as f32,
    );
    counters.set(
        ids.pipeline_layouts(),
        values.hal.pipeline_layouts.read() as f32,
    );
    counters.set(ids.samplers(), values.hal.samplers.read() as f32);
    counters.set(
        ids.command_encoders(),
        values.hal.command_encoders.read() as f32,
    );
    counters.set(
        ids.shader_modules(),
        values.hal.shader_modules.read() as f32,
    );
    counters.set(ids.query_sets(), values.hal.query_sets.read() as f32);
    counters.set(ids.fences(), values.hal.fences.read() as f32);
    counters.set(ids.buffer_memory(), mb(values.hal.buffer_memory.read()));
    counters.set(ids.texture_memory(), mb(values.hal.texture_memory.read()));
    counters.set(
        ids.memory_allocations(),
        values.hal.memory_allocations.read() as f32,
    );
}
