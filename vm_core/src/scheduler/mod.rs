use std::{
    collections::BinaryHeap,
    time::{Duration},
};

use crate::SystemTime;
use crate::util::ProcessId;

/// The Scheduler to schedule what task will run and for how long
/// 
/// Currently it just picks the task with the oldest value for (last_ran + sleep_for)
#[derive(Default)]
pub struct Scheduler {
    task_list: BinaryHeap<SchedulerTask>,
    tasks_to_remove: Vec<ProcessId>,

    average_instructions: RollingAverage,
    average_vm_duration: RollingAverage,
    average_total_duration: RollingAverage,
    current_time: Option<SystemTime>,

    total_iterations: u64,
}

#[derive(Debug)]
pub struct SchedulerTask {
    pid: ProcessId,
    last_ran: SystemTime,
    sleep_for: Option<Duration>,
}

impl SchedulerTask {
    pub fn time_available_to_run(&self) -> SystemTime {
        if let Some(sleep) = self.sleep_for {
            self.last_ran.checked_add(sleep).unwrap()
        } else {
            self.last_ran
        }
    }
}

impl PartialOrd for SchedulerTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            //backwards *^*
            other.time_available_to_run()
                .cmp(&self.time_available_to_run()),
        )
    }
}

impl Ord for SchedulerTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // our partial cmp is absolute
        Self::partial_cmp(self, other).unwrap()
    }
}

impl Eq for SchedulerTask {
    fn assert_receiver_is_total_eq(&self) {}
}

impl PartialEq for SchedulerTask {
    fn eq(&self, other: &Self) -> bool {
        self.time_available_to_run()
            .eq(&other.time_available_to_run())
    }
}

impl SchedulerTask {
    pub fn new(pid: ProcessId) -> Self {
        SchedulerTask {
            pid,
            last_ran: SystemTime::UNIX_EPOCH,
            sleep_for: None,
        }
    }
}

impl Scheduler {
    pub fn add_task(&mut self, pid: ProcessId) {
        self.task_list.push(SchedulerTask::new(pid));
    }

    pub fn total_iterations(&self) -> u64{
        self.total_iterations
    }

    pub fn remove_task(&mut self, pid: ProcessId) {
        self.tasks_to_remove.push(pid);
    }

    pub fn schedule_next_task(&mut self) -> Option<(ProcessId, u32)> {

        let now = crate::systime_now();
        if let Some(last_time) = self.current_time{
            let dur = now.duration_since(last_time).unwrap().as_nanos();
            self.average_total_duration.roll(dur as i128); 
        }
        self.current_time = Some(now);


        while !self.task_list.is_empty() {
            if self
                .tasks_to_remove
                .contains(&self.task_list.peek_mut().unwrap().pid)
            {
                let pid = self.task_list.pop().unwrap().pid;
                self.tasks_to_remove
                    .remove(self.tasks_to_remove.iter().position(|t| *t == pid).unwrap());
            } else {
                break;
            }
        }

        let task = self.task_list.peek_mut()?;
        
        if task.time_available_to_run().gt(&now){
            let dur = task.time_available_to_run().duration_since(now).unwrap();
            crate::wait_for(dur);
            self.current_time = Some(crate::systime_now());
        }

        let iterations = (self.average_instructions.average() * 200000)
            .checked_div(self.average_vm_duration.average())
            .unwrap_or(500);
        Some((task.pid, iterations as u32))
    }

    pub fn scheduled_task_report(
        &mut self,
        pid: ProcessId,
        iterations: u32,
        start: SystemTime,
        end: SystemTime,
    ) {
        let duration = (end.duration_since(start).unwrap()).as_nanos();
        self.average_instructions.roll(iterations as i128);
        self.average_vm_duration.roll(duration as i128);
        self.total_iterations += iterations as u64;

        if !self.tasks_to_remove.contains(&pid){
            let mut task = self.task_list.pop().unwrap();
    
            task.last_ran = start;
    
            self.task_list.push(task);
        }
    }

    pub fn has_task(&mut self) -> bool {
        !self.task_list.is_empty()
    }
}

#[derive(Default)]
struct RollingAverage {
    average: i128,
    counter: u8,
}

impl RollingAverage {
    pub fn roll(&mut self, val: i128) {
        if self.counter < 150 {
            self.counter += 1;
        }
        self.average = self.average + (val - self.average) / self.counter as i128;
    }
    pub fn average(&self) -> i128 {
        self.average
    }
}
