use std::collections::HashMap;

use crate::{
    task::Task,
    util::{Page, ProcessId},
};

#[derive(Default)]
pub struct TaskPool {
    pub sys_mem: TaskPoolSharedMemory,
    pub task_pool: HashMap<ProcessId, Task>,
}

impl TaskPool {
    pub fn get_task(&self, pid: ProcessId) -> &Task {
        self.task_pool.get(&pid).unwrap()
    }

    pub fn get_task_mut(&mut self, pid: ProcessId) -> &mut Task {
        self.task_pool.get_mut(&pid).unwrap()
    }

    pub fn add_task(&mut self, task: Task) {
        self.task_pool.insert(task.pid(), task);
    }

    pub fn remove_task(&mut self, pid: ProcessId) {
        let mut task = self.task_pool.remove(&pid).unwrap();
        for st in self.task_pool.values_mut() {
            for (pageid, _v_addr) in &st.memory_mapping.mapping {
                task.memory_mapping.mapping.retain(|rt| rt.0 == *pageid);
            }
        }
        for (page_id, _) in task.memory_mapping.mapping {
            self.remote_page(page_id);
        }
    }

    fn remote_page(&mut self, remove_page_id: usize) {
        for st in self.task_pool.values_mut() {
            for (page_id, _v_addr) in &mut st.memory_mapping.mapping {
                if *page_id >= remove_page_id {
                    *page_id -= 1;
                }
            }
        }
    }
}

#[derive(Default)]
pub struct TaskPoolSharedMemory {
    pub v_mem: Vec<Page>,
    pub ll_bit: bool,
}
