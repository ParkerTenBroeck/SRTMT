use std::collections::HashMap;

use crate::thread::{Thread, ThreadError, ThreadMemory, ThreadRunResult};

#[derive(Default)]
pub struct System {
    threads: SystemThreads,
    core: SystemCore,
}

impl System {
    pub fn run(&mut self) {
        let mut shit_bool = false;
        let mut mem = ThreadMemory::new(&mut shit_bool);
        while !self.threads.thread_pool.is_empty() {
            let len = self.threads.thread_pool.len();
            for id in 0..len {
                let iterations = 5000;
                println!("Starting thread: {} for {} iterations", id, iterations);
                let res = self.run_thread(id, &mut mem, iterations);
                //println!("{:#?}", self.threads.thread_pool[id]);
                match res {
                    Ok(ok) => match ok {
                        ThreadRunResult::Continue => {}
                        ThreadRunResult::Wait(_actually_ran) => {}
                        ThreadRunResult::Exit(_actually_ran) => {}
                    },
                    Err(err) => {
                        print!(
                            "Thread: {} encountered an error: {:#?}\nDUMP: {:#?}\nTerminating",
                            id, self.threads.thread_pool[id], err
                        );
                        self.threads.thread_pool.remove(id);
                        break;
                    }
                }
            }
        }
    }

    pub fn add_thread<const SIZE: usize>(
        &mut self,
        initial_pages: [u16; SIZE],
    ) -> [&mut Page; SIZE] {
        let mut thread = Thread::default();

        for page in initial_pages {
            thread
                .memory_mapping
                .mapping
                .push((self.threads.sys_mem.v_mem.len(), page));
            self.threads.sys_mem.v_mem.push([0; 0x10000]);
        }

        let mut t = Vec::new();

        let mut start_page_id = 0usize;
        let mut pages = self.threads.sys_mem.v_mem.as_mut_slice();

        for (page_id, _) in &thread.memory_mapping.mapping {
            pages = &mut pages[(page_id - start_page_id)..];
            match std::mem::take(&mut pages) {
                [] => panic!(
                    "Page mapping doesnt exist: page_id: {} for thread: {}",
                    page_id,
                    self.threads.thread_pool.len()
                ),
                [first, rest @ ..] => {
                    pages = rest;
                    // this is the same deal as the ll_bit thing we unset this to none before we return
                    // so no lifetime is outlived
                    t.push(first);
                }
            }
            start_page_id = *page_id + 1;
        }

        self.threads.thread_pool.push(thread);

        t.try_into().unwrap()
    }

    fn run_thread<'c>(
        &mut self,
        id: usize,
        mem: &mut ThreadMemory<'c>,
        iters: u32,
    ) -> Result<ThreadRunResult, ThreadError> {
        let thread = self.threads.thread_pool.get_mut(id).unwrap();

        // this is sketchy but we know that we un-set this later in the function before we return
        // so we can (hopfully) do this even if its kinda really bad
        let ll_bit = &mut self.threads.sys_mem.ll_bit;
        let mut ll_bit = unsafe { std::mem::transmute(ll_bit) };
        std::mem::swap(&mut ll_bit, &mut mem.ll_bit);

        let mut start_page_id = 0usize;
        let mut pages = self.threads.sys_mem.v_mem.as_mut_slice();

        for (page_id, v_addr) in &thread.memory_mapping.mapping {
            pages = &mut pages[(page_id - start_page_id)..];
            match std::mem::take(&mut pages) {
                [] => panic!(
                    "Page mapping doesnt exist: page_id: {} for thread: {}",
                    page_id, id
                ),
                [first, rest @ ..] => {
                    pages = rest;
                    // this is the same deal as the ll_bit thing we unset this to none before we return
                    // so no lifetime is outlived
                    mem.mem[*v_addr as usize] = Some(unsafe { std::mem::transmute(first) });
                }
            }
            start_page_id = *page_id + 1;
        }
        self.core.current_thread_id = id;
        let res = thread.run(&mut self.core, mem, iters);

        for (_, v_addr) in &thread.memory_mapping.mapping {
            mem.mem[*v_addr as usize] = None;
        }
        std::mem::swap(&mut ll_bit, &mut mem.ll_bit);

        res
    }

    pub fn remove_page(&mut self, to_remove_page_id: usize) {
        self.threads.sys_mem.v_mem.remove(to_remove_page_id);
        for thread in &mut self.threads.thread_pool {
            for (page_id, _) in &mut thread.memory_mapping.mapping {
                if *page_id >= to_remove_page_id {
                    *page_id -= 1;
                }
            }
        }
    }
}

#[derive(Default)]
pub struct SystemThreads {
    sys_mem: SystemMemory,
    thread_pool: Vec<Thread>,
}

#[derive(Default)]
pub struct SystemCore {
    current_thread_id: usize,
    partial_process_output: HashMap<usize, String>,
}

impl SystemCore {
    pub fn system_call(
        &mut self,
        id: u32,
        thread: &mut Thread,
        mem: &mut ThreadMemory<'_>,
    ) -> InterfaceCallResult {
        match id {
            0 => return InterfaceCallResult::ImmediateKill(None),
            1 => {
                println!(
                    "Thread: {} -> {}",
                    self.current_thread_id, thread.vm_state.reg[4] as i32
                );
            }
            4 => {
                let mut address = thread.vm_state.reg[4];
                let mut str: Vec<u8> = Vec::new();
                loop {
                    match mem.mem.get_mut(address as usize >> 16).unwrap() {
                        Some(page) => {
                            let char = page[address as u16 as usize];
                            if char == 0 {
                                break;
                            }
                            str.push(char);
                            address += 1;
                        }
                        None => {
                            return InterfaceCallResult::ImmediateKill(Some(
                                ThreadError::MemoryDoesNotExistError(address, thread.vm_state.pc),
                            ))
                        }
                    }
                }

                let str = String::from_utf8(str);
                if let Ok(str) = str {
                    println!("Thread: {} -> {}", self.current_thread_id, str);
                } else {
                    return InterfaceCallResult::MalformedCallArgs;
                }
            }

            5 => {
                let char = thread.vm_state.reg[4] as u8 as char;
                if char != '\n' {
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        self.partial_process_output.entry(self.current_thread_id)
                    {
                        e.insert(char.into());
                    } else {
                        self.partial_process_output
                            .get_mut(&self.current_thread_id)
                            .unwrap()
                            .push(char);
                    }
                } else if let Some(msg) =
                    self.partial_process_output.remove(&self.current_thread_id)
                {
                    println!("Thread: {} -> {}", self.current_thread_id, msg);
                } else {
                    println!("Thread: {} -> ", self.current_thread_id);
                }
            }
            _ => return InterfaceCallResult::InvalidCall(id),
        }
        InterfaceCallResult::Continue
    }

    pub fn breakpoint(
        &mut self,
        id: u32,
        _thread: &mut Thread,
        _mem: &mut ThreadMemory<'_>,
    ) -> InterfaceCallResult {
        match id {
            534 => {}
            _ => return InterfaceCallResult::InvalidCall(id),
        }
        InterfaceCallResult::Continue
    }
}

pub enum InterfaceCallResult {
    Continue,
    ImmediateKill(Option<ThreadError>),
    MalformedCallArgs,
    InvalidCall(u32),
    Wait,
}

#[derive(Default)]
pub struct SystemMemory {
    v_mem: Vec<Page>,
    ll_bit: bool,
}

pub type Page = [u8; 0x10000];
pub type PageId = usize;
