use core::slice;
use alloc::{vec::Vec, rc::Rc, string::ToString};
use bitfield::size_of;
// use bitflags::bitflags;
use crate::{device::disk::{fat::{FAT16SuperBlock, Fat16BootSector, Attributes}, ide::IDE_DISKS, disk::{SECTOR_SIZE, DiskDriver}}, serial_println, serial_print, parallel::modules::Elf64SymbolItem};
use super::{task::{TaskId, Task}, modules::ModuleLoadedInfo};
use xmas_elf::{ElfFile, sections::{ShType}};

pub struct Process {
    id : TaskId,
    task: Task,
    code: Vec<u8>,
}

impl Process {
    pub fn read(filename : &str,physical_memory_offset :u64) {
        ModuleLoadedInfo::load(&filename.to_string(), physical_memory_offset);
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