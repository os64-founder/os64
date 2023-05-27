
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

pub struct SuperBlock {
	root : *mut Directory,
	// info : *const!,
}

pub struct IndexNode {
	file_size : usize,
	blocks : usize,
	attribute : usize,
	super_block : *mut SuperBlock,
	// info : *const!,
}

// #define FS_ATTR_FILE	(1UL << 0)
// #define FS_ATTR_DIR	(1UL << 1)

pub struct Directory {
	name : [u8; MAX_PATH],
	name_length : usize,
	// child_node : List<Node>,
	// subdirs_list : List<Directory>,
	node : IndexNode,
	// parent : &Directory,
}

pub struct File {
	position : usize,
	mode : u64,
	dir : *mut Directory,
	// data : * const!,
}

pub trait File_System {
    
    // fn get_name() -> &'static str;
    // fn get_sign() -> u16;

    fn block_write(block : &SuperBlock);
    fn block_put(block : &SuperBlock);
    fn node_write(node :&IndexNode);
	fn node_create(node :&IndexNode, dir : &Directory, mode : u64) -> u64;
	// fn node_lookup(node :&Node, dir : &Directory) -> Directory;
    fn directory_make(node :&IndexNode, dir : &Directory, mode : u64) -> u64;
    fn directory_remove(node :&IndexNode, dir : &Directory) -> u64;
	fn directory_rename(old_node :&IndexNode, old_dir : &Directory, new_node : &IndexNode, new_dir : &Directory) -> u64;
	fn directory_get_attributes(dir : &Directory) -> Result<u64, &'static str>;
	fn directory_set_attributes(dir : &Directory, attributes : u64) -> Result<(), &'static str>;
	fn directory_compare(dir : &Directory, source_filename : &'static str, destination_filename : &'static str) -> Result<u64, &'static str>;
	fn directory_hash(dir : &Directory, filename : &'static str) -> Result<u64, &'static str>;
	fn directory_release(dir : &Directory) -> Result<u64, &'static str>;
	fn directory_iput(dir : &Directory, node : &IndexNode) -> Result<u64, &'static str>;
    fn file_open(file : &File, node : &IndexNode) -> Result<(), &'static str>;
    fn file_close(file : &File, node : &IndexNode) -> Result<(), &'static str>;
    fn file_read(file : &File, buffer : *mut u8, size : usize, position : usize) -> Result<u64, &'static str>;
    fn file_write(file : &File, buffer : *mut u8, size : usize, position : usize) -> Result<u64, &'static str>;
    fn file_seek(file : &File, offset : usize, origin : u8) -> Result<u64, &'static str>;
    fn io_control(file : &File, node : &IndexNode, command : u64, argment : u64) -> Result<u64, &'static str>;
}

pub fn mount<T : File_System>(directory : &'static str,  device : &'static str, file_system : &T ) -> Result<(),&'static str> {
    Ok(())
}

pub fn unmount<T : File_System>(directory : &'static str ) -> Result<(),&'static str> {
    Ok(())
}
