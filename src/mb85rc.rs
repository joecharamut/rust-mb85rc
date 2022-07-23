use embedded_hal::blocking::i2c::{Transactional, Operation, WriteRead};

pub struct MB85RC<I2C> {
    i2c: I2C,
    device_addr: u8,
}

impl<I2C> MB85RC<I2C>
where
    I2C: WriteRead
{
    pub fn new(i2c: I2C, device_addr: u8) -> Self {
        Self { i2c, device_addr }
    }

    pub fn read_bytes(&mut self, addr: u16, buf: &mut [u8]) -> Result<(), I2C::Error> {
        let addr_hi = (addr >> 8) as u8;
        let addr_lo = (addr & 0xFF) as u8;
        let addr_buf = [addr_hi, addr_lo];

        self.i2c.write_read(self.device_addr, &addr_buf, buf)
    }

    pub fn write_bytes(&mut self, addr: u16, buf: &[u8]) -> Result<(), I2C::Error> {
        let addr_hi = (addr >> 8) as u8;
        let addr_lo = (addr & 0xFF) as u8;
        let addr_buf = [addr_hi, addr_lo];
        let mut ops = [
            // assert start condition (automatic)
            // send device address (auto)
            // send r/w bit (auto)
            Operation::Write(&addr_buf), // address of byte
            Operation::Write(buf), // read data
        ];
        self.i2c.exec(self.device_addr, &mut ops)
    }

    fn command_device_id(&mut self, buf: &mut [u8; 3]) -> Result<(), I2C::Error> {
        let dev_addr_buf = [self.device_addr];
        let device_id2_buf = [0xF9];

        let mut ops = [
            Operation::Write(&dev_addr_buf),
            Operation::Write(&device_id2_buf),
            Operation::Read(buf),
        ];

        self.i2c.exec(0xF8, &mut ops)
    }

    pub fn read_manufacturer_id(&mut self) -> Result<u16, I2C::Error> {
        let mut buf = [0u8, 0u8, 0u8];
        self.command_device_id(&mut buf)?;
        let mfr = ((buf[0] as u16) << 4) | (((buf[1] as u16) >> 4) & 0xF);
        Ok(mfr)
    }

    pub fn read_capacity(&mut self) -> Result<u16, I2C::Error> {
        let mut buf = [0u8, 0u8, 0u8];
        self.command_device_id(&mut buf)?;
        // capacity is 2^n bytes, where n is the lower nybble of device id byte 1
        let cap = 1 << ((buf[1] as u16) & 0xF);
        Ok(cap)
    }
}
