use std::thread;
use vm::VM;


#[derive(Debug)]
pub struct Scheduler {
    next_pid: u32,
    max_pid: u32,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            next_pid: 0,
            max_pid: 50000,
        }
    }

    /// Takes a VM and runs it in a background thread
    pub fn get_thread(&mut self, mut vm: VM) -> thread::JoinHandle<u32> {
        thread::spawn(move || {
            vm.run()
        })
    }
}


