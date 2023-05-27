pub mod clock;
pub mod disk;
pub mod graphics;
pub mod network;
pub mod printer;
pub mod serial;
pub mod usb;

use crate::Error;
use self::disk::{ide::IDE_DISKS, disk::init_disks};

pub trait Device {
    fn open(&self) -> Result<(), Error>;
    fn close(&self) -> Result<(), Error>;
    fn control(&self, code: u32, value: usize) -> Result<(), Error>;
}

pub trait CharacterDevice: Device {
    fn read(&self, buf: &mut [u8]) -> Result<usize, Error>;
    fn write(&self, buf: &[u8]) -> Result<usize, Error>;
}

pub trait BlockDevice: Device {
    fn read_block(&self, buf: &mut [u8]) -> Result<(), Error>; 
    fn write_block(&self, buf: &[u8]) -> Result<(), Error>;
    fn block_size(&self) -> usize; 
    fn size(&self) -> usize;
}

pub fn devices_init() {
    init_disks();
    // let _ =IDE_DISKS[0].init();
    // IDE_DISKS[1].init();
}
