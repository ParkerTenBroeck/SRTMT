pub mod syscore;

use crate::SystemTime;

pub use syscore::*;

// ------------------------------------------------------------------

use crate::task::{Task, TaskError, TaskMemory, TaskRunResult};

use crate::taskpool::TaskPool;
use crate::util::{Page, ProcessId};

#[derive(Default)]
pub struct System {
    tasks: TaskPool,
    core: SystemCore,
}

impl System {
    pub fn run_blocking(&mut self) -> u64 {
        //well.... idk what you want from me??? better data structures pffff thats for nerds :)
        let mut shit_bool = false;
        let mut mem = TaskMemory::new(&mut shit_bool);

        while let Some((pid, iterations)) = self.core.scheduler.schedule_next_task() {
            let (res, start, end) = self.run_task(pid, &mut mem, iterations);

            self.post_task_stuff();

            let (iterations, _remove) = match res {
                Ok(ok) => match ok {
                    TaskRunResult::Continue => (iterations, false),
                    TaskRunResult::Wait(actually_ran) => (actually_ran, false),
                    TaskRunResult::Exit(actually_ran, code) => {
                        tracing::info!("Task: {} exited with code: {}", pid, code);
                        self.remove_task(pid);
                        (actually_ran, true)
                    }
                },
                Err((err, ran)) => {
                    tracing::info!(
                        "Task: {} encountered an error: {:#?}\nDUMP: {:#?}\nTerminating",
                        pid,
                        err,
                        self.tasks.get_task(pid)
                    );
                    self.remove_task(pid);
                    (ran, false)
                }
            };
            self.core
                .scheduler
                .scheduled_task_report(pid, iterations, start, end);
        }
        self.core.scheduler.total_iterations()
    }

    #[inline(always)]
    fn post_task_stuff(&mut self) {
        if let Some(new_task_info) = self.core.create_new_task.take() {
            let mut new_task = Task::new(new_task_info.new_id);

            let shared_core_and_data = self
                .tasks
                .get_task(new_task_info.creator_id)
                .memory_mapping
                .mapping
                .first()
                .unwrap()
                .0;
            new_task
                .memory_mapping
                .mapping
                .push((shared_core_and_data, 0x0)); //first data page shared between two tasks

            // self.tasks.sys_mem.v_mem.push(self.tasks.sys_mem.v_mem[shared_core_and_data]); //personal stack memory for task
            new_task
                .memory_mapping
                .mapping
                .push((self.tasks.sys_mem.v_mem.len(), 0x7FFF));
            self.tasks.sys_mem.v_mem.push([0; 0x10000]); //personal stack memory for task

            new_task.vm_state.pc = new_task_info.start_addr; //start of function
            new_task.vm_state.reg[4] = new_task_info.argument_ptr; //ptr to arguments in memory
            new_task.vm_state.reg[29] = 0x80000000; //start of stack
            new_task.vm_state.reg[31] = 0xFFFFFFFF;
            tracing::info!(
                "Created new task: {}\nDUMP{{:?}}",
                new_task.pid(),
                //new_task
            );
            self.add_task(new_task);
        }
    }

    fn remove_task(&mut self, pid: ProcessId) {
        self.tasks.remove_task(pid);
        self.core.scheduler.remove_task(pid);
    }

    fn add_task(&mut self, task: Task) {
        self.core.scheduler.add_task(task.pid());
        self.tasks.add_task(task);
    }

    pub fn add_task_with_pages<const SIZE: usize>(
        &mut self,
        initial_pages: [u16; SIZE],
    ) -> [&mut Page; SIZE] {
        let mut task = Task::new(self.core.next_task_id());

        let initial_len = self.tasks.sys_mem.v_mem.len();

        for page in initial_pages {
            task.memory_mapping
                .mapping
                .push((self.tasks.sys_mem.v_mem.len(), page));
            self.tasks.sys_mem.v_mem.push([0; 0x10000]);
        }

        let pid = task.pid();
        self.add_task(task);

        let mut t = Vec::new();

        let mut start_page_id = 0usize;
        let mut pages = self.tasks.sys_mem.v_mem.as_mut_slice();

        for page_id in initial_len..(initial_len + SIZE) {
            pages = &mut pages[(page_id - start_page_id)..];
            match std::mem::take(&mut pages) {
                [] => panic!(
                    "Page mapping doesnt exist: page_id: {} for task: {}",
                    page_id, pid
                ),
                [first, rest @ ..] => {
                    pages = rest;
                    // this is the same deal as the ll_bit thing we unset this to none before we return
                    // so no lifetime is outlived
                    t.push(first);
                }
            }
            start_page_id = page_id + 1;
        }

        t.try_into().unwrap()
    }

    fn run_task<'c>(
        &mut self,
        pid: ProcessId,
        mem: &mut TaskMemory<'c>,
        iters: u32,
    ) -> (
        Result<TaskRunResult, (TaskError, u32)>,
        SystemTime,
        SystemTime,
    ) {
        let task = self.tasks.task_pool.get_mut(&pid).unwrap();

        // this is sketchy but we know that we un-set this later in the function before we return
        // so we can (hopfully) do this even if its kinda really bad
        let ll_bit = &mut self.tasks.sys_mem.ll_bit;
        let mut ll_bit = unsafe { std::mem::transmute(ll_bit) };
        std::mem::swap(&mut ll_bit, &mut mem.ll_bit);

        let mut start_page_id = 0usize;
        let mut pages = self.tasks.sys_mem.v_mem.as_mut_slice();

        for (page_id, v_addr) in &task.memory_mapping.mapping {
            pages = &mut pages[(page_id - start_page_id)..];
            match std::mem::take(&mut pages) {
                [] => panic!(
                    "Page mapping doesnt exist: page_id: {} for task: {}",
                    page_id,
                    task.pid()
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

        let start = crate::systime_now();
        let res = task.run(&mut self.core, mem, iters);
        let end = crate::systime_now();

        for (_, v_addr) in &task.memory_mapping.mapping {
            mem.mem[*v_addr as usize] = None;
        }
        std::mem::swap(&mut ll_bit, &mut mem.ll_bit);

        (res, start, end)
    }

    pub fn remove_page(&mut self, to_remove_page_id: usize) {
        self.tasks.sys_mem.v_mem.remove(to_remove_page_id);
        for task in &mut self.tasks.task_pool.values_mut() {
            for (page_id, _) in &mut task.memory_mapping.mapping {
                if *page_id >= to_remove_page_id {
                    *page_id -= 1;
                }
            }
        }
    }
}
