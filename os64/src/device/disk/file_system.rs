// 本文试图抽象一个文件系统类
use alloc::{boxed::Box, rc::Rc, vec::Vec, string::String};
use super::disk::DiskDriver;
use bitfield::size_of;
use bitflags::bitflags;

/// 0  空              24  NEC DOS         81  Minix / 旧 Linu bf  Solaris        
/// 1  FAT12           27  Hidden NTFS Win 82  Linux 交换 / So c1  DRDOS/sec (FAT-
/// 2  XENIX root      39  Plan 9          83  Linux           c4  DRDOS/sec (FAT-
/// 3  XENIX usr       3c  PartitionMagic  84  OS/2 hidden or  c6  DRDOS/sec (FAT-
/// 4  FAT16 <32M      40  Venix 80286     85  Linux 扩展      c7  Syrinx         
/// 5  扩展            41  PPC PReP Boot   86  NTFS 卷集       da  非文件系统数据 
/// 6  FAT16           42  SFS             87  NTFS 卷集       db  CP/M / CTOS / .
/// 7  HPFS/NTFS/exFAT 4d  QNX4.x          88  Linux 纯文本    de  Dell 工具      
/// 8  AIX             4e  QNX4.x 第2部分  8e  Linux LVM       df  BootIt         
/// 9  AIX 可启动      4f  QNX4.x 第3部分  93  Amoeba          e1  DOS 访问       
/// a  OS/2 启动管理器 50  OnTrack DM      94  Amoeba BBT      e3  DOS R/O        
/// b  W95 FAT32       51  OnTrack DM6 Aux 9f  BSD/OS          e4  SpeedStor      
/// c  W95 FAT32 (LBA) 52  CP/M            a0  IBM Thinkpad 休 ea  Rufus alignment
/// e  W95 FAT16 (LBA) 53  OnTrack DM6 Aux a5  FreeBSD         eb  BeOS fs        
/// f  W95 扩展 (LBA)  54  OnTrackDM6      a6  OpenBSD         ee  GPT            
/// 10  OPUS            55  EZ-Drive        a7  NeXTSTEP        ef  EFI (FAT-12/16/
/// 11  隐藏的 FAT12    56  Golden Bow      a8  Darwin UFS      f0  Linux/PA-RISC  
/// 12  Compaq 诊断     5c  Priam Edisk     a9  NetBSD          f1  SpeedStor      
/// 14  隐藏的 FAT16 <3 61  SpeedStor       ab  Darwin 启动     f4  SpeedStor      
/// 16  隐藏的 FAT16    63  GNU HURD or Sys af  HFS / HFS+      f2  DOS 次要       
/// 17  隐藏的 HPFS/NTF 64  Novell Netware  b7  BSDI fs         fb  VMware VMFS    
/// 18  AST 智能睡眠    65  Novell Netware  b8  BSDI swap       fc  VMware VMKCORE 
/// 1b  隐藏的 W95 FAT3 70  DiskSecure 多启 bb  Boot Wizard 隐  fd  Linux raid 自动
/// 1c  隐藏的 W95 FAT3 75  PC/IX           bc  Acronis FAT32 L fe  LANstep        
/// 1e  隐藏的 W95 FAT1 80  旧 Minix        be  Solaris 启动    ff  BBT           
pub enum FileSystemKind {
    Empty       = 0x00,
    Fat12       = 0x01,
    Fat16_V1    = 0x04,
    Fat16_V2    = 0x06,
    Fat32_V1    = 0x0B,
    Fat32       = 0x0C,
    Fat16       = 0x0A,
}

pub const MAX_LENGTH_FOR_FILE_SYSTEM_TYPE_NAME : usize = 60;
pub const MAX_PATH : usize = 256;

#[derive(Clone,Copy,Debug)]
pub struct Date(pub u16,pub u8,pub u8);
#[derive(Clone,Copy,Debug)]
pub struct Time(pub u8,pub u8,pub u8);

#[derive(Clone,Copy,Debug)]
pub struct DateTime(pub Date,pub Time);

impl Date {
    pub fn to_u16(&self) -> u16 {
        ((self.0 - 1980) << 9) | ((self.1 as u16 & 0xF) << 5) | (self.2 as u16 & 0x1F)
    }
}

impl From<u16> for Date {
    fn from(value: u16) -> Self {
        let years = (value >> 9) + 1980;
        let months = (value >> 5)  as u8 & 0xF;
        let days = value as u8 & 0x1F;
        Date(years, months, days)
    }
}

impl Time {
    pub fn to_u16(&self) -> u16 {
        ((self.0 as u16) << 11) | ((self.1 as u16 & 0x3F) << 5) | ((self.2 as u16 / 2) & 0x1F)
    } 
}

impl From<u16> for Time {
    fn from(value: u16) -> Self {
        let hours = (value >> 11) as u8;
        let minutes = (value >> 5) as u8 & 0x3F; 
        let seconds = (value & 0x1F) as u8 * 2;
        Time(hours, minutes, seconds)
    }
}


bitflags! {
    pub struct FileOpenMode : u8 {
        //打开已存在的文件
        const   OPEN    = 0x01;
        //创建新的文件
        const   CREATE  = 0x02;
        //以只读方式打开文件（文件指针在文件头）
        const   READ    = 0x04;
        //以写入方式打开文件（如文件已存在，首先会清空文件，文件指针在文件头）
        const   WRITE   = 0x08;
        //以追加方式打开文件（无论文件是否存在，文件指针在文件尾）
        const   APPEND  = 0x04;
    }
}

// 以下是文件系统需要实现的部分：

pub trait SuperBlock {
    fn write(&self);
    fn get_root(&self) -> Rc<dyn Directory>;
}

pub trait IndexNode {
    fn get_parent(&self) -> Rc<dyn Directory>;
    fn get_size(&self) -> usize;

    fn get_name(&self) -> String;
    fn set_name(&self, name : &str, super_block : &Rc<dyn SuperBlock>);

    fn get_attribute(&self) -> u64;
    fn set_attribute(&mut self, value : u64, super_block : &Rc<dyn SuperBlock>);
    
    fn set_write_datetime(&self, value : DateTime, super_block : &Rc<dyn SuperBlock>);
    fn get_write_datetime(&self) -> DateTime;
}

///已经加载或创建的目录
pub trait Directory {
    fn get_node(&self) -> Rc<dyn IndexNode>;

    /// get sub dirs and files
	fn get_children(&self) -> Vec<Rc<dyn IndexNode>>;

    /// get files
	fn get_files(&self) -> Vec<Rc<dyn IndexNode>>;

    /// get directories
	fn get_directories(&self) -> Vec<Rc<dyn IndexNode>>;

    /// open the file
    fn open_file(&self, node : Rc<dyn IndexNode>, super_block : &Rc<dyn SuperBlock>) -> Rc<dyn File>;

    /// get the sub directory
    fn load_directory(&self, node : Rc<dyn IndexNode>, super_block : &Rc<dyn SuperBlock>) -> Rc<dyn Directory>;

    /// create directory
    fn create_directory(&self, super_block : &Rc<dyn SuperBlock>) -> Rc<dyn Directory>;

    /// delete directory
    fn delete_directory(&self, super_block : &Rc<dyn SuperBlock>);
}

///已经打开或创建的文件
pub trait File {
    fn get_node(&self) -> Rc<dyn IndexNode>;

    fn get_mode(&self) -> FileOpenMode;

    fn get_position(&self);
    fn set_position(&mut self);

    fn read(&self, super_block : &Rc<dyn SuperBlock>);
    fn write(&self, super_block : &Rc<dyn SuperBlock>);

    fn flush(&self, super_block : &Rc<dyn SuperBlock>);
    fn close(&self, super_block : &Rc<dyn SuperBlock>);
}

pub trait FileSystem {
    fn super_block(driver : Rc<dyn DiskDriver>) -> Rc<dyn SuperBlock>;
}

// pub fn mount<T : FileSystem>(directory : &'static str,  device : &'static str, file_system : &T ) -> Result<(),&'static str> {
//     Ok(())
// }

// pub fn unmount<T : FileSystem>(directory : &'static str ) -> Result<(),&'static str> {
//     Ok(())
// }