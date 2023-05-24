// 本文试图实现 FAT12/16/32 ，以下词：
// FAT：文件分配表(File Allocation Table)
// 链表结构：文件中的数据块，FAT 用 链表结构 表示
// 支持对磁盘空间的划分管理,如簇(Cluster)的概念
// FAT12:常用于软盘，720K / 1.44M / 2.88M ； FAT16:<2GB； FAT32:<2TB （未考究）
// 文件大小上限不同: FAT12:<4MB, FAT16:<2GB, FAT32:<4GB。（未考究）
// 簇大小不同: FAT12:1KB、2KB, FAT16:16KB~64KB, FAT32:4KB~32KB。（未考究）
// 文件分配表大小不同: FAT12:12比特, FAT16:16比特, FAT32:32比特。
// FAT12:早期DOS, FAT16:DOS/Windows, FAT32:Windows98及以上。

//
//  (FAT12) 1.44M 软盘扇区示意图
//   ____________  ____________  ____________  ____________  ______________________________________
//  |            |             |             |             |                                       |
//  |            |             |             |             |                                       |
//  | BootSector |    FAT1     |    FAT2     |  Root Dir   |                Data                   |
//  |            |             |             |             |                                       |
//  |____________| ____________| ____________| ____________| ______________________________________|
//   0            1           9 10         18 19         32 33                                 2879
//
//  FAT12 中每簇 占用 12bit, 即 1.5byte,则 2880簇 需要 2880*1.5=4320 bytes = 8.4375 扇区,因此 FAT 占用 9 扇区。
//  Root Dir 占用 14个扇区，每扇区可存 16 个目录项， 得 224 个
//  FAT中: 最开始项：0xFF0 为磁盘标识字， 低8位须和 Media 一致，其余位必须位1
//      0xFFF 表示已被占用，0x000 表示可用， 0x002 - 0xFEF (已被占用)表示文件的下一个簇号
//      0xFF0 - 0xFF6 保留  0xFF7 坏簇   0xFF8 - 0xFFF 文件的最后一个簇
//  

use bitflags::bitflags;
use super::file_system::*;

bitflags! { 
    ///目录项属性
    struct Attributes: u8 {
        const READ_ONLY = 0b0000_0001;
        const HIDDEN    = 0b0000_0010;
        const SYSTEM    = 0b0000_0100;
        const VOLUNE_ID = 0b0000_1000;
        const DIRECTORY = 0b0001_0000;
        const ARCHIVE   = 0b0010_0000;
        const LONG_NAME = 0b0000_1111;
    }
}

/// FAT12/16 BootSectorHeader 完全一致，共62字节
/// FAT12/16 不同之处在于 FAT 项是 12bit 还是 16 bit
/// 磁道数(也叫柱面数) = sectors / sectors_per_track / heads = 2880 / 18 / 2 = 80
#[repr(packed)] 
pub struct Fat16BootSectorHeader {
    jmp_boot: [u8;3],       // 跳转指令
    oem_name: [u8;8],       // OEM名称,如 'OS64    '
    bytes_per_sector: u16,  // 每扇区字节数,512
    sectors_per_cluster: u8,// 每簇扇区数, 1
    reserved_sectors: u16,  // 保留扇区数, FAT12必须为1
    fats: u8,               // FAT表个数, 2
    root_entries: u16,      // 根目录项数, 224
    sectors: u16,           // 总扇区数, 2880/1440...
    media: u8,              // 媒体描述符, 0xF0
    sectors_per_fat: u16,   // FAT占用扇区数, 9
    sectors_per_track: u16, // 每磁道扇区数, 18
    heads: u16,             // 磁头数, 2
    hidden_sectoss: u32,    // 隐藏扇区数, 0 
    totel_sectors: u32,     // 总扇区数(若sectors为0)
    drviver_number: u8,     // 驱动器号(用于int13中断), 0
    reserved1: u8,          // 0
    boot_sign: u8,          // 扩展引导标记, 0x29
    volume_id: u32,         // 卷序列号, 0
    volume_label: [u8;11],  // 卷标 'OS64    '
    file_system_type: [u8;8],//文件系统类型, 'FAT12   '
}

///共62+448+2=512字节
#[repr(packed)]
pub struct Fat16BootSector {
    header: Fat16BootSectorHeader,
    boot_code: [u8;448],    // 引导代码
    magic: [u8;2],          // 魔数,0xAA55
}

/// FAT32 的 BootSectorHeader 共90字节
///
///   FAT32 磁盘扇区示意图
///   ____________  ____________  ____________  ____________  ____________ ____________ ____________ ____________ _______________ 
///  |            |             |             |             |             |            |            |            |               |
///  |            |             |             |    Backup   |             |            |            |            |               |
///  | BootSector |   FS_INF    |  Reserved   |  BootSector |   Reserved  |    FAT1    |    FAT2    |  Root Dir  |      Data     |
///  |            |             |             |             |             |            |            |            |               |
///  |____________| ____________| ____________| ____________| ____________|____________|____________|____________|_______________|
///   0            1             2             6             7             36           2086         4136         4144
struct Fat32BootSectorHeader {
    jmp_boot: [u8;3],       // 跳转指令, 0x9058EB
    oem_name: [u8;8],       // OEM名称,如 'OS64    '
    bytes_per_sector: u16,  // 每扇区字节数, 512
    sectors_per_cluster: u8,// 每簇扇区数, 8
    reserved_sectors: u16,  // 保留扇区数, 36
    fats: u8,               // FAT表个数, 2
    root_entries: u16,      // 根目录项数, 0
    total_sectors_u16: u16, // 总扇区数, 0
    media: u8,              // 媒体描述符, 0xF8
    sectors_per_fat_u16: u16,// FAT占用扇区数, 0
    sectors_per_track: u16, // 每磁道扇区数, 63
    heads: u16,             // 磁头数, 255
    hidden_sectors: u32,    // 隐藏扇区数, 2048 
    totel_sectors: u32,     // 总扇区数(若sectors为0) 2103296

	sectors_per_fat: u32,   // FAT占用扇区数, 2050
    extended_flags: u16,    // 扩展标志, 0
    file_system_version: u16,// FAT32版本号, 0
    root_cluster : u32,     //根目录簇号, 2
	fs_info_sector : u16,   // fs_info 结构体扇区号, 1
    boot_sector_backup : u16,// 引导扇区备份扇区号, 6
	reserved : [u8;12],     //保留, 0

    drviver_number: u8,     // 驱动器号(用于int13中断), 0x80
    reserved1: u8,          // 0
    boot_sign: u8,          // 扩展引导标记, 0x29
    volume_id: u32,         // 卷序列号, 0x00004823
    volume_label: [u8;11],  // 卷标 'OS64    '
    file_system_type: [u8;8],//文件系统类型, 'FAT32   '
}

///共90+420+2=512字节
#[repr(packed)]
pub struct Fat32BootSector {
    header: Fat32BootSectorHeader,//
    boot_code: [u8;420],    // 引导代码
    magic: [u8;2],          // 魔数,0xAA55
}


///目录项，32字节，每扇区可以存 512/32 = 16 项
#[repr(packed)]
pub struct Fat16DirectoryItem {
    name : [u8;11],         //文件名 8+3 结构
    attributes : u8,        //文件属性
    reserved : [u8;10],     //保留
    write_time : u16,       //最后修改时间
    write_date : u16,       //最后修改日期
    cluster_index : u16,    //起始簇号
    file_size : u32,        //文件大小
}

/// 目录项，32字节，每扇区可以存 512/32 = 16 项
/// 和Fat16DirectoryItem不同之处在于： 10个保留字节已经被使用
#[repr(packed)]
pub struct Fat32DirectoryItem
{
    name : [u8;11],         //文件名 8+3 结构
    attributes : u8,        //文件属性
	reserved : u8,          //保留
                            //EXT|BASE => 8(BASE).3(EXT)
                            //BASE:LowerCase(8),UpperCase(0)
                            //EXT:LowerCase(16),UpperCase(0)
	create_time_tenth : u8, //创建时间的毫秒级时间戳
	create_time : u16,	    //文件创建时间
	create_date : u16,      //文件创建日期
	last_access_date : u16, //最后访问日期
	cluster_index_hight : u16,//起始簇号(高16bit)
    write_time : u16,       //最后修改时间
    write_date : u16,       //最后修改日期
    cluster_index : u16,    //起始簇号
    file_size : u32,        //文件大小
}

//为了快速找到空簇而设置的扇区，512字节
#[repr(packed)]
struct Fat32_FSInfo
{
	lead_sign : u32,        //扇区标识符,固定为: 0x41615252
    reserved1 : [u8;480],   //保留
	struct_sign : u32,      //结构标识符,固定为: 0x61417272
	free_count : u32,       //上一次记录的空闲簇数量大概值,如果为 0xffffffff,则需重新计算
	next_free : u32,        //空闲簇起始搜索位置,如果为 0xffffffff,则从簇号2开始搜索
	reserved2 : [u8;12],    //保留
	trail_sign : u32,       //结束标识符,固定为: 0xaa550000
}

///长目录项，每项32字节
#[repr(packed)]
pub struct Fat32DirectoryLongItem
{
	order : u8,
	name1 :[u16; 5],
	attributes : u8,
	kind : u8,
	check_sum : u8,
	name2 : [u16; 6],
	first_cluster_low : u16, // 必须为 0
	name3 : [u16; 2],
}

pub struct Date(u16,u8,u8);
pub struct Time(u8,u8,u8);

impl Date {
    pub fn to_u16(&self) -> u16 {
        ((self.0 - 1980) << 9) | ((self.1 as u16 & 0xF) << 4) | (self.2 as u16 & 0x1F)
    }
}

impl From<u16> for Date {
    fn from(value: u16) -> Self {
        let years = (value >> 9) + 1980;
        let months = (value >> 4)  as u8 & 0xF;
        let days = value as u8 & 0x1F;
        Date(years, months, days)
    }
}

impl Time {
    pub fn to_u16(&self) -> u16 {
        ((self.0 as u16) << 10) | ((self.1 as u16 & 0x3F) << 4) | ((self.2 as u16 / 2) & 0x1F)
    } 
}

impl From<u16> for Time {
    fn from(value: u16) -> Self {
        let hours = (value >> 10) as u8;
        let minutes = (value >> 4) as u8 & 0x3F; 
        let seconds = (value & 0x1F) as u8 * 2;
        Time(hours, minutes, seconds)
    }
}

// check_sum

// #define LOWERCASE_BASE (8)
// #define LOWERCASE_EXT (16)
// void DISK1_FAT32_FS_init();
// unsigned int DISK1_FAT32_read_FAT_Entry(unsigned int fat_entry);
// unsigned long DISK1_FAT32_write_FAT_Entry(unsigned int fat_entry,unsigned int value);

pub struct Fat32;

impl File_System for Fat32 {
    fn get_name() -> &'static str { return "FAT32"; }

    fn get_sign() -> u16 {
        //FAT32
        return 0x400B;
        //FAT16
        // return 0x4006;
    }

    fn block_write(block : &Block) {
        todo!()
    }

    fn block_put(block : &Block) {
        todo!()
    }

    fn node_write(node :&Node) {
        todo!()
    }

    fn node_create(node :&Node, dir : &Directory, mode : u64) -> u64 {
        todo!()
    }

    fn directory_make(node :&Node, dir : &Directory, mode : u64) -> u64 {
        todo!()
    }

    fn directory_remove(node :&Node, dir : &Directory) -> u64 {
        todo!()
    }

    fn directory_rename(old_node :&Node, old_dir : &Directory, new_node : &Node, new_dir : &Directory) -> u64 {
        todo!()
    }

    fn directory_get_attributes(dir : &Directory) -> Result<u64, &'static str> {
        todo!()
    }

    fn directory_set_attributes(dir : &Directory, attributes : u64) -> Result<(), &'static str> {
        todo!()
    }

    fn directory_compare(dir : &Directory, source_filename : &'static str, destination_filename : &'static str) -> Result<u64, &'static str> {
        todo!()
    }

    fn directory_hash(dir : &Directory, filename : &'static str) -> Result<u64, &'static str> {
        todo!()
    }

    fn directory_release(dir : &Directory) -> Result<u64, &'static str> {
        todo!()
    }

    fn directory_iput(dir : &Directory, node : &Node) -> Result<u64, &'static str> {
        todo!()
    }

    fn file_open(file : &File, node : &Node) -> Result<(), &'static str> {
        todo!()
    }

    fn file_close(file : &File, node : &Node) -> Result<(), &'static str> {
        todo!()
    }

    fn file_read(file : &File, buffer : *mut u8, size : usize, position : usize) -> Result<u64, &'static str> {
        todo!()
    }

    fn file_write(file : &File, buffer : *mut u8, size : usize, position : usize) -> Result<u64, &'static str> {
        todo!()
    }

    fn file_seek(file : &File, offset : usize, origin : u8) {
        todo!()
    }

    fn io_control(file : &File, node : &Node, command : u64, argment : u64) -> Result<u64, &'static str> {
        todo!()
    }

}