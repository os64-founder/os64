use bitfield::size_of;
use lazy_static::lazy_static;
use x86_64::{structures::paging::PageTable, VirtAddr, PhysAddr};
use core::{fmt, borrow::BorrowMut};
use alloc::{string::{String, ToString}, collections::BTreeMap, vec::Vec, rc::Rc, slice};
use xmas_elf::{ElfFile, sections::ShType, program::Type};
use crate::{serial_println, device::disk::{ide::IDE_DISKS, disk::SECTOR_SIZE, disk::DiskDriver, fat::{Fat16BootSector, FAT16SuperBlock, Attributes}}, serial_print, memory::translate_addr};

// enum PT
// {
//   PT_NULL = 0,
//   PT_LOAD = 1,
//   PT_DYNAMIC = 2,
//   PT_INTERP = 3,
//   PT_NOTE = 4,
//   PT_SHLIB = 5,
//   PT_PHDR = 6,
//   PT_TLS = 7,
//   PT_LOOS = 0x60000000,
//   PT_HIOS = 0x6fffffff,
//   PT_LOPROC = 0x70000000,
//   PT_HIPROC = 0x7fffffff,
//   // The remaining values are not in the standard.
//   // Frame unwind information.
//   PT_GNU_EH_FRAME = 0x6474e550,
//   PT_SUNW_EH_FRAME = 0x6474e550,
//   // Stack flags.
//   PT_GNU_STACK = 0x6474e551,
//   // Read only after relocation.
//   PT_GNU_RELRO = 0x6474e552,
//   // Platform architecture compatibility information
//   PT_ARM_ARCHEXT = 0x70000000,
//   // Exception unwind tables
//   PT_ARM_EXIDX = 0x70000001
// }

///符号绑定信息（高4位）
#[repr(u8)]
#[derive(Clone,Copy,Debug)]
pub enum SymbolBinding {
    Local   = 0x0,
    Global  = 0x1,
    Weak    = 0x2,
}

impl From<u8> for SymbolBinding {
    fn from(n: u8) -> Self {
        match n {
            0x0 => SymbolBinding::Local,
            0x1 => SymbolBinding::Global,
            0x2 => SymbolBinding::Weak,
            _ => unreachable!() 
        }
    }
}

impl Into<u8> for SymbolBinding {
    fn into(self) -> u8 {
        match self {
            SymbolBinding::Local => 0x0,
            SymbolBinding::Global => 0x1,
            SymbolBinding::Weak => 0x2
        }
    } 
}

impl fmt::Display for SymbolBinding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SymbolBinding::Local => write!(f, "Local"),
            SymbolBinding::Global => write!(f, "Global"),
            SymbolBinding::Weak => write!(f, "Weak"),
        }
    }
}

///符号类型信息（低4位）
#[repr(u8)]
#[derive(Clone,Copy,Debug)]
pub enum SymbolKind {
    Unknown     = 0,
    Object      = 1,
    Function    = 2,
    Section     = 3,
    File        = 4,
}

impl From<u8> for SymbolKind {
    fn from(n: u8) -> Self {
        match n {
            0x0 => SymbolKind::Unknown,
            0x1 => SymbolKind::Object,
            0x2 => SymbolKind::Function,
            0x3 => SymbolKind::Section,
            0x4 => SymbolKind::File,
            _ => unreachable!() 
        }
    }
}

impl Into<u8> for SymbolKind {
    fn into(self) -> u8 {
        match self {
            SymbolKind::Unknown => 0x0,
            SymbolKind::Object => 0x1,
            SymbolKind::Function => 0x2,
            SymbolKind::Section => 0x3,
            SymbolKind::File => 0x4
        }
    }
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SymbolKind::Unknown => write!(f, "Unknown"),
            SymbolKind::Object => write!(f, "Object"),
            SymbolKind::Function => write!(f, "Function"),
            SymbolKind::Section => write!(f, "Section"),
            SymbolKind::File => write!(f, "File"),
        }
    }
}

///段信息特殊值
#[derive(Clone,Copy,Debug)]
#[repr(u16)]
pub enum SymbolSection {
    Undefined   = 0x0,
    Absolute    = 0xFFF1,
    Common      = 0xFFF2,
    Value(u16),
}

impl SymbolSection {
    fn from(n: u16) -> SymbolSection {
        match n {
            0x0 => SymbolSection::Undefined,
            0xFFF1 => SymbolSection::Absolute,
            0xFFF2 => SymbolSection::Common,
            _ => SymbolSection::Value(n)
        }
    }

    fn into(v : SymbolSection) -> u16 {
        match v {
            SymbolSection::Undefined => 0x0,
            SymbolSection::Absolute => 0xFFF1,
            SymbolSection::Common => 0xFFF2,
            SymbolSection::Value(u) => u,
        }
    }
}

impl fmt::Display for SymbolSection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SymbolSection::Undefined => write!(f, "Undefined"),
            SymbolSection::Absolute => write!(f, "Absolute"),
            SymbolSection::Common => write!(f, "Common"),
            Self::Value(v) => write!(f, "{}", v),
        }
    }
}

/// A item in section .symtab
/// 符号表项,即 .symtab 中的项
#[repr(packed)]
#[derive(Clone,Copy)]
pub struct Elf64SymbolItem {
    name : u32,
    info : u8,
    other : u8,
    section : u16,
    address : u64,
    size : u64,
}

impl Elf64SymbolItem {
    pub fn get_binding(&self) -> SymbolBinding {
        SymbolBinding::from((self.info >> 4) & 0xF)
    }

    pub fn get_type(&self) -> SymbolKind {
        SymbolKind::from(self.info & 0xF)
    }

    pub fn get_section(&self) -> SymbolSection {
        SymbolSection::from(self.section)
    }
}

impl fmt::Debug for Elf64SymbolItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = self.name;
        let address = self.address;
        let size = self.size;
        write!(f, "SymbolItem: {{ name: {}, info: {}-{} ,", name, self.get_binding(), self.get_type()); 
        write!(f, " other: {},", self.other);
        write!(f, " section: {},", SymbolSection::from(self.section));
        // write!(f, " address: {:016x},", address);
        write!(f, " address: {},", address);
        write!(f, " size: {} }}", size)
    }
}

///目前只保存 Global-Funciton 类型的Function
pub struct ModuleSymbol {
    pub name: String,
    pub address: usize,
    pub size: usize,
}

#[repr(align(8))]
#[repr(C)]
#[derive(Clone,Copy,PartialEq,PartialOrd,Eq,Ord)]
pub struct PageKey {
    value : usize,
}

impl fmt::Debug for PageKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PageKey: {{ value: 0x{:016x} }}", self.value)
    }
}

impl PageKey {
    pub fn from(level : u8, l4_index : u16, l3_index : u16, l2_index : u16, l1_index : u16) -> PageKey {
        assert!(level<=4);
        assert!(l4_index<512);
        assert!(l3_index<512);
        assert!(l2_index<512);
        assert!(l1_index<512);
        let level_bits = (level as usize) << 60;
        let l4_index = (l4_index as usize) << 39;
        let l3_index = (l3_index as usize) << 30;
        let l2_index = (l2_index as usize) << 21;
        let l1_index = (l1_index as usize) << 12;
        PageKey {
            value : match level {
                4 => {  level_bits },
                3 => {  level_bits | l4_index },
                2 => {  level_bits | l4_index | l3_index },
                1 => {  level_bits | l4_index | l3_index | l2_index },
                0 => {  level_bits | l4_index | l3_index | l2_index | l1_index},
                _ => {0}
            }
        }
    }

    pub fn from_virtual_address(level : u8, virtual_address : usize) -> PageKey {
        assert!(level<=4);
        let level_bits = (level as usize) << 60;
        let virtual_address = virtual_address & 0x0000_FFFF_FFFF_F000;
        PageKey {
            value : match level {
                4 => level_bits | 0,
                3 => level_bits | virtual_address & 0o777_000_000_000_0000,
                2 => level_bits | virtual_address & 0o777_777_000_000_0000,
                1 => level_bits | virtual_address & 0o777_777_777_000_0000,
                0 => level_bits | virtual_address & 0o777_777_777_777_0000,
                _ => 0
            }
        }
    }

    pub fn level(&self) -> u8 {
        (self.value >> 60) as u8 & 0x07
    }

    pub fn l4_index(&self) -> u16 {
        (self.value >> 39) as u16 & 0x1FF
    }

    pub fn l3_index(&self) -> u16 {
        (self.value >> 30) as u16 & 0x1FF
    }

    pub fn l2_index(&self) -> u16 {
        (self.value >> 21) as u16 & 0x1FF
    }

    pub fn l1_index(&self) -> u16 {
        (self.value >> 12) as u16 & 0x1FF
    }
}


///页信息
#[repr(C)]
pub struct Page {
    ///数据: 页表 或 4K内存
    pub data : Rc<PageTable>,
    
    ///页物理地址
    pub physical_address: PhysAddr,

    ///    
    pub key: PageKey,
}

impl Page {
    fn new(key: &PageKey, physical_memory_offset: u64) -> Page {
        let mut data = Rc::new(PageTable::new());
        let address = Rc::as_ptr(data.borrow_mut()) as u64;
        //这个 VirtAddr 是 init 进程中的
        let addr = VirtAddr::new(address);

        let physical_address = unsafe { translate_addr(addr,VirtAddr::new(physical_memory_offset)).expect("translate_addr error") };
        Page {
            data,
            physical_address,
            key: *key,
        }
    }

    pub fn page_table(&mut self) -> &mut Rc<PageTable> {
        &mut self.data
    }

    pub fn get_data(&mut self) -> *mut [u8;4096] {
        Rc::as_ptr(&mut self.data) as usize as * mut[u8;4096]
    }
}

///模块基本信息
pub struct ModuleInfo {
    pub name: String,
    pub version: u32,
    pub api_version: u32,
}

impl ModuleInfo {
    pub fn new(name : &str, version : u32, api_version : u32) -> ModuleInfo {
        ModuleInfo {
            name : name.to_string(),
            version,
            api_version,
        }
    }
}

///已加载模块
pub struct ModuleLoadedInfo {
    pub info: ModuleInfo,
    pub pages: BTreeMap<PageKey,Page>,
    pub symbols : BTreeMap<String,ModuleSymbol>,
    use_count : usize,
}

impl ModuleLoadedInfo {
    pub fn new(info : ModuleInfo) -> ModuleLoadedInfo {
        ModuleLoadedInfo {
            info,
            pages: BTreeMap::new(),
            symbols: BTreeMap::new(),
            use_count: 0,
        }
    }

    pub fn page(&mut self, key: &PageKey, physical_memory_offset: u64) -> &mut Page {
        if !self.pages.contains_key(key) {
            let page = Page::new(key,physical_memory_offset);
            self.pages.insert(*key,page);
        }
        self.pages.get_mut(key).unwrap()
    }

    pub fn page_by_address(&mut self, virtual_address: usize, physical_memory_offset: u64)  -> &mut Page {
        let l4_key = PageKey::from_virtual_address(4, virtual_address);
        let _ = self.page(&l4_key, physical_memory_offset);

        let l3_key = PageKey::from_virtual_address(3, virtual_address);
        let _ = self.page(&l3_key,physical_memory_offset);
        
        let l2_key = PageKey::from_virtual_address(2, virtual_address);
        let _ = self.page(&l2_key,physical_memory_offset);                                    

        let l1_key = PageKey::from_virtual_address(1, virtual_address);
        let _ = self.page(&l1_key,physical_memory_offset);

        let l0_key = PageKey::from_virtual_address(0, virtual_address);
        self.page(&l0_key,physical_memory_offset)
    }
}

// pub static ALLMODULES : BTreeMap<String,ModuleLoadedInfo> = BTreeMap::new();

impl  ModuleLoadedInfo {
    pub fn read(filename : &str) -> Vec<u8> {
        let driver = Rc::new(IDE_DISKS[1]);
        let mut data : [u32;SECTOR_SIZE] = [0;SECTOR_SIZE];
        //读取启动扇区
        let _ = driver.read(0, 1, &mut data);
        let boot_sector = unsafe {*(data.as_mut_ptr() as *mut Fat16BootSector)};
        let super_block = Rc::new(FAT16SuperBlock::new(driver.clone(), Rc::new(boot_sector)));

        let slice = b"FIRSTAPP   ";
        let result = super_block.root.find_children(slice, Attributes::empty());
        serial_println!("found {} app", result.borrow().len());

        let result = super_block.root.open_file(&super_block,result.borrow()[0].clone());
        let file = result.expect("can not open file");
        file.read_all_bytes(&super_block)
    }

    pub fn load(filename : &String,physical_memory_offset :u64) {
        
        // if !ALLMODULES.contains_key(&filename) {
            let mut module = ModuleInfo::new(&filename,0,0);
            let mut module = ModuleLoadedInfo::new(module);
            let all_bytes = ModuleLoadedInfo::read(&filename);
            let elf_file = ElfFile::new(&all_bytes).expect("elf format error");
            
            for program_header  in elf_file.program_iter() {
                match program_header.get_type() {
                    Ok(t) => {
                        match t {
                            Type::Phdr => {
                                // serial_println!("Phdr: ...");
                            },
                            Type::Load => {
                                // 需要进行加载 文件 offset 开始的 file_size 字节到 虚拟地址 virtual_addr,
                                // 总字节数 mem_size, 对齐字节数为 align, 页读写跑属性为 flag
                                let page_4k_count = ((program_header.mem_size() as usize - 1) >> 12) + 1;
                                // let page_4k_size = page_4k_count << 12;

                                //需要映射到进程的地址
                                let mut virtual_address = program_header.virtual_addr() as usize;
                                let mut offset = program_header.offset() as usize;
                                let mut copy_size = program_header.file_size() as usize;
                                serial_println!("virtual_address = 0x{:016x}, offset = {}, copy_size = {}", virtual_address, offset, copy_size);

                                for _ in 0..page_4k_count {
                                    let data = module.page_by_address(virtual_address, physical_memory_offset).get_data();
                                    //复制内容
                                    unsafe {
                                        let size = if copy_size > 4096 { 4096 } else { copy_size };
                                        for i in 0..size {
                                            (*data)[i] = all_bytes[offset + i];
                                        }
                                        copy_size -= size;
                                        offset += size;
                                    }
                                    virtual_address += 4096;
                                }
                            },
                            Type::OsSpecific(v) => {
                                serial_println!("OsSpecific: v = 0x{:08x}", v);
                            },
                            // Type::Dynamic => {},
                            // Type::Interp => {},
                            // Type::Note => {},
                            // Type::ShLib => {},
                            // Type::Tls => {},
                            // Type::GnuRelro => {},
                            // Type::ProcessorSpecific(_) => {},
                            _ => {},
                        }
                    },
                    Err(_) => {
                        serial_println!("Error");
                    }
                }
                // pub type_: Type_,
                // pub flags: Flags,
                // pub offset: u64,
                // pub virtual_addr: u64,
                // pub physical_addr: u64,
                // pub file_size: u64,
                // pub mem_size: u64,
                // pub align: u64,
            // }
        }
    }

    pub fn print(elf_file : &ElfFile) {
        serial_println!("header: {:?} ", elf_file.header);
        for s in elf_file.section_iter() {
            match s.get_name(&elf_file) {
                Ok(name) => {
                    serial_println!("section {}: {:?} ", name, s);
                    match s.get_type() {
                        Ok(t) => { 
                            match t {
                                ShType::ProgBits => { //代码段：如 .text 和 .comment 以及 .debug_*

                                },
                                ShType::StrTab => { //字符串表：如 .shstrtab 和 .strtab
                                    // .shstrtab 用于存储段名
                                    // .strtab 用于存储函数名
                                    let data = s.raw_data(elf_file);
                                    for (i,byte) in data.iter().enumerate() {
                                        if *byte == 0 {
                                            serial_println!();
                                            if i+1 != data.len() {
                                                serial_print!("{}:\t", i+1);
                                            }
                                        } else {
                                            serial_print!("{}",*byte as char);
                                        }
                                    }
                                },
                                ShType::SymTab => { //符号表： 如 .symtab
                                    let sizeof = size_of::<Elf64SymbolItem>();
                                    let data = s.raw_data(elf_file);
                                    // for (i,byte) in data.iter().enumerate() {
                                    //     serial_print!("{:02x}",byte);
                                    //     if (i + 1)  % 8 == 0 {
                                    //         serial_print!(" ");
                                    //         if (i + 1) % sizeof == 0 {
                                    //             serial_println!();
                                    //         }
                                    //     }
                                    // }
                                    // serial_println!();
                                    let items = unsafe { 
                                        slice::from_raw_parts(
                                            data.as_ptr() as *const Elf64SymbolItem, 
                                            data.len() / sizeof,
                                        ) 
                                    };
                                    for item in items {
                                        // let item = unsafe { *(data.as_mut_ptr() as *mut [Elf64SymbolItem]) };
                                        serial_println!("{:?}", item);
                                    }
                                },
                                ShType::Null => {

                                },
                                _ => {},
                            }
                        },
                        Err(e) => {},
                    }
                },
                Err(_) => {
                    serial_println!("section: {:?}", s);
                },
            }
        }

        for p in elf_file.program_iter() {
            serial_println!("program: {:?}", p);
        }
    }

}