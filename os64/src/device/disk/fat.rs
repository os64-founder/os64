//see also: https://wiki.osdev.org/FAT
// 本文试图实现 FAT12/16/32,将来也考虑ExFAT及vFAT
// FAT：文件分配表(File Allocation Table)
// 链表结构：文件中的数据块，FAT 用 链表结构 表示
// 支持对磁盘空间的划分管理,如簇(Cluster)的概念
// FAT12: 常用于软盘。文件分配表单项12bit,簇大小0.5~4KB,分区最大容量小于16M
// FAT16: 用于早期硬盘。文件分配表单项16bit,簇大小最大64KB,分区最大容量小于 2G? 4G?
// FAT32: 文件分配表单项32bit(但保留最高4bit),簇大小最大2~32KB，单个文件最大2G

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

use core::{slice, cell::RefCell, borrow::Borrow};
use bitfield::size_of;
use bitflags::bitflags;
use alloc::{boxed::Box, rc::{Rc, Weak}, vec::Vec, string::String};
use crate::{serial_println, serial_print};
use super::{disk::{DiskDriver, SECTOR_SIZE}, file_system::{Time, Date, FileSystem, SuperBlock, IndexNode, Directory, File, DateTime, FileOpenMode}};

bitflags! { 
    ///目录项属性
    pub struct Attributes: u8 {
        const READ_ONLY = 0b0000_0001;
        const HIDDEN    = 0b0000_0010;
        const SYSTEM    = 0b0000_0100;
        const VOLUME_ID = 0b0000_1000;
        const DIRECTORY = 0b0001_0000;
        const ARCHIVE   = 0b0010_0000;
        const LONG_NAME = 0b0000_1111;
    }
}

/// FAT12/16 BootSector 完全一致，共62字节 + 448 + 2 =512字节
/// FAT12/16 不同之处在于 FAT 项是 12bit 还是 16 bit
/// 磁道数(也叫柱面数) = sectors / sectors_per_track / heads = 2880 / 18 / 2 = 80
#[repr(packed)] 
#[derive(Clone,Copy,Debug)]
pub struct Fat16BootSector {
    pub jmp_boot: [u8;3],       // 跳转指令
    pub oem_name: [u8;8],       // OEM名称,如 'OS64    '
    pub bytes_per_sector: u16,  // 每扇区字节数,512
    pub sectors_per_cluster: u8,// 每簇扇区数, 1
    pub reserved_sectors: u16,  // 保留扇区数, FAT12必须为1
    pub fats: u8,               // FAT表个数, 2
    pub root_entries: u16,      // 根目录项数, 224
    pub totel_sectors_u16: u16, // 总扇区数, 2880/1440...
    pub media: u8,              // 媒体描述符, 0xF0 0xF8
    pub sectors_per_fat: u16,   // FAT占用扇区数, 9
    pub sectors_per_track: u16, // 每磁道扇区数, 18
    pub heads: u16,             // 磁头数, 2
    pub hidden_sectors: u32,    // 隐藏扇区数, 0 
    pub totel_sectors: u32,     // 总扇区数(若sectors为0)

    pub drviver_number: u8,     // 驱动器号(用于int13中断), 0
    pub reserved1: u8,          // 0
    pub boot_sign: u8,          // 扩展引导标记, 0x29
    pub volume_id: u32,         // 卷序列号, 0
    pub volume_label: [u8;11],  // 卷标 'OS64    '
    pub file_system_type: [u8;8],//文件系统类型, 'FAT12   '

    pub boot_code: [u8;448],    // 引导代码
    pub magic: u16,             // 魔数,0xAA55
}

impl Fat16BootSector {
    pub fn get_totel_sectors(&self) -> usize {
        if self.totel_sectors_u16 == 0 {
            self.totel_sectors as usize
        } else {
            self.totel_sectors_u16 as usize
        }
    }

    //在我们的范例种：
    // 1个保留扇区,2*200个FAT扇区,32个根目录扇区；1、2簇号保留
    // 所以 (簇号-2)*4 + 433 即 簇号 * 4 + 425
    pub fn get_sector_index(&self, cluster_index : usize) -> usize {
        self.hidden_sectors as usize + self.reserved_sectors as usize 
         + self.fats as usize * self.sectors_per_fat as usize 
         + self.root_entries as usize * size_of::<Fat16DirectoryItem>() / self.bytes_per_sector as usize
         + (cluster_index - 2) * self.sectors_per_cluster as usize
    }
}

/// FAT32 的 BootSector 共90字节 + 420 + 2 =512字节
///
///   FAT32 磁盘扇区示意图
///   ____________  ____________  ____________  ____________  ____________ ____________ ____________ ____________ _______________ 
///  |            |             |             |             |             |            |            |            |               |
///  |            |             |             |    Backup   |             |            |            |            |               |
///  | BootSector |   FS_INF    |  Reserved   |  BootSector |   Reserved  |    FAT1    |    FAT2    |  Root Dir  |      Data     |
///  |            |             |             |             |             |            |            |            |               |
///  |____________| ____________| ____________| ____________| ____________|____________|____________|____________|_______________|
///   0            1             2             6             7             36           2086         4136         4144
#[repr(packed)]
#[derive(Clone,Copy,Debug)]
pub struct Fat32BootSector {
    pub jmp_boot: [u8;3],       // 跳转指令, 0x9058EB
    pub oem_name: [u8;8],       // OEM名称,如 'OS64    '
    pub bytes_per_sector: u16,  // 每扇区字节数, 512
    pub sectors_per_cluster: u8,// 每簇扇区数, 8
    pub reserved_sectors: u16,  // 保留扇区数, 36
    pub fats: u8,               // FAT表个数, 2
    pub root_entries: u16,      // 根目录项数, 0
    pub total_sectors_u16: u16, // 总扇区数, 0
    pub media: u8,              // 媒体描述符, 0xF8
    pub sectors_per_fat_u16: u16,// FAT占用扇区数, 0
    pub sectors_per_track: u16, // 每磁道扇区数, 63
    pub heads: u16,             // 磁头数, 255
    pub hidden_sectors: u32,    // 隐藏扇区数, 2048 
    pub totel_sectors: u32,     // 总扇区数(若sectors为0) 2103296

	pub sectors_per_fat: u32,   // FAT占用扇区数, 2050
    pub extended_flags: u16,    // 扩展标志, 0
    pub file_system_version: u16,// FAT32版本号, 0
    pub root_cluster : u32,     //根目录簇号, 2
	pub fs_info_sector : u16,   // fs_info 结构体扇区号, 1
    pub boot_sector_backup : u16,// 引导扇区备份扇区号, 6
	pub reserved : [u8;12],     //保留, 0

    pub drviver_number: u8,     // 驱动器号(用于int13中断), 0x80
    pub reserved1: u8,          // 0
    pub boot_sign: u8,          // 扩展引导标记, 0x29
    pub volume_id: u32,         // 卷序列号, 0x00004823
    pub volume_label: [u8;11],  // 卷标 'OS64    '
    pub file_system_type: [u8;8],//文件系统类型, 'FAT32   '

    pub boot_code : [u8;420],   // 引导代码
    pub magic : u16,            // 魔数,0xAA55
}

///目录项，32字节，每扇区可以存 512/32 = 16 项
#[repr(packed)]
#[derive(Clone,Copy,Debug)]
pub struct Fat16DirectoryItem {
    pub name : [u8;11],         //文件名 8+3 结构
    pub attributes : Attributes,//文件属性
    pub reserved : [u8;10],     //保留
    pub write_time : u16,       //最后修改时间
    pub write_date : u16,       //最后修改日期
    pub cluster_index : u16,    //起始簇号
    pub file_size : u32,        //文件大小
}

impl Fat16DirectoryItem {
    fn root() -> Fat16DirectoryItem {
        let mut name = [' ' as u8;11];
        name[0] = '/' as u8;
        Fat16DirectoryItem {
            name,
            attributes : Attributes::DIRECTORY,
            reserved : [0;10],
            write_time : Time(0,0,0).to_u16(),
            write_date : Date(2023,6,1).to_u16(),
            cluster_index : 0,
            file_size : 0,
        }
    }
}

/// 目录项，32字节，每扇区可以存 512/32 = 16 项
/// 和Fat16DirectoryItem不同之处在于： 10个保留字节已经被使用
#[repr(packed)]
#[derive(Clone,Copy,Debug)]
pub struct Fat32DirectoryItem
{
    pub name : [u8;11],         //文件名 8+3 结构
    pub attributes : Attributes,        //文件属性
	pub reserved : u8,          //保留
                                //EXT|BASE => 8(BASE).3(EXT)
                                //BASE:LowerCase(8),UpperCase(0)
                                //EXT:LowerCase(16),UpperCase(0)
	pub create_time_tenth : u8, //创建时间的毫秒级时间戳
	pub create_time : u16,	    //文件创建时间
	pub create_date : u16,      //文件创建日期
	pub last_access_date : u16, //最后访问日期
	pub cluster_index_high : u16,//起始簇号(高16bit)
    pub write_time : u16,       //最后修改时间
    pub write_date : u16,       //最后修改日期
    pub cluster_index : u16,    //起始簇号
    pub file_size : u32,        //文件大小
}

//为了快速找到空簇而设置的扇区，512字节
#[repr(packed)]
#[derive(Clone,Copy,Debug)]
struct Fat32_FSInfo
{
	pub lead_sign : u32,        //扇区标识符,固定为: 0x41615252
    pub reserved1 : [u8;480],   //保留
	pub struct_sign : u32,      //结构标识符,固定为: 0x61417272
	pub free_count : u32,       //上一次记录的空闲簇数量大概值,如果为 0xffffffff,则需重新计算
	pub next_free : u32,        //空闲簇起始搜索位置,如果为 0xffffffff,则从簇号2开始搜索
	pub reserved2 : [u8;12],    //保留
	pub trail_sign : u32,       //结束标识符,固定为: 0xaa550000
}

///长名字目录项，每项32字节
#[repr(packed)]
#[derive(Clone,Copy,Debug)]
pub struct FatDirectoryItemLongName
{
	pub order : u8,
	pub name1 :[u16; 5],
	pub attributes : u8,
	pub kind : u8,
	pub check_sum : u8,
	pub name2 : [u16; 6],
	pub first_cluster_low : u16, // 必须为 0
	pub name3 : [u16; 2],
}

//以下先基于FAT16实现文件系统各项功能，之后再考虑进行抽象

///(目前假定)
/// 在磁盘上 fat_count 份 FAT 是 连续存放 的，
/// 以 start_sector_index 为起始扇区，
/// 共占据 fat_count * sectors_per_fat 扇区。
pub struct FAT16Fats {
    pub data : Vec<u16>,
    pub start_sector_index : u64,
    pub fat_count : usize,
    pub sectors_per_fat : usize,
    pub bytes_per_sector : usize,
    pub total_clusters : usize,
}

pub const FAT16_EMPTY_CLUSTER   : u16 = 0x0000;
pub const FAT16_END_FLAG        : u16 = 0xFFF8;
pub const FAT16_BAD_CLUSTER     : u16 = 0xFFF7;
pub const FAT16_END_OF_FILE     : u16 = 0xFFFF;

impl FAT16Fats {
    pub fn new(driver : &Rc<dyn DiskDriver>, boot_sector : &Fat16BootSector) -> FAT16Fats {
        let fat_sectors = boot_sector.sectors_per_fat as usize ;
        let fat_bytes  = fat_sectors * boot_sector.bytes_per_sector as usize ;
        let fat_bits = fat_bytes * 8 / (boot_sector.get_totel_sectors() / boot_sector.sectors_per_cluster as usize);
        assert!(fat_bits==16);
        FAT16Fats {
            // driver,
            data: Vec::new(),
            start_sector_index : boot_sector.reserved_sectors as u64,
            fat_count : boot_sector.fats as usize,
            sectors_per_fat : boot_sector.sectors_per_fat as usize,
            bytes_per_sector : boot_sector.bytes_per_sector as usize,
            total_clusters : boot_sector.get_totel_sectors() / (boot_sector.sectors_per_cluster as usize),
        }
    }

    /// read all FATs from disk
    pub fn init(&mut self, driver : &Rc<dyn DiskDriver>) {
        let fats_sectors = self.fat_count * self.sectors_per_fat;
        let fats_bytes = fats_sectors * self.bytes_per_sector ;
        self.data = Vec::with_capacity(fats_bytes / 2);
        unsafe{self.data.set_len(fats_bytes / 2);}
        let data =  unsafe { slice::from_raw_parts_mut(self.data.as_ptr() as *mut u32, fats_bytes / 4) };
        let _ = driver.read(self.start_sector_index, fats_sectors, data);
        serial_println!("fat_start_sectors = {}, fats_sectors = {}", self.start_sector_index, fats_sectors);
    }

    /// write all FATs to disk
    pub fn flush(&self, driver : &Rc<dyn DiskDriver>) {
        let fats_sectors = self.fat_count * self.sectors_per_fat;
        let fats_bytes = fats_sectors * self.bytes_per_sector ;
        let data =  unsafe { slice::from_raw_parts_mut(self.data.as_ptr() as *mut u32, fats_bytes / 4) };
        let _ = driver.write(self.start_sector_index, fats_sectors, data);
    }

    /// 获取以index为起点的所有簇
    /// 如果“table_value”大于或等于 (>=) 0xFFF8，则链中不再有簇。这意味着整个文件已被读取。
    /// 如果“table_value”等于 (==) 0xFFF7，则此簇已标记为“坏”。“坏”簇容易出错，应该避免。
    /// 如果“table_value”不是上述情况之一，那么它是文件中下一个簇的簇号。
    /// 索引 0 和 1 下的条目是保留的。第零个条目是保留的，因为索引 0 用作其他条目的值，表示给定的集群是空闲的。
    /// 第零个条目必须保存低 8 位中 BPB_Media 字段的值，其余位必须设置为零。
    /// 例如，如果 BPB_Media 为 0xF8，则第零个条目的值应为 0xFFF8。
    /// 第一个条目是为将来保留的，必须保持值 0xFFFF。
    pub fn get_all_clusters(&self, index : u16) -> Vec<u16> {
        let mut ret = Vec::new();
        let mut index = index as usize;
        ret.push(index as u16);
        while self.data[index] < FAT16_END_FLAG {
            let temp = self.data[index];
            ret.push(temp);
            index = temp as usize;
        }
        ret
    }

    fn try_alloc_a_cluster(&mut self, all_clusters : &mut Vec<u16>) -> bool  {
        let last = all_clusters[all_clusters.len() - 1] as usize;
        // for i in last+1..self.total_clusters {
        //     if self.data[i] == FAT16_EMPTY_CLUSTER {
        //         all_clusters.push(self.data[i]);
        //     }
        // }
        true
    }

    ///为以index为起点的链，再申请count个簇
    pub fn alloc_clusters(&mut self, index : u16, count : u16) -> Vec<u16> {
        let mut ret = self.get_all_clusters(index);
        // for i in 0..count {
        //     self.try_alloc_a_cluster(&mut ret);
        // }
        ret
    }

    ///释放以index为起点的所有簇
    pub fn free_entries(&mut self, index : u16) {
        //
    }
}

/// 缓存整个BootSector
/// 缓存FAT：读一份，出错时读另一份；写双份
/// 获取根目录的节点
#[derive(Clone)]
pub struct FAT16SuperBlock {
    pub driver : Rc<dyn DiskDriver>,
    pub sector0 : Rc<Fat16BootSector>,
    /// in FAT, cluster0/1 is reserved
    /// so record cluster2 sector index
    pub cluster2_sector_index : usize,
    pub clusters_count : usize,
    pub fats : Rc<FAT16Fats>,
    pub root : Rc<FAT16Directory>,
}

impl FAT16SuperBlock {
    pub fn new(driver : Rc<dyn DiskDriver>, sector0 : Rc<Fat16BootSector>) -> FAT16SuperBlock {
        let cluster0_sector_index =  sector0.reserved_sectors as usize; 
        let clusters_count = sector0.get_totel_sectors() / (sector0.sectors_per_cluster as usize);
        let root = Rc::new(FAT16SuperBlock::ReadRoot(&driver, &sector0));
        let mut fats = FAT16Fats::new(&driver, &sector0);
        fats.init(&driver);
        let fats = Rc::new(fats);

        FAT16SuperBlock {
            driver,
            sector0,
            cluster2_sector_index: cluster0_sector_index,
            clusters_count,
            fats,
            root,
        }
    }

    ///读取根目录数据
    pub fn ReadRoot(driver : &Rc<dyn DiskDriver>, boot_sector : &Fat16BootSector) -> FAT16Directory {
        let root_sectors = (boot_sector.root_entries * 32 / boot_sector.bytes_per_sector) as usize;
        let root_bytes = root_sectors * boot_sector.bytes_per_sector as usize;

        let mut data: Vec<u8> = Vec::with_capacity(root_bytes);
        unsafe{data.set_len(root_bytes);}

        let mut root_buffer  : Vec<Fat16DirectoryItem> = Vec::with_capacity(boot_sector.root_entries as usize);
        unsafe{root_buffer.set_len(boot_sector.root_entries as usize);}
        let temp: &mut [u32] =  unsafe { slice::from_raw_parts_mut(data.as_mut_ptr() as *mut u32, root_bytes / 4) };
        let root_start_sectors = boot_sector.reserved_sectors  as u64 + boot_sector.fats as u64 * boot_sector.sectors_per_fat as u64;
        let _ = driver.read(root_start_sectors, root_sectors, temp);
        serial_println!("root_start_sectors = {}, root_sectors = {}", root_start_sectors, root_sectors);
        // for i in 0..128 {
        //     serial_print!("{:08x} ",data[i]);
        // }
        // serial_print!("");
        serial_println!("root_buffer.len() = {}", root_buffer.len());
        let mut root_item = Fat16DirectoryItem::root();

        FAT16Directory {
            children_data : Rc::new(data), 
            bytes : root_bytes, 
            parent : None, 
            data : root_item,
            clusters_index: Vec::new(),
        }
    }

}

impl SuperBlock for FAT16SuperBlock {
    fn write(&self) {
        self.fats.flush(&self.driver);
        // self.root.flush();
    }

    fn get_root(&self) -> Rc<dyn Directory> {
        self.root.clone()
    }
}

///
#[derive(Clone)]
pub struct FAT16IndexNode {
    ///父目录
    parent : Rc<FAT16Directory>,
    index : usize,
    longname_indexes : Vec<usize>,
}

impl FAT16IndexNode {
    fn new(parent : Rc<FAT16Directory>, index : usize, longname_indexex: Vec<usize>) -> FAT16IndexNode {
        FAT16IndexNode {
            parent,
            index,
            longname_indexes: longname_indexex,
        }
    }

    #[inline(always)]
    fn get_item(&self) -> Fat16DirectoryItem {
        self.parent.get_child_item(self.index)
    }
}

impl IndexNode for FAT16IndexNode {
    fn get_parent(&self) -> Rc<dyn Directory> {
        self.parent.clone()
    }

    fn get_size(&self) -> usize {
        self.get_item().file_size as usize
    }

    fn get_name(&self) -> String {
        u8_11_to_string(&self.get_item().name)
    }

    fn set_name(&self, name : &str, super_block : &Rc<dyn SuperBlock>) {
        todo!()
    }

    fn get_attribute(&self) -> u64 {
        self.get_item().attributes.bits as u64
    }

    fn set_attribute(&mut self, value : u64, super_block : &Rc<dyn SuperBlock>) {
        self.get_item().attributes.bits = value as u8;
    }

    ///这个是有Bug的,复制出来的是备份,无法完成时间设置的
    fn set_write_datetime(&self, value : DateTime, super_block : &Rc<dyn SuperBlock>) {
        let mut item = self.get_item();
        item.write_date = value.0.to_u16();
        item.write_time = value.1.to_u16();
    }

    fn get_write_datetime(&self) -> DateTime {
        let item = self.get_item();
        DateTime(Date::from(item.write_date), Time::from(item.write_time))
    }
}
#[derive(Clone)]
pub struct FAT16Directory {
    ///数据
    children_data : Rc<Vec<u8>>,
    // 字节数
    bytes : usize,
    ///父目录
    parent : Option<Rc<FAT16Directory>>,
    ///数据
    data : Fat16DirectoryItem,
    ///目录簇编号
    clusters_index : Vec<u16>,
}

impl FAT16Directory {
    pub fn get_parent(&self) -> Option<Rc<FAT16Directory>> {
        self.parent.clone()
    }

    pub fn get_data(&self) -> Fat16DirectoryItem {
        self.data
    }

    pub fn get_child_item(&self, index : usize) -> Fat16DirectoryItem {
        unsafe {
            let data = self.children_data.as_ptr().add(index * size_of::<Fat16DirectoryItem>()) as *mut Fat16DirectoryItem;
            *data
        }
    }

    pub fn get_child_longname(&self, index : usize) -> FatDirectoryItemLongName {
        unsafe {
            let data = self.children_data.as_ptr().add(index * size_of::<FatDirectoryItemLongName>()) as *mut FatDirectoryItemLongName;
            *data
        }
    }

    pub fn get_children_longname(&self, indexes : Vec<usize>) -> Vec<FatDirectoryItemLongName> {
        let mut ret : Vec<FatDirectoryItemLongName> = Vec::new();
        for i in indexes {
            ret.push(self.get_child_longname(i));
        }
        ret
    }

    pub fn find_children(&self, name : &[u8;11], attributes : Attributes) -> RefCell<Vec<Rc<FAT16IndexNode>>> {
        let mut ret = RefCell::new(Vec::new());
        let count = self.bytes / size_of::<Fat16DirectoryItem>();
        let mut longname: Vec<usize> = Vec::new();
        for i in 0..count {
            let item = self.get_child_item(i);
            if item.attributes.contains(Attributes::LONG_NAME) {
                longname.push(i);
            }
            else {
                longname.clear();
                if item.attributes.contains(attributes) {
                    if *name==item.name {
                        // let temp = Weak::new();
                        let node = Rc::new(FAT16IndexNode::new( Rc::new(self.clone()),i,longname));
                        ret.borrow_mut().push(node);
                        break
                    }
                }
            }
        }
        ret
    }

    pub fn open_file(&self, super_block : &Rc<FAT16SuperBlock>, index_node : Rc<FAT16IndexNode>) -> Result<Rc<FAT16File>,&'static str> {
        let item = self.get_child_item(index_node.index);
        let all_clusters = super_block.fats.get_all_clusters(item.cluster_index);
        let ret = Rc::new(FAT16File::new(Rc::new(self.clone()), index_node,RefCell::new(all_clusters)));
        Ok(ret)
    }
}

impl Directory for FAT16Directory {
    fn get_node(&self) -> Rc<dyn IndexNode> {
        todo!()
    }

    fn get_children(&self) -> Vec<Rc<dyn IndexNode>> {
        todo!()
    }

    fn get_files(&self) -> Vec<Rc<dyn IndexNode>> {
        todo!()
    }

    fn get_directories(&self) -> Vec<Rc<dyn IndexNode>> {
        todo!()
    }

    /// open the file
    fn open_file(&self, node : Rc<dyn IndexNode>, super_block : &Rc<dyn SuperBlock>) -> Rc<dyn File> {
        todo!()
    }

    /// get the sub directory
    fn load_directory(&self, node : Rc<dyn IndexNode>, super_block : &Rc<dyn SuperBlock>) -> Rc<dyn Directory> {
        todo!()
    }

    fn create_directory(&self, super_block : &Rc<dyn SuperBlock>) -> Rc<dyn Directory> {
        todo!()
    }

    fn delete_directory(&self, super_block : &Rc<dyn SuperBlock>) {
        todo!()
    }
}

pub struct FAT16File  {
    // pub driver : Rc<dyn DiskDriver>,
    pub path : Rc<FAT16Directory>,
    pub node : Rc<FAT16IndexNode>,
    pub indexes : RefCell<Vec<u16>>,
    pub pos : usize,
}

impl FAT16File {
    pub fn new(path : Rc<FAT16Directory>, node : Rc<FAT16IndexNode>, indexes : RefCell<Vec<u16>>) -> FAT16File {
        FAT16File { path, node, indexes, pos: 0 }
    }

    pub fn read_all_bytes(&self, super_block : &Rc<FAT16SuperBlock>) -> Box<Vec<u8>> {
        let item = self.path.get_child_item(self.node.index);
        let size = item.file_size as usize;
        let mut ret : Vec<u8> = Vec::with_capacity(size);
        unsafe{ret.set_len(size);}
        let mut data : [u32;SECTOR_SIZE*4] = [0;SECTOR_SIZE*4];
        let sector_index = super_block.sector0.get_sector_index(item.cluster_index as usize) as u64;
        let _ = super_block.driver.read(sector_index, super_block.sector0.sectors_per_cluster as usize, &mut data);
        let data = unsafe {*(data.as_mut_ptr() as *mut [u8;512*4])};
        for i in 0..size {
            ret[i] = data[i];
            serial_print!("{}", ret[i] as char);
        }
        Box::new(ret)
    }

    pub fn get_index_node() {
    }
}

impl File for FAT16File {
    fn get_node(&self) -> Rc<dyn IndexNode> {
        todo!()
    }

    fn get_mode(&self) -> FileOpenMode {
        todo!()
    }

    fn get_position(&self) {
        todo!()
    }

    fn set_position(&mut self) {
        todo!()
    }

    fn read(&self, super_block : &Rc<dyn SuperBlock>) {
        todo!()
    }
    // pub fn read(&self, buffer : *mut u8, size :usize) -> usize {
    //     let item = self.path.get_child_item(self.node.index);
    //     let mut size = size;
    //     if size + self.pos > item.file_size as usize {
    //         size = item.file_size as usize - self.pos;
    //     }        
    //     // serial_println!("FAT16File::read()");
    //     0
    // }

    fn write(&self, super_block : &Rc<dyn SuperBlock>) {
        todo!()
    }

    fn flush(&self, super_block : &Rc<dyn SuperBlock>) {
        todo!()
    }

    fn close(&self, super_block : &Rc<dyn SuperBlock>) {
        todo!()
    }
}

impl FileSystem for FAT16SuperBlock {
    fn super_block(driver : Rc<dyn DiskDriver>) -> Rc<dyn SuperBlock> {
        let mut data : [u32;SECTOR_SIZE] = [0;SECTOR_SIZE];
        //读取启动扇区
        driver.read(0, 1, &mut data);
        let boot_sector = unsafe {*(data.as_mut_ptr() as *mut Fat16BootSector)};
        Rc::new(FAT16SuperBlock::new(driver.clone(), Rc::new(boot_sector)))
    }
}

pub fn name_ext_to_u8_11(name : &str, ext : &str) -> [u8;11] {
    let mut ret : [u8;11] = [' ' as u8;11];
    let name = &name[0..8].to_uppercase();
    let ext  = &ext[0..3].to_uppercase();
    for (i, c) in name.chars().enumerate() {
        ret[i] = c as u8;
    }
    for (i, c) in ext.chars().enumerate() {
        ret[8+i] = c as u8;
    } 
    ret    
}

pub fn str_to_u8_11(v : &str) -> [u8;11] {
    let pos = v.find('.');
    match pos {
        None => {
            name_ext_to_u8_11(v, "")
        },
        Some(pos) => {
            if pos == v.len()-1 {
                name_ext_to_u8_11(&v[0..pos], "")
            } else {
                name_ext_to_u8_11(&v[0..pos], &v[pos+1..])
            }
        }     
    }
}

pub fn u8_11_to_string(v : &[u8;11]) -> String {
    let mut name = String::new();
    for i in 0..8 {
        let c = v[i] as char;
        if c==' ' {
            break;
        }
        name.push(c);
    }
    let mut ext = String::new(); 
    for i in 8..11 {
        let c = v[i] as char;
        if c==' ' {
            break;
        }
        ext.push(c);
    }
    if ext.len()>0 {
        name + "." + ext.as_str()
    } else {
        name
    }
}
