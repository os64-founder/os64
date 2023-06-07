use core::slice;
use alloc::{vec::Vec, rc::Rc, string::{ToString, String}, collections::BTreeMap};
use bitfield::size_of;
use crate::{device::disk::{ide::IDE_DISKS, disk::DiskDriver, disk::SECTOR_SIZE, fat::{Fat16BootSector, Attributes, FAT16SuperBlock}}, serial_println, parallel::modules::{DEFAULT_PAGE_SIZE, Elf64SymbolItem}, serial_print};
use super::{task::{TaskId, Task}, modules::{ModuleLoadedInfo, ModuleInfo, DEFAULT_STACK_ADDRESS, DEFAULT_STACK_SIZE}};
use xmas_elf::{ElfFile, sections::ShType, program::Type};

pub struct Process {
    id : TaskId,
    task: Task,
    code: Vec<u8>,
}

impl Process {
    pub fn read(filename : &str,physical_memory_offset :u64) {
        let mut pm = ProcessManager::new();
        pm.load(&filename.to_string(), physical_memory_offset);
        // let mut code = Vec::new();
        // File::open(path)?.read_to_end(&mut code)?;
        // Ok(Self { 
        //     task, 
        //     executor: Executor::new(),
        //     code 
        // })

        // let mut executor = Executor::new();
        // executor.spawn(task);
        // Self { task, executor }
    }

    pub fn load(elf_file : &ElfFile) {
    }

    pub fn run(&mut self) -> isize {
        todo!()
        // self.executor.spawn((self.code)(self.task));
        // self.executor.run()
    }
}

struct ProcessManager {
    pub modules : BTreeMap<String,ModuleLoadedInfo>,
}

impl ProcessManager {
    pub fn new() -> ProcessManager {
        ProcessManager {
            modules: BTreeMap::new(),
        }
    }

    pub fn read(&mut self, filename : &str) -> Vec<u8> {
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

    pub fn load(&mut self, filename : &String, physical_memory_offset :u64) {    
        if !self.modules.contains_key(filename) {
            let mut module = ModuleInfo::new(&filename,0,0);
            let mut module = ModuleLoadedInfo::new(module);
            let all_bytes = self.read(&filename);
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
                                let page_count = ((program_header.mem_size() as usize - 1) / DEFAULT_PAGE_SIZE) + 1;

                                //需要映射到进程的地址
                                let mut virtual_address = program_header.virtual_addr() as usize;
                                let mut offset = program_header.offset() as usize;
                                let mut copy_size = program_header.file_size() as usize;
                                serial_println!("virtual_address = 0x{:016x}, offset = {}, copy_size = {}", virtual_address, offset, copy_size);

                                for _ in 0..page_count {
                                    let temp = module.page_by_address(virtual_address, physical_memory_offset);
                                    let data = temp.get_data();
                                    //复制内容
                                    unsafe {
                                        let size = if copy_size > 4096 { 4096 } else { copy_size };
                                        for i in 0..size {
                                            (*data)[i] = all_bytes[offset + i];
                                        }
                                        copy_size -= size;
                                        offset += size;
                                    }
                                    virtual_address += DEFAULT_PAGE_SIZE;
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
            }

            // init stack pages
            let mut virtual_address = DEFAULT_STACK_ADDRESS;
            let mut stack_size  = 0;
            while stack_size < DEFAULT_STACK_SIZE {
                let _ = module.page_by_address(virtual_address, physical_memory_offset);
                virtual_address += DEFAULT_PAGE_SIZE;
                stack_size += DEFAULT_PAGE_SIZE;
            }
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

    pub fn run(&mut self, filename : &String) {
        //
    }

}