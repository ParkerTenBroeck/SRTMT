use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use crate::{
    scheduler::{Scheduler, SchedulerTask},
    task::{Task, TaskError, TaskMemory},
    util::TaskId,
};

use super::System;

#[derive(Default)]
pub struct SystemCore {
    pub(super) partial_process_output: HashMap<TaskId, String>,
    pub(super) next_task_id: u32,
    pub(super) scheduler: Scheduler,
}

impl System {
    pub fn system_call(
        &mut self,
        id: u32,
        task: &mut Task,
        scheduler_task: &mut SchedulerTask,
        mem: &mut TaskMemory<'_, '_>,
    ) -> InterfaceCallResult {
        match id {
            0 => return InterfaceCallResult::Exit,
            1 => {
                tracing::info!("Task: {} -> {}", task.tid(), task.vm_state.reg[4] as i32);
            }
            4 => {
                let mut address = task.vm_state.reg[4];
                let mut str: Vec<u8> = Vec::new();
                loop {
                    match mem.mem.get(address as usize >> 16).unwrap() {
                        Some(page) => {
                            let char = page.get_u8(address as u16);
                            if char == 0 {
                                break;
                            }
                            str.push(char);
                            address += 1;
                        }
                        None => {
                            return InterfaceCallResult::ImmediateKill(Some(
                                TaskError::MemoryDoesNotExistError(address, task.vm_state.pc),
                            ))
                        }
                    }
                }

                let str = String::from_utf8(str);
                if let Ok(str) = str {
                    tracing::info!("Task: {} -> {}", task.tid(), str);
                } else {
                    return InterfaceCallResult::MalformedCallArgs;
                }
            }

            5 => {
                let char = task.vm_state.reg[4] as u8 as char;
                if char != '\n' {
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        self.core.partial_process_output.entry(task.tid())
                    {
                        e.insert(char.into());
                    } else {
                        self.core
                            .partial_process_output
                            .get_mut(&task.tid())
                            .unwrap()
                            .push(char);
                    }
                } else if let Some(msg) = self.core.partial_process_output.remove(&task.tid()) {
                    tracing::info!("Task: {} -> {}", task.tid(), msg);
                } else {
                    tracing::info!("Task: {} -> ", task.tid());
                }
            }
            60 => {
                let time = crate::systime_now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap();
                let dur = time.as_nanos() as u64;
                task.vm_state.reg[2] = dur as u32;
                task.vm_state.reg[3] = (dur >> 32) as u32;
            }
            100 => {
                let mut new_task = Task::new_subthread(task.thread_id().1, self.next_task_id());

                let shared_core_and_data = task.memory_mapping.mapping.first().unwrap().clone();

                new_task.memory_mapping.mapping.push(shared_core_and_data);

                // tasks default stack
                new_task
                    .memory_mapping
                    .mapping
                    .push((self.sys_mem.new_page(), 0x7FFF));

                new_task.vm_state.pc = task.vm_state.reg[4];
                new_task.vm_state.reg[4] = task.vm_state.reg[5];
                new_task.vm_state.reg[29] = 0x80000000; //start of stack
                new_task.vm_state.reg[31] = 0xFFFFFFFF;

                tracing::info!(
                    "Created new task: {}\nDUMP{{:?}}",
                    new_task.tid(),
                    //new_task
                );
                task.vm_state.reg[2] = new_task.tid().into_raw();
                self.add_task(new_task);
            }
            101 => {
                let val = task.vm_state.reg[4] as u64 | ((task.vm_state.reg[5] as u64) << 32);
                let dur = Duration::from_nanos(val);
                scheduler_task.sleep_for = Some(dur);
                //23479387.80014355
                //248832255.48680574
                //233960039
                return InterfaceCallResult::Wait;
            }
            102 => {
                // stop doing tings and stuff and
                return InterfaceCallResult::Wait;
            }
            // Futex wake
            200 => {
                let futex_addr = task.vm_state.reg[4];
                let tasks_to_wake = task.vm_state.reg[5];
            }
            // Futex wait
            201 => {
                let futex_addr = task.vm_state.reg[4];
                let condition = task.vm_state.reg[5];
            }
            _ => return InterfaceCallResult::InvalidCall(id),
        }
        InterfaceCallResult::Continue
    }

    pub fn next_task_id(&mut self) -> TaskId {
        self.core.next_task_id += 1;
        TaskId::from_raw(self.core.next_task_id)
    }

    pub fn breakpoint(
        &mut self,
        id: u32,
        _task: &mut Task,
        _scheduler_task: &mut SchedulerTask,
        _mem: &TaskMemory<'_, '_>,
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
    ImmediateKill(Option<TaskError>),
    MalformedCallArgs,
    InvalidCall(u32),
    WaitRepeated,
    Wait,
    Exit,
}
