use embedded_hal::blocking::i2c;
use std::io::{Seek, SeekFrom, Read, Write, Error, ErrorKind};

/// Builder to create the interface with parameters
pub struct Builder {
    device_addr: u8,
    device_size: i32,
}

impl Builder {
    /// Create a new builder with default parameters
    pub fn new() -> Self {
        Self { device_addr: 0x50, device_size: -1, }
    }

    /// Set the I2C device address for the FRAM module
    pub fn with_address(mut self, address: u8) -> Self {
        self.device_addr = address;
        self
    }

    /// Set the size of the FRAM module in bytes (not currently used for anything)
    pub fn with_size(mut self, size: i32) -> Self {
        self.device_size = size;
        self
    }

    /// Finish the builder and construct the interface by attaching an I2C bus
    pub fn connect_i2c<I2C>(self, i2c: I2C) -> MB85RC<I2C>
    where 
        I2C: i2c::WriteRead + i2c::Write
    {
        MB85RC::new(i2c, self.device_addr)
    }
}

/// Interface for the FRAM module over I2C
/// 
/// Construct this using a [`Builder`] to set the address, size, and etc
pub struct MB85RC<I2C> {
    i2c: I2C,
    device_addr: u8,
    cursor: u16,
}

impl<I2C> MB85RC<I2C>
where
    I2C: i2c::WriteRead + i2c::Write
{
    fn new(i2c: I2C, device_addr: u8) -> Self {
        Self { i2c, device_addr, cursor: 0 }
    }

    fn fram_read(&mut self, addr: u16, buf: &mut [u8]) -> Result<usize, <I2C as i2c::WriteRead>::Error> {
        let addr_hi = (addr >> 8) as u8;
        let addr_lo = (addr & 0xFF) as u8;
        let addr_buf = [addr_hi, addr_lo];

        self.i2c.write_read(self.device_addr, &addr_buf, buf).and(Ok(buf.len()))
    }

    fn fram_write(&mut self, addr: u16, buf: &[u8]) -> Result<usize, <I2C as i2c::Write>::Error> {
        let addr_hi = (addr >> 8) as u8;
        let addr_lo = (addr & 0xFF) as u8;
        let addr_buf = [addr_hi, addr_lo];
        let mut write_vec = addr_buf.to_vec();
        write_vec.extend_from_slice(buf);

        let write_buf = write_vec.as_slice();

        self.i2c.write(self.device_addr, write_buf).and(Ok(buf.len()))
    }
}

impl<I2C> Seek for MB85RC<I2C> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match pos {
            SeekFrom::Start(p) => {
                self.cursor = p as u16;
                Ok(self.cursor.into())
            },
            SeekFrom::Current(p) => {
                if p < 0 {
                    self.cursor = self.cursor - (p.abs() as u16);
                } else {
                    self.cursor = self.cursor + p as u16;
                }
                
                Ok(self.cursor.into())
            },
            SeekFrom::End(_) => {
                unimplemented!("MB85RC cannot be seeked from the end")
            },
        }
    }
}

impl<I2C> Read for MB85RC<I2C> 
where
    I2C: i2c::WriteRead + i2c::Write
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // TODO: properly return an error
        self.fram_read(self.cursor, buf).map_err(|_| Error::new(ErrorKind::Other, "I2C Error"))
    }
}

impl<I2C> Write for MB85RC<I2C>
where
    I2C: i2c::WriteRead + i2c::Write
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // TODO: properly return an error
        self.fram_write(self.cursor, buf).map_err(|_| Error::new(ErrorKind::Other, "I2C Error"))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // No need to flush anything
        Ok(())
    }
}
