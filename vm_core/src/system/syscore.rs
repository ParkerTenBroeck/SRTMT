use std::collections::HashMap;

use crate::{
    scheduler::Scheduler,
    task::{Task, TaskError, TaskMemory},
    util::ProcessId,
};

#[derive(Default)]
pub struct SystemCore {
    pub(super) partial_process_output: HashMap<ProcessId, String>,
    pub(super) create_new_task: Option<TaskCreationInfo>,
    pub(super) next_task_id: u32,
    pub(super) scheduler: Scheduler,
}

pub struct TaskCreationInfo {
    pub(super) start_addr: u32,
    pub(super) argument_ptr: u32,
    pub(super) new_id: ProcessId,
    pub(super) creator_id: ProcessId,
}

impl SystemCore {
    pub fn system_call(
        &mut self,
        id: u32,
        task: &mut Task,
        mem: &mut TaskMemory<'_>,
    ) -> InterfaceCallResult {
        match id {
            0 => return InterfaceCallResult::Exit,
            1 => {
                tracing::info!("Task: {} -> {}", task.pid(), task.vm_state.reg[4] as i32);
            }
            4 => {
                let mut address = task.vm_state.reg[4];
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
                                TaskError::MemoryDoesNotExistError(address, task.vm_state.pc),
                            ))
                        }
                    }
                }

                let str = String::from_utf8(str);
                if let Ok(str) = str {
                    tracing::info!("Task: {} -> {}", task.pid(), str);
                } else {
                    return InterfaceCallResult::MalformedCallArgs;
                }
            }

            5 => {
                let char = task.vm_state.reg[4] as u8 as char;
                if char != '\n' {
                    if let std::collections::hash_map::Entry::Vacant(e) =
                        self.partial_process_output.entry(task.pid())
                    {
                        e.insert(char.into());
                    } else {
                        self.partial_process_output
                            .get_mut(&task.pid())
                            .unwrap()
                            .push(char);
                    }
                } else if let Some(msg) = self.partial_process_output.remove(&task.pid()) {
                    tracing::info!("Task: {} -> {}", task.pid(), msg);
                } else {
                    tracing::info!("Task: {} -> ", task.pid());
                }
            }
            100 => {
                if self.create_new_task.is_some() {
                    return InterfaceCallResult::Wait;
                }
                let tcs = TaskCreationInfo {
                    start_addr: task.vm_state.reg[4],
                    argument_ptr: task.vm_state.reg[5],
                    new_id: self.next_task_id(),
                    creator_id: task.pid(),
                };
                task.vm_state.reg[2] = tcs.new_id.into_raw();
                self.create_new_task = Some(tcs);
            }
            _ => return InterfaceCallResult::InvalidCall(id),
        }
        InterfaceCallResult::Continue
    }

    pub fn next_task_id(&mut self) -> ProcessId {
        self.next_task_id += 1;
        ProcessId::from_raw(self.next_task_id)
    }

    pub fn breakpoint(
        &mut self,
        id: u32,
        _task: &mut Task,
        _mem: &mut TaskMemory<'_>,
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
    Wait,
    Exit,
}
