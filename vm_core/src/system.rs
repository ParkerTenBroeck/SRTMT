use std::collections::HashMap;

use crate::thread::{Thread, ThreadError, ThreadId, ThreadMemory, ThreadRunResult};

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
            for thread_index in 0..len {
                let iterations = 1;
                //println!("Starting thread: {} for {} iterations", self.threads.thread_pool[thread_index].id(), iterations);
                let res = self.run_thread(thread_index, &mut mem, iterations);
                //println!("{:#?}", self.threads.thread_pool[id]);
                
                
                if let Some(new_thread_info) = self.core.create_new_thread.take() {
    
                    let mut new_thread = Thread::new(new_thread_info.new_id);
    
                    let shared_core_and_data = self
                        .threads
                        .get_thread(new_thread_info.creator_id)
                        .memory_mapping
                        .mapping
                        .first()
                        .unwrap()
                        .0;
                    new_thread
                        .memory_mapping
                        .mapping
                        .push((shared_core_and_data, 0x0)); //first data page shared between two threads

                    // self.threads.sys_mem.v_mem.push(self.threads.sys_mem.v_mem[shared_core_and_data]); //personal stack memory for thread
                    new_thread
                        .memory_mapping
                        .mapping
                        .push((self.threads.sys_mem.v_mem.len(), 0x7FFF));
                    self.threads.sys_mem.v_mem.push([0; 0x10000]); //personal stack memory for thread
    
                    new_thread.vm_state.pc = new_thread_info.start_addr; //start of function
                    new_thread.vm_state.reg[4] = new_thread_info.argument_ptr; //ptr to arguments in memory
                    new_thread.vm_state.reg[29] = 0x80000000; //start of stack
                    new_thread.vm_state.reg[31] = 0xFFFFFFFF;
                    println!(
                        "Created new thread: {}\nDUMP{{:?}}",
                        new_thread.id(),
                        //new_thread
                    );
                    self.threads.thread_pool.push(new_thread);
                }
                
                
                
                
                
                
                match res {
                    Ok(ok) => match ok {
                        ThreadRunResult::Continue => {}
                        ThreadRunResult::Wait(_actually_ran) => {}
                        ThreadRunResult::Exit(_actually_ran, code) => {
                            println!(
                                "Thread: {} exited with code: {}",
                                self.threads.thread_pool[thread_index].id(),
                                code
                            );
                            self.threads.thread_pool.remove(thread_index);
                            break;
                        }
                    },
                    Err(err) => {
                        println!(
                            "Thread: {} encountered an error: {:#?}\nDUMP: {:#?}\nTerminating",
                            self.threads.thread_pool[thread_index].id(),
                            err,
                            self.threads.thread_pool[thread_index]
                        );
                        self.threads.thread_pool.remove(thread_index);
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
        let mut thread = Thread::new(self.core.next_thread_id());

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
                    thread.id()
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
        index: usize,
        mem: &mut ThreadMemory<'c>,
        iters: u32,
    ) -> Result<ThreadRunResult, ThreadError> {
        let thread = self.threads.thread_pool.get_mut(index).unwrap();

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
                    page_id,
                    thread.id()
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

impl SystemThreads {
    fn get_thread(&self, id: ThreadId) -> &Thread {
        self.thread_pool.iter().find(|t| t.id() == id).unwrap()
    }
}

#[derive(Default)]
pub struct SystemCore {
    partial_process_output: HashMap<ThreadId, String>,
    create_new_thread: Option<ThreadCreationInfo>,
    next_thread_id: u32,
}

pub struct ThreadCreationInfo {
    start_addr: u32,
    argument_ptr: u32,
    new_id: ThreadId,
    creator_id: ThreadId,
}

impl SystemCore {
    pub fn system_call(
        &mut self,
        id: u32,
        thread: &mut Thread,
        mem: &mut ThreadMemory<'_>,
    ) -> InterfaceCallResult {
        match id {
            0 => return InterfaceCallResult::Exit,
            1 => {
                println!(
                    "Thread: {} -> {}",
                    thread.id(),
                    thread.vm_state.reg[4] as i32
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
                    println!("Thread: {} -> {}", thread.id(), str);
                } else {
                    return InterfaceCallResult::MalformedCallArgs;
                }
            }

            5 => {
                let char = thread.vm_state.reg[4] as u8 as char;
                if char != '\n' {
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        self.partial_process_output.entry(thread.id())
                    {
                        e.insert(char.into());
                    } else {
                        self.partial_process_output
                            .get_mut(&thread.id())
                            .unwrap()
                            .push(char);
                    }
                } else if let Some(msg) = self.partial_process_output.remove(&thread.id()) {
                    println!("Thread: {} -> {}", thread.id(), msg);
                } else {
                    println!("Thread: {} -> ", thread.id());
                }
            }
            100 => {
                if self.create_new_thread.is_some() {
                    return InterfaceCallResult::Wait;
                }
                let tcs = ThreadCreationInfo {
                    start_addr: thread.vm_state.reg[4],
                    argument_ptr: thread.vm_state.reg[5],
                    new_id: self.next_thread_id(),
                    creator_id: thread.id(),
                };
                thread.vm_state.reg[2] = tcs.new_id.into_raw();
                self.create_new_thread = Some(tcs);
            }
            _ => return InterfaceCallResult::InvalidCall(id),
        }
        InterfaceCallResult::Continue
    }

    pub fn next_thread_id(&mut self) -> ThreadId {
        self.next_thread_id += 1;
        ThreadId::from_raw(self.next_thread_id)
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
    Exit,
}

#[derive(Default)]
pub struct SystemMemory {
    v_mem: Vec<Page>,
    ll_bit: bool,
}

pub type Page = [u8; 0x10000];
pub type PageId = usize;
