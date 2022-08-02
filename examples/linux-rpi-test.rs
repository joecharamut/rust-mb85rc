use std::io::{Seek, Read, Write};
use rand::{self, RngCore};

use linux_embedded_hal::I2cdev;
use mb85rc::Builder;

fn test_sized<T>(f: &mut T, size: usize, at: u64)
where
    T: Read + Seek + Write
{
    print!("Test write (array of {}) ({} to {}): ", size, at, at + size as u64);

    // get some random data
    let mut random_data = vec![0u8; size];
    rand::thread_rng().fill_bytes(&mut random_data);

    // prepare to write it
    if let Err(e) = f.seek(std::io::SeekFrom::Start(at)) {
        println!("FAIL (Seek)");
        println!("{}", e);
        return;
    }

    // do the write
    if let Err(e) = f.write(&random_data) {
        println!("FAIL (Write)");
        println!("{}", e);
        return;
    }

    println!("OK");

    print!("Test read: ");

    // read buf with filler bytes
    let mut big_buf = vec![0xCDu8; size];

    // prepare to read
    if let Err(e) = f.seek(std::io::SeekFrom::Start(at)) {
        println!("FAIL (Seek)");
        println!("{}", e);
        return;
    }

    // do it
    if let Err(e) = f.read(&mut big_buf) {
        println!("FAIL (Read)");
        println!("{}", e);
        return;
    }

    // make sure it actually reads back
    assert_eq!(big_buf, random_data);
    println!("OK");
}

fn main() {
    // open /dev/i2c-1 because that's what the raspi exposes as the main i2c bus
    let i2c = I2cdev::new("/dev/i2c-1").unwrap();

    // default address for the fram is 0x50
    // let the library auto detect size
    let mut fram = Builder::new().with_address(0x50).connect_i2c(i2c);

    // make sure the capacity is there
    println!("Fram capacity: {:?}", fram.fram_size());

    // test sizes in increments of powers of 2
    for i in 0..16 {
        let size = 1 << i;
        
        if size > fram.fram_size() {
            break;
        }

        test_sized(&mut fram, 1 << i, 0);
    }
}
