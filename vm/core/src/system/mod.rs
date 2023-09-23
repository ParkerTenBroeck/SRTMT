pub mod syscore;

use crate::scheduler::SchedulerTask;
use crate::SystemTime;

use rclite::Arc;
pub use syscore::*;

// ------------------------------------------------------------------

use crate::task::{PageVAddressStart, Task, TaskError, TaskMemory, TaskRunResult};

use crate::taskpool::{TaskPool, TaskPoolSharedMemory};
use crate::util::{Page, ThreadId, TaskId};

#[derive(Default)]
pub struct System {
    tasks: TaskPool,
    pub core: SystemCore,
    sys_mem: Arc<TaskPoolSharedMemory>,
}

impl System {
    pub fn run_blocking(&mut self) -> u64 {
        let ll_arc = self.sys_mem.clone();
        let mut mem = TaskMemory::new(&ll_arc.ll_bit);

        while let Some((mut task, iterations)) = self.core.scheduler.schedule_next_task() {
            let (res, start, end) = self.run_task(&mut task, &mut mem, iterations);

            let tid = task.tid();
            //self.post_task_stuff();

            let (iterations, remove) = match res {
                Ok(ok) => match ok {
                    TaskRunResult::Continue => (iterations, false),
                    TaskRunResult::Wait(actually_ran) => (actually_ran, false),
                    TaskRunResult::Exit(actually_ran, code) => {
                        tracing::info!("Task: {} exited with code: {}", tid.0, code);

                        // if tid == self.tasks.get_tasks(tid.1);

                        (actually_ran, true)
                    }
                },
                Err((err, ran)) => {
                    tracing::info!(
                        "Task: {} encountered an error: {:#?}\nDUMP: {:#?}\nTerminating",
                        tid.0,
                        err,
                        self.tasks.get_task(tid.0)
                    );
                    (ran, true)
                }
            };

            if remove{
                self.tasks.remove_task(task.tid().0);
            }
            self.core.scheduler.scheduled_task_report(
                if remove { None } else { Some(task) },
                iterations,
                start,
                end,
            );
        }

        self.core.scheduler.total_iterations()
    }

    fn remove_task(&mut self, task: TaskId) {
        self.tasks.remove_task(task);

        self.core.scheduler.remove_task(task);
    }

    fn add_task(&mut self, task: Task) {
        self.core.scheduler.add_task(task.thread_id());
        self.tasks.add_task(task);
    }

    pub fn add_task_with_pages(
        &mut self,
        initial_pages: &[u16],
        initializer: impl FnOnce(Vec<(Arc<Page>, PageVAddressStart)>),
    ) {
        let mut task = Task::new_mainthread(self.next_task_id());

        for page in initial_pages {
            task.memory_mapping
                .mapping
                .push((self.sys_mem.new_page(), *page));
        }

        let mem = self.sys_mem.v_mem.read().unwrap();
        let mut t = Vec::new();
        for mapping in &task.memory_mapping.mapping {
            t.push(mapping.clone());
        }
        drop(mem);
        initializer(t);
        self.add_task(task);
    }

    fn run_task(
        &mut self,
        scheduler_task: &mut SchedulerTask,
        mem: &mut TaskMemory<'_, '_>,
        iters: u32,
    ) -> (
        Result<TaskRunResult, (TaskError, u32)>,
        SystemTime,
        SystemTime,
    ) {
        let task = self.tasks.get_task(scheduler_task.tid().0);
        let mut task = task.lock().unwrap();

        self.sys_mem
            .clone()
            .task_with_mapping(&mut task, mem, |task, mem| {
                let start = crate::systime_now();
                let res = task.run(self, scheduler_task, mem, iters);
                let end = crate::systime_now();
                (res, start, end)
            })
    }
}
