
///磁盘分区表入口, 16字节
#[repr(packed)]
pub struct DiskPartitionTableEntry {
	flags : u8,
	start_head : u8,
	start_sector_cylinder:u16,//0~5bit: sector; 6~15bit: cylinder
	kind : u8,
	end_head : u8,
	end_sector_cylinder	:u16,//0~5bit: sector; 6~15bit: cylinder
	start_lba : u32,
	sectors_limit : u32,
}

///磁盘分区表, 512 字节
#[repr(packed)]
pub struct DiskPartitionTable {
	reserved : [u8; 446],
	parts : [DiskPartitionTableEntry; 4],
	trail_sign : u16,
}

pub const MAX_LENGTH_FOR_FILE_SYSTEM_TYPE_NAME : usize = 60;
pub const MAX_PATH : usize = 256;

pub struct Block {
	root : *mut Directory,
	// info : *const!,
}

pub struct Node {
	file_size : usize,
	blocks : usize,
	attribute : usize,
	super_block : *mut Block,
	// info : *const!,
}

// #define FS_ATTR_FILE	(1UL << 0)
// #define FS_ATTR_DIR		(1UL << 1)

pub struct Directory {
	name : [u8; MAX_PATH],
	name_length : usize,
	// child_node : List<Node>,
	// subdirs_list : List<Directory>,
	node : Node,
	// parent : &Directory,
}

pub struct File {
	position : usize,
	mode : u64,
	dir : *mut Directory,
	// data : * const!,
}

pub trait File_System {
    
    fn get_name() -> &'static str;
    fn get_sign() -> u16;

    fn block_write(block : &Block);
    fn block_put(block : &Block);
    fn node_write(node :&Node);
	fn node_create(node :&Node, dir : &Directory, mode : u64) -> u64;
	// fn node_lookup(node :&Node, dir : &Directory) -> Directory;
    fn directory_make(node :&Node, dir : &Directory, mode : u64) -> u64;
    fn directory_remove(node :&Node, dir : &Directory) -> u64;
	fn directory_rename(old_node :&Node, old_dir : &Directory, new_node : &Node, new_dir : &Directory) -> u64;
	fn directory_get_attributes(dir : &Directory) -> Result<u64, &'static str>;
	fn directory_set_attributes(dir : &Directory, attributes : u64) -> Result<(), &'static str>;
	fn directory_compare(dir : &Directory, source_filename : &'static str, destination_filename : &'static str) -> Result<u64, &'static str>;
	fn directory_hash(dir : &Directory, filename : &'static str) -> Result<u64, &'static str>;
	fn directory_release(dir : &Directory) -> Result<u64, &'static str>;
	fn directory_iput(dir : &Directory, node : &Node) -> Result<u64, &'static str>;
    fn file_open(file : &File, node : &Node) -> Result<(), &'static str>;
    fn file_close(file : &File, node : &Node) -> Result<(), &'static str>;
    fn file_read(file : &File, buffer : *mut u8, size : usize, position : usize) -> Result<u64, &'static str>;
    fn file_write(file : &File, buffer : *mut u8, size : usize, position : usize) -> Result<u64, &'static str>;
    fn file_seek(file : &File, offset : usize, origin : u8);
    fn io_control(file : &File, node : &Node, command : u64, argment : u64) -> Result<u64, &'static str>;
}

// struct super_block * mount_fs(char * name,struct Disk_Partition_Table_Entry * DPTE,void * buf);
// unsigned long register_filesystem(struct file_system_type * fs);
// unsigned long unregister_filesystem(struct file_system_type * fs);
