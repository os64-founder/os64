use alloc::vec::Vec;

use super::task::{TaskId, Task};

pub struct Process {
    id : TaskId,
    task: Task,
    code: Vec<u8>,
}

impl Process {
    pub fn new(filename : &str) -> Self {
        todo!()

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