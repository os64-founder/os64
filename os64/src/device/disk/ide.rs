//see also: https://wiki.osdev.org/IDE
//本文件实现了IDE DMA方式进行硬盘数据的读写
//本文件用到了如下单词缩写：
//ISA: Industry Standard Architecture,工业标准体系结构。
//IDE: Integrated Drive Electronics,集成驱动器电子装置。IDE接口的硬盘，通过IDE线，连接到电脑。
//ATA: Advanced Technology Attachment,高级技术附件。
//DMA: Direct Memory Access,直接内存访问。
//LBA: Logical Block Addressing,逻辑块寻址。
use bitflags::bitflags;
use lazy_static::lazy_static;
use crate::{architecture::x86_64_asm::{asm_out_u8, asm_in_u8, asm_in_u32, asm_out_u32}, serial_println, serial_print};
use super::disk::{DiskIdentifyInfo, SECTOR_SIZE, DiskDriver};

///see also: https://wiki.osdev.org/IDE#Commands
enum AtaCommands {
    ReadPio    = 0x20,
    WritePio   = 0x30,
    Identify   = 0xEC,
}

const ATA_REGISTER_DATA             : u16 = 0x00;   //数据寄存器 
const ATA_REGISTER_ERROR            : u16 = 0x01;   //错误寄存器 
const ATA_REGISTER_SECTOR_COUNT0    : u16 = 0x02;   //扇区计数寄存器0
const ATA_REGISTER_LBA0             : u16 = 0x03;   //
const ATA_REGISTER_LBA1             : u16 = 0x04;   //
const ATA_REGISTER_LBA2             : u16 = 0x05;   //
const ATA_REGISTER_DEVICE           : u16 = 0x06;   //设备及磁头寄存器（各4位?）
const ATA_REGISTER_COMMAND          : u16 = 0x07;   //命令寄存器
const ATA_REGISTER_STATUS           : u16 = 0x07;   //状态寄存器
const ATA_REGISTER_SECTOR_COUNT1    : u16 = 0x08;   //扇区计数寄存器1
const ATA_REGISTER_LBA3             : u16 = 0x09;   //
const ATA_REGISTER_LBA4             : u16 = 0x0A;   //
const ATA_REGISTER_LBA5             : u16 = 0x0B;   //

const PORT_IDE0_BASE        : u16 = 0x1F0;  //
const PORT_IDE1_BASE        : u16 = 0x170;  //

const PORT_IDE0_CONTROL     : u16 = 0x3F6;  //命令控制端口
const PORT_IDE1_CONTROL     : u16 = 0x376;  

bitflags! {
    /// ATA状态
    /// see also: https://wiki.osdev.org/IDE#Status
    struct AtaStatus: u8 {
        const BUSY          = 0b1000_0000;
        const READY         = 0b0100_0000;
        const WRITE_FAULT   = 0b0010_0000;
        const SEEK_COMPLETE = 0b0001_0000;
        const REQUEST_READY = 0b0000_1000;
        const CORRECTED     = 0b0000_0100;
        const INDEX         = 0b0000_0010;
        const ERROR         = 0b0000_0001;
    }

    /// ATA错误
    /// see also: https://wiki.osdev.org/IDE#Errors
    struct AtaError: u8 {
        const BAD_BLOCK             = 0b1000_0000;
        const UNCORRECTABLE_DATA    = 0b0100_0000;
        const MEDIA_CHANGED         = 0b0010_0000;
        const ID_MARK_NOT_FOUND     = 0b0001_0000;
        const MEDIA_CHANGE_REQUEST  = 0b0000_1000;
        const COMMAND_ABORTED       = 0b0000_0100;
        const TRACK_0_NOT_FOUND     = 0b0000_0010;
        const NO_ADDRESS_MARK       = 0b0000_0001;
    }
}

#[derive(Clone,Copy,Debug)]
pub struct IdeDiskDriver {
    index : u8,
    port_base : u16,
    port_control : u16,
}

lazy_static! {
    /// 两条IDE线，各两块硬盘，共4块硬盘
    pub static ref IDE_DISKS : [IdeDiskDriver; 4] = [
        IdeDiskDriver{
            index : 0,
            port_base : PORT_IDE0_BASE,
            port_control : PORT_IDE0_CONTROL,
        },
        IdeDiskDriver{
            index : 1,
            port_base : PORT_IDE0_BASE,
            port_control : PORT_IDE0_CONTROL,
        },
        IdeDiskDriver{
            index : 2,
            port_base : PORT_IDE1_BASE,
            port_control : PORT_IDE1_CONTROL,
        },
        IdeDiskDriver{
            index : 3,
            port_base : PORT_IDE1_BASE,
            port_control : PORT_IDE1_CONTROL,
        },
    ];
}

impl IdeDiskDriver {
    fn wait(&self) {
        while unsafe { asm_in_u8(self.port_base + ATA_REGISTER_STATUS) } & AtaStatus::BUSY.bits != 0 {}
    }

    fn wait_error(&self) -> bool {
        self.wait();
        let status = unsafe { asm_in_u8(self.port_base + ATA_REGISTER_STATUS) };
        status & (AtaStatus::WRITE_FAULT.bits | AtaStatus::ERROR.bits) != 0
    }

    fn select(&self, sector: u64, count: u8) {
        assert_ne!(count, 0);
        self.wait();
        unsafe {
            // generate interrupt
            asm_out_u8(self.port_control, 0);
            asm_out_u8(self.port_base + ATA_REGISTER_SECTOR_COUNT0, count);
            asm_out_u8(self.port_base + ATA_REGISTER_LBA0, (sector & 0xFF) as u8);
            asm_out_u8(self.port_base + ATA_REGISTER_LBA1, ((sector >> 8) & 0xFF) as u8);
            asm_out_u8(self.port_base + ATA_REGISTER_LBA2, ((sector >> 16) & 0xFF) as u8);
            asm_out_u8(self.port_base + ATA_REGISTER_DEVICE,
                0xE0 | ((self.index & 1) << 4) | (((sector >> 24) & 0xF) as u8),
            );
        }
    }
}

impl DiskDriver for IdeDiskDriver {
    fn init(&self) -> Result<DiskIdentifyInfo,()> {
        self.wait();
        unsafe {
            // step1: select drive
            asm_out_u8(self.port_base + ATA_REGISTER_DEVICE, (0xE0 | ((self.index & 1) << 4)) as u8);
            self.wait();

            // step2: send ATA identify command
            asm_out_u8(self.port_base + ATA_REGISTER_COMMAND, AtaCommands::Identify as u8);
            self.wait();

            // step3: polling
            if asm_in_u8(self.port_base + ATA_REGISTER_STATUS) == 0 || self.wait_error() {
                return Err(());
            }

            //读取磁盘信息
            let mut data = [0; SECTOR_SIZE];
            asm_in_u32(self.port_base + ATA_REGISTER_DATA, data.as_mut_ptr(), SECTOR_SIZE);
            let disk_info = *(data.as_mut_ptr() as *mut DiskIdentifyInfo);
            // serial_println!("{:?}", disk_info);
            print_u8_arrays("serial_number = ",disk_info.serial_number.as_ptr(),20);
            print_u8_arrays("firmware_version = ",disk_info.firmware_version.as_ptr(),8);
            print_u8_arrays("model_number = ",disk_info.model_number.as_ptr(),40);
            let total_sector = disk_info.total_user_lba_for_48_address_feature_set;
            let total_kb = total_sector / 2;
            if total_kb < 1024  {
                serial_println!("total_size = {} KB",total_kb);
            } else {
                let total_mb = total_kb as f64 / 1024.0;
                if total_mb < 1024.0 {
                    serial_println!("total_size = {} MB",total_mb);
                } else {
                    let total_gb = total_mb as f64 / 1024.0;
                    serial_println!("total_size = {} GB",total_gb);
                }
            }
            Ok(disk_info)
        }
    }

    /// Read ATA DMA. Block size = 512 bytes.
    fn read(&self, sector: u64, count: usize, data: &mut [u32]) -> Result<(), ()> {
        assert_eq!(data.len(), count * SECTOR_SIZE);
        self.wait();
        unsafe {
            self.select(sector, count as u8);
            asm_out_u8(self.port_base + ATA_REGISTER_COMMAND, AtaCommands::ReadPio as u8);
            for i in 0..count {
                let ptr = &mut data[(i as usize) * SECTOR_SIZE];
                if self.wait_error() {
                    return Err(());
                }
                asm_in_u32(self.port_base + ATA_REGISTER_DATA, ptr, SECTOR_SIZE);
            }
            for i in data {
                serial_print!("{:04x} ",i);
            }        
        }
        Ok(())
    }

    /// Write ATA DMA. Block size = 512 bytes.
    fn write(&self, sector: u64, count: usize, data: &[u32]) -> Result<(), ()> {
        assert_eq!(data.len(), count * SECTOR_SIZE);
        self.wait();
        unsafe {
            self.select(sector, count as u8);
            asm_out_u8(self.port_base + ATA_REGISTER_COMMAND, AtaCommands::WritePio as u8);
            for i in 0..count {
                let ptr = &data[(i as usize) * SECTOR_SIZE];
                if self.wait_error() {
                    return Err(());
                }
                asm_out_u32(self.port_base + ATA_REGISTER_DATA, ptr, SECTOR_SIZE);
            }
        }
        Ok(())
    }
}

pub fn ide_handler(ide_index : usize) {
}

pub fn print_u8_arrays(title : &str, string : *const u8, size : isize ) {
    serial_print!("{}",title);
    unsafe {
        for i in 0..size {
            serial_print!("{}",*(string.offset(i)) as char);
        }
    }
    serial_print!("\n");
}
