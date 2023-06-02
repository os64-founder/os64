use alloc::{vec::Vec, rc::Rc};
use crate::{device::disk::{fat::{FAT16SuperBlock, Fat16BootSector, Attributes}, ide::IDE_DISKS, disk::{SECTOR_SIZE, DiskDriver}}, serial_println};
use super::task::{TaskId, Task};
use xmas_elf::ElfFile;

pub struct Process {
    id : TaskId,
    task: Task,
    code: Vec<u8>,
}

impl Process {
    pub fn Read(filename : &str) {
        let driver = Rc::new(IDE_DISKS[1]);
        let mut data : [u32;SECTOR_SIZE] = [0;SECTOR_SIZE];
        //读取启动扇区
        driver.read(0, 1, &mut data);
        let boot_sector = unsafe {*(data.as_mut_ptr() as *mut Fat16BootSector)};
        let super_block = Rc::new(FAT16SuperBlock::new(driver.clone(), Rc::new(boot_sector)));

        let slice = b"FIRSTAPP   ";
        let result = super_block.root.find_children(slice, Attributes::empty());
        serial_println!("found {} app", result.borrow().len());

        let result = super_block.root.open_file(&super_block,result.borrow()[0].clone());
        let file = result.expect("can not open file");
        let all_bytes = file.read_all_bytes(&super_block);
        let elf = ElfFile::new(&all_bytes).expect("error elf format");
        
        for s in elf.section_iter() {
            //let r = s.get_name(&elf).expect("error in get section name");
            serial_println!("section: {:?}", s);
        }

        for p in elf.program_iter() {            
            // let d = p.get_data(&elf).expect("error in get data");
            serial_println!("program: {:?}", p);
        }
        
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

    pub fn run(&mut self) -> isize {
        todo!()
        // self.executor.spawn((self.code)(self.task));
        // self.executor.run()
    }
}