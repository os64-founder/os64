use core::slice;
use alloc::{vec::Vec, rc::Rc};
use bitfield::size_of;
// use bitflags::bitflags;
use crate::{device::disk::{fat::{FAT16SuperBlock, Fat16BootSector, Attributes}, ide::IDE_DISKS, disk::{SECTOR_SIZE, DiskDriver}}, serial_println, serial_print, parallel::modules::Elf64SymbolItem};
use super::task::{TaskId, Task};
use xmas_elf::{ElfFile, sections::{ShType}};

pub struct Process {
    id : TaskId,
    task: Task,
    code: Vec<u8>,
}

impl Process {
    pub fn read(filename : &str) {
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
        let all_bytes = file.read_all_bytes(&super_block);
        let elf_file = ElfFile::new(&all_bytes).expect("error elf format");
        Process::load(&elf_file);

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

    pub fn run(&mut self) -> isize {
        todo!()
        // self.executor.spawn((self.code)(self.task));
        // self.executor.run()
    }
}