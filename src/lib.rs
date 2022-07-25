#![warn(missing_docs)]
//! Quick and basic implimentation of an interface for writing and reading
//! to MB85RC-series I2C FRAM modules
//! 
//! Developed with the MB85RC256V in mind

mod mb85rc;
pub use mb85rc::{MB85RC, Builder};
