use std::{
    collections::HashMap,
    fmt::Display,
    sync::{atomic::AtomicBool, Mutex, RwLock},
};

use rclite::Arc;

use crate::{
    task::{Task, TaskMemory},
    util::{Page, TaskId},
};

#[derive(Default)]
pub struct TaskPool {
    pub task_pool: HashMap<TaskId, Arc<Mutex<Task>>>,
}

impl TaskPool {
    pub fn get_task(&self, task: TaskId) -> Arc<Mutex<Task>> {
        self.task_pool.get(&task).unwrap().clone()
    }

    pub fn add_task(&mut self, task: Task) {
        self.task_pool
            .insert(task.tid(), Arc::new(Mutex::new(task)));
    }

    pub fn remove_task(&mut self, tid: TaskId) -> Arc<Mutex<Task>> {
        self.task_pool.remove(&tid).unwrap()
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct PageId(usize);
impl PageId {
    pub(crate) fn raw(&self) -> usize {
        self.0
    }
}

impl Display for PageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PageId({})", self.0)
    }
}

#[derive(Default)]
pub struct TaskPoolSharedMemory {
    pub v_mem: RwLock<Vec<(Arc<Page>, PageMetaData)>>,
    pub ll_bit: AtomicBool,
}

impl TaskPoolSharedMemory {
    pub fn new_page(&self) -> Arc<Page> {
        let mut writter = self.v_mem.write().unwrap();
        for (page_id, meta) in writter.iter_mut().enumerate() {
            if meta.0.strong_count() == 1 {
                meta.0 = Arc::new(Page::new());
                return meta.0.clone();
            }
        }
        let new = Arc::new(Page::new());
        writter.push((new.clone(), PageMetaData {}));
        new
    }

    pub fn task_with_mapping<R, F>(
        &self,
        task: &mut Task,
        mem: &mut TaskMemory<'_, '_>,
        scope: F,
    ) -> R
    where
        F: for<'f> FnOnce(&mut Task, &mut TaskMemory<'_, 'f>) -> R,
    {
        for (page_id, v_addr) in &task.memory_mapping.mapping {
            //extend the lifetime
            let page = unsafe { std::mem::transmute::<&Page, &Page>(&**page_id) };
            mem.mem[*v_addr as usize] = Some(page);
        }

        let res = scope(task, mem);

        // make sure that after extending the lifetime we MUST remove all the references we placed into here ( or things break badly :) )
        for (_, v_addr) in &task.memory_mapping.mapping {
            mem.mem[*v_addr as usize] = None;
        }

        res
        // todo!();
    }
}

pub struct PageMetaData {}
