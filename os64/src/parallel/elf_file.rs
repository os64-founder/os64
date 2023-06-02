enum Architecture {
    X86_64,
    X86,
    ARM,
    // ...
}

#[repr(packed)] 
#[derive(Clone,Copy,Debug)]
pub struct ElfHeader {
    magic: [u8; 4],  // 必须是0x7fELF
    class: u8,       // 32/64位架构 
    endian: u8,      // 大小端 
    version: u8,     // ELF版本
    os_abi: u8,      // 操作系统和ABI 
    abi_version: u8,
    pad: [u8; 7],      
    file_type: u16,   // 文件类型
    machine: u16,     // 目标架构
    version2: u32,     // 版本号
    entry_point: u64, // 入口点地址
    program_header_offset: u64, // 程序头表偏移
    section_header_offset: u64, // 节头表偏移
    flags: u64,      // 标志
    header_size: u16, // 头部大小
    program_header_entry_size: u16, // 程序头条目大小
    program_header_num: u16,     // 程序头表项数 
    section_header_entry_size: u16, // 节头条目大小
    section_header_num: u16,     // 节头表项数
    section_header_string_index: u16, // 节头表字符串表索引

    // ... 更多字段
}

#[repr(packed)] 
#[derive(Clone,Copy,Debug)]
struct ProgramHeader {
    header_type: u32, // 条目类型
    offset: u64,      // 偏移 
    virtual_address: u64, // 虚拟地址
    physical_address: u64,  // 物理地址 
    file_size: u64,   // 文件大小
    memory_size: u64, // 内存大小
    flags: u64,       // 标志
    align: u64,       // 对齐
}

#[repr(packed)] 
#[derive(Clone,Copy,Debug)]
struct SectionHeader {
    name_index: u32,   // 节名字符串表索引
    header_type: u32,  // 条目类型 
    flags: u64,        // 标志
    virtual_address: u64, // 虚拟地址
    offset: u64,       // 文件偏移
    size: u64,         // 大小
    link: u32,         // 链接
    info: u32,         // 附加信息 
    align: u64,        // 对齐
    entry_size: u64    // 条目大小
}

#[repr(packed)] 
#[derive(Clone,Copy,Debug)]
struct SymbolTableEntry {
    name_index: u32,   // 符号名在字符串表中的索引
    info: u8,          // 符号类型和绑定属性
    other: u8,         // 未定义 
    section_index: u16,// 该符号所在节的索引 
    value: u64,        // 该符号的值 
    size: u64          // 该符号的大小
}

struct ElfFile {
    file: File,
    header: ElfHeader,
    program_headers: Vec<ProgramHeader>,
    section_headers: Vec<SectionHeader>,
    symbol_table: Vec<SymbolTableEntry>,
}

impl ElfFile {
    fn new(path: &str) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let header = parse_header(&file)?;
        let program_headers = parse_program_headers(&file, header.program_header_offset, header.program_header_entry_size, header.program_header_num)?;
        let section_headers = parse_section_headers(&file, header.section_header_offset, header.section_header_entry_size, header.section_header_num)?;
        let symbol_table = parse_symbol_table(&file, header.symbol_table_offset, header.symbol_entry_size, header.symbol_table_num)?;
        Ok(ElfFile {
            file, 
            header,
            program_headers,
            section_headers,
            symbol_table 
        })
    }
}

// 各个解析函数的实现...

fn parse_header(file: &File) -> std::io::Result<ElfHeader> {
    // 解析ELF头并返回
}