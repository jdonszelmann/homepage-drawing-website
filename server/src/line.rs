use serde::{Serialize,Deserialize};

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct Line(
    pub f64, // x
    pub f64, // y
    pub f64, // oldx
    pub f64, // oldy
    pub u8,  // color
);
