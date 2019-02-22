
pub trait Filter {
    fn apply(&mut self, key: &str, value: u64) -> bool;
}

/// Accept all keys
pub struct All;
impl Filter for All {
    fn apply(&mut self, _: &str, _: u64) -> bool { true }
}

/// Select keys using a callback.
pub struct Select<F: FnMut(&str, u64) -> bool>(pub F);

impl<F: FnMut(&str, u64) -> bool> Filter for Select<F> {
    fn apply(&mut self, key: &str, val: u64) -> bool {
        self.0(key, val)
    }
}

impl Filter for &str {
    fn apply(&mut self, key: &str, _: u64) -> bool {
        key.contains(&*self)
    }
}

/// Accet keys that contain a given sub-string.
pub struct Contains<'l>(pub &'l str);

impl<'l> Filter for Contains<'l> {
    fn apply(&mut self, key: &str, _: u64) -> bool {
        key.contains(self.0)
    }
}

/// Accet keys that start with a given sub-string.
pub struct StartsWith<'l>(pub &'l str);
impl<'l> Filter for StartsWith<'l> {
    fn apply(&mut self, key: &str, _: u64) -> bool {
        key.starts_with(self.0)
    }
}

/// Accet keys that end with a given sub-string.
pub struct EndsWith<'l>(pub &'l str);
impl<'l> Filter for EndsWith<'l> {
    fn apply(&mut self, key: &str, _: u64) -> bool {
        key.ends_with(self.0)
    }
}

/// Accet counters that are greater than a given value.
pub struct GreaterThan(pub u64);

impl Filter for GreaterThan {
    fn apply(&mut self, _key: &str, val: u64) -> bool {
        val > self.0
    }
}

/// Accet counters that are less than a given value.
pub struct LessThan(pub u64);

impl Filter for LessThan {
    fn apply(&mut self, _key: &str, val: u64) -> bool {
        val < self.0
    }
}
