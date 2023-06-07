use x86_64::{structures::paging::{PageTable, PageTableIndex, PageTableFlags}, VirtAddr, PhysAddr};
use core::fmt;
use alloc::{string::{String, ToString}, collections::BTreeMap, boxed::Box};
use crate::{serial_println, memory::translate_addr};

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

/// 512G - 2M
pub const DEFAULT_STACK_ADDRESS : usize = 0o000_777_777_000_0000;
/// 64K
pub const DEFAULT_STACK_SIZE : usize = 0o000_000_000_020_0000;
/// 4K
pub const DEFAULT_PAGE_SIZE : usize = 0o000_000_000_001_0000;

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

    pub fn get_sub_key(&self, virtual_address : usize) -> Option<PageKey> {
        if self.level() == 0 {
            None
        } else {
            Some(PageKey::from_virtual_address(self.level() - 1, virtual_address))
        }
    }

    pub fn level(&self) -> u8 {
        (self.value >> 60) as u8 & 0x07
    }

    pub fn index(&self) -> u16 {
        let bits = 12 + 9 * self.level();
        (self.value >> bits) as u16 & 0x1FF
    }
 
    pub fn l4_index(&self) -> PageTableIndex {
        PageTableIndex::new((self.value >> 39) as u16 & 0x1FF)
    }

    pub fn l3_index(&self) -> PageTableIndex {
        PageTableIndex::new((self.value >> 30) as u16 & 0x1FF)
    }

    pub fn l2_index(&self) -> PageTableIndex {
        PageTableIndex::new((self.value >> 21) as u16 & 0x1FF)
    }

    pub fn l1_index(&self) -> PageTableIndex {
        PageTableIndex::new((self.value >> 12) as u16 & 0x1FF)
    }
}


///页信息
pub struct Page {
    ///数据: 页表 或 4K内存
    pub data : Box<PageTable>,
    pub key: PageKey,
    pub sub_pages : BTreeMap<u16,Box<Page>>,//[Option<Rc<Page>>;512],//
}

impl Page {
    fn new(key: &PageKey) -> Page {
        Page {
            data: Box::new(PageTable::new()),
            key: *key,
            sub_pages: BTreeMap::new(),
        }
    }

    pub fn page_table(&mut self) -> &mut Box<PageTable> {
        &mut self.data
    }

    pub fn get_data(&mut self) -> *mut [u8;4096] {
        self.data.as_ref() as *const PageTable as usize as * mut[u8;4096]
    }

    pub fn physical_address(&self, physical_memory_offset: u64) -> PhysAddr {
        let address = self.data.as_ref() as *const PageTable as u64;
        //这个 VirtAddr 是 init 进程中的
        let addr = VirtAddr::new(address);
        unsafe { translate_addr(addr,VirtAddr::new(physical_memory_offset)).expect("translate_addr error") }
    }

    pub fn sub_page(&mut self,virtual_address: usize,physical_memory_offset: u64) -> Option<&mut Box<Page>> {
        let key = self.key.get_sub_key(virtual_address);
        match key {
            None => None,
            Some(key) => {
                let index = key.index();
                if !self.sub_pages.contains_key(&index) {
                    let page = Box::new(Page::new(&key));
                    let phys_addr = page.physical_address(physical_memory_offset);
                    serial_println!("key = {:?}, page address = 0x{:016x}", key, phys_addr.as_u64());
                    self.data[PageTableIndex::new(index)].set_addr(phys_addr, PageTableFlags::empty());
                    self.sub_pages.insert(index, page);
                }
                self.sub_pages.get_mut(&index)
            },
        }
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
    pub level4 : Page,
    pub symbols : BTreeMap<String,ModuleSymbol>,
    use_count : usize,
}

impl ModuleLoadedInfo {
    pub fn new(info : ModuleInfo) -> ModuleLoadedInfo {
        let l4_key = PageKey::from_virtual_address(4, 0);
        ModuleLoadedInfo {
            info,
            level4: Page::new(&l4_key),
            symbols: BTreeMap::new(),
            use_count: 0,
        }
    }

    pub fn page_by_address(&mut self, virtual_address: usize, physical_memory_offset: u64) -> &mut Box<Page> {
        let l3 = self.level4.sub_page(virtual_address, physical_memory_offset).unwrap();
        let l2 = l3.sub_page(virtual_address, physical_memory_offset).unwrap();
        let l1 = l2.sub_page(virtual_address, physical_memory_offset).unwrap();
        l1.sub_page(virtual_address, physical_memory_offset).unwrap()
    }
}
