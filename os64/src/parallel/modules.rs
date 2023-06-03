use core::fmt;
use alloc::{rc::Rc, string::String, boxed::Box, collections::BTreeMap, vec::Vec};
use xmas_elf::{ElfFile, sections::{ShType}};

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
    pub fn get_bingding(&self) -> SymbolBinding {
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
        write!(f, "SymbolItem: {{ name: {}, info: {}-{} ,", name, self.get_bingding(), self.get_type()); 
        write!(f, " other: {},", self.other);
        write!(f, " section: {},", SymbolSection::from(self.section));
        // write!(f, " address: {:016x},", address);
        write!(f, " address: {},", address);
        write!(f, " size: {} }}", size)
    }
}

pub struct ModuleSymbol {
    pub name: String,
    pub address: usize,
    pub size: usize,
    pub binding : SymbolBinding,
    pub kind : SymbolKind,
}


pub struct ModuleVirtualMemory {
    pub start: usize,
    pub size: usize,
}

pub struct ModuleInfo {
    pub name: String,
    pub version: u32,
    pub api_version: u32,
    pub symbols: Vec<String>,
}

pub struct ModuleLoadedInfo {
    pub info: ModuleInfo,
    pub memory: ModuleVirtualMemory,
}

pub struct ModuleManager {
    pub modules: Vec<Box<ModuleLoadedInfo>>,
}
