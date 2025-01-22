use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub mod mandelbrot;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Analytics {
    pub ip_address: String,
    pub path: String,
    pub iso_code: String,
    pub count: usize,
}

impl Display for Analytics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {}, {}, {})",
            self.ip_address, self.iso_code, self.path, self.count
        )
    }
}
