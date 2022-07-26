use embedded_hal::blocking::i2c;
use core::fmt;
use std::error::Error;
use std::io::{Seek, SeekFrom, Read, Write, ErrorKind};
use std::io;

/// Interface for the FRAM module over I2C
/// 
/// Construct this using a [`Builder`] to set the address and size
pub struct MB85RC<I2C> {
    i2c: I2C,
    device_addr: u8,
    device_size: u32,
    cursor: u16,
}

impl<I2C> MB85RC<I2C>
where
    I2C: i2c::WriteRead + i2c::Write,
    <I2C as i2c::WriteRead>::Error: Error,
    <I2C as i2c::Write>::Error: Error,
{
    fn new(mut i2c: I2C, device_addr: u8, size: Option<u32>) -> Self {
        let device_size = match size {
            Some(s) => s,
            None => {
                let meta = match Self::read_metadata(&mut i2c, device_addr) {
                    Ok(v) => v,
                    Err(_) => {
                        panic!("Could not automatically get FRAM size. Use `Builder::with_size(u32)`.");
                    },
                };
                let size = (1 << (meta[1] & 0xF)) * 1024;
                println!("Device size reports to be {} bytes.", size);
                size
            },
        };

        Self {
            i2c,
            device_addr,
            device_size,
            cursor: 0,
        }
    }

    /// Directly read bytes at `addr` into the provided buffer
    pub fn fram_read(&mut self, addr: u16, buf: &mut [u8]) -> Result<usize, Mb85rcError> {
        let addr_hi = (addr >> 8) as u8;
        let addr_lo = (addr & 0xFF) as u8;
        let addr_buf = [addr_hi, addr_lo];

        match self.i2c.write_read(self.device_addr, &addr_buf, buf) {
            Ok(_) => Ok(buf.len()),
            Err(e) => Err(Mb85rcError::new(format!("I2C Error: {}", e).as_str())),
        }
    }

    /// Directly write bytes at `addr` from the provided buffer
    pub fn fram_write(&mut self, addr: u16, buf: &[u8]) -> Result<usize, Mb85rcError> {
        let addr_hi = (addr >> 8) as u8;
        let addr_lo = (addr & 0xFF) as u8;
        let addr_buf = [addr_hi, addr_lo];
        let write_buf = [&addr_buf, buf].concat();

        match self.i2c.write(self.device_addr, &write_buf) {
            Ok(_) => Ok(buf.len()),
            Err(e) => Err(Mb85rcError::new(format!("I2C Error: {}", e).as_str())),
        }
    }

    fn read_metadata(i2c: &mut I2C, addr: u8) -> Result<[u8;3], Mb85rcError> {
        // density of the FRAM module is 2^N kB, where N is the lower nybble of the second metadata byte
        let write_buf = [addr << 1];
        let mut read_buf = [0u8; 3];

        match i2c.write_read(0xF8 >> 1, &write_buf, &mut read_buf) {
            Ok(_) => Ok(read_buf),
            Err(e) => Err(Mb85rcError::new(format!("I2C Error: {}", e).as_str())),
        }
    }

    /// Get the auto-detected or [manually set](Builder::with_size) size of the device
    pub fn fram_size(&self) -> u32 {
        self.device_size
    }
}

impl<I2C> Seek for MB85RC<I2C> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match pos {
            SeekFrom::Start(p) => {
                let new_cursor = p as i64;

                if new_cursor >= self.device_size.into() {
                    Err(io::Error::new(ErrorKind::UnexpectedEof, "Cannot seek past device memory size"))
                } else {
                    self.cursor = p as u16;
                    Ok(self.cursor.into())
                }
            },
            SeekFrom::Current(p) => {
                let new_cursor = (self.cursor as i64) + p;
                
                if new_cursor < 0 {
                    Err(io::Error::new(ErrorKind::InvalidInput, "Invalid argument (position would be negative)"))
                } else {
                    self.cursor = new_cursor as u16;
                    Ok(self.cursor.into())
                }
            },
            SeekFrom::End(p) => {
                let new_cursor = (self.cursor as i64) + p;

                if new_cursor < 0 {
                    Err(io::Error::new(ErrorKind::InvalidInput, "Invalid argument (position would be negative)"))
                } else if new_cursor >= self.device_size.into() {
                    Err(io::Error::new(ErrorKind::UnexpectedEof, "Cannot seek past device memory size"))
                } else {
                    self.cursor = new_cursor as u16;
                    Ok(self.cursor.into())
                }
            },
        }
    }
}

impl<I2C> Read for MB85RC<I2C> 
where
    I2C: i2c::WriteRead + i2c::Write,
    <I2C as i2c::WriteRead>::Error: Error,
    <I2C as i2c::Write>::Error: Error,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.fram_read(self.cursor, buf).map_err(|e| io::Error::new(ErrorKind::Other, e))
    }
}

impl<I2C> Write for MB85RC<I2C>
where
    I2C: i2c::WriteRead + i2c::Write,
    <I2C as i2c::WriteRead>::Error: Error,
    <I2C as i2c::Write>::Error: Error,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.fram_write(self.cursor, buf).map_err(|e| io::Error::new(ErrorKind::Other, e))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // No need to flush anything
        Ok(())
    }
}

/// Builder to create the interface with parameters
pub struct Builder {
    device_addr: u8,
    device_size: Option<u32>,
}

impl Builder {
    /// Create a new builder with default parameters
    pub fn new() -> Self {
        Self {
            device_addr: 0x50,
            device_size: None,
        }
    }

    /// Set the I2C device address for the FRAM module
    pub fn with_address(mut self, address: u8) -> Self {
        self.device_addr = address;
        self
    }

    /// Set the size of the FRAM module in bytes (overrides auto-detection)
    pub fn with_size(mut self, size: u32) -> Self {
        self.device_size = Some(size);
        self
    }

    /// Finish the builder and construct the interface by attaching an I2C bus
    pub fn connect_i2c<I2C>(self, i2c: I2C) -> MB85RC<I2C>
    where 
        I2C: i2c::WriteRead + i2c::Write,
        <I2C as i2c::WriteRead>::Error: Error,
        <I2C as i2c::Write>::Error: Error,
    {
        MB85RC::new(i2c, self.device_addr, self.device_size)
    }
}

/// Error type for anything that might happen on the I2C side of things
#[derive(Debug)]
pub struct Mb85rcError {
    details: String,
}

impl Mb85rcError {
    fn new(msg: &str) -> Mb85rcError {
        Mb85rcError { details: msg.to_string() }
    }
}

impl fmt::Display for Mb85rcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for Mb85rcError {
    fn description(&self) -> &str {
        &self.details
    }
}
