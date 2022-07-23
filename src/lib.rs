//! Quick and basic implimentation of an interface for writing and reading
//! to MB85RC-series I2C FRAM modules
//! 
//! Developed with the MB85RC256V in mind
//! 
//! Still to-do: figure out how to query the device id and size

mod mb85rc;
pub use mb85rc::{MB85RC, Builder};
