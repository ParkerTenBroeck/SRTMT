use std::fmt::Debug;

use crate::{
    system::SystemCore,
    util::{Page, PageId, ProcessId},
};

#[derive(Debug)]
pub struct Task {
    pid: ProcessId,
    pub name: Option<String>,
    pub vm_state: VmState,
    pub memory_mapping: TaskMemoryMapping,
}

impl Task {
    pub fn new(id: ProcessId) -> Self {
        Self {
            pid: id,
            vm_state: Default::default(),
            memory_mapping: Default::default(),
            name: None,
        }
    }

    pub fn pid(&self) -> ProcessId {
        self.pid
    }
}

pub type PageVAddressStart = u16;

#[derive(Default)]
pub struct TaskMemoryMapping {
    pub mapping: Vec<(PageId, PageVAddressStart)>,
}

impl Debug for TaskMemoryMapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct Mapped {
            p_id: PageId,
            pvas: PageVAddressStart,
        }
        impl Debug for Mapped {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(&format!(
                    "PageId: {} -> VAddress: {:#010X}",
                    self.p_id,
                    (self.pvas as u32) << 16
                ))
                .finish()
            }
        }
        f.debug_list()
            .entries(
                self.mapping
                    .iter()
                    .map(|&(p_id, pvas)| Mapped { p_id, pvas }),
            )
            .finish()
    }
}

#[derive(Default)]
pub struct VmState {
    pub pc: u32,
    pub hi: u32,
    pub lo: u32,
    pub reg: [u32; 32],
}

impl Debug for VmState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VmState")
            .field("pc", &self.pc)
            .field("hi", &self.hi)
            .field("lo", &self.lo)
            .field("$zero", &self.reg[0])
            .field("$1/at", &self.reg[1])
            .field("$2/v0", &self.reg[2])
            .field("$3/v1", &self.reg[3])
            .field("$4/a0", &self.reg[4])
            .field("$5/a1", &self.reg[5])
            .field("$6/a2", &self.reg[6])
            .field("$7/a3", &self.reg[7])
            .field("$8/t0", &self.reg[8])
            .field("$9/t1", &self.reg[9])
            .field("$10/t2", &self.reg[10])
            .field("$11/t3", &self.reg[11])
            .field("$12/t4", &self.reg[12])
            .field("$13/t5", &self.reg[13])
            .field("$14/t6", &self.reg[14])
            .field("$15/t7", &self.reg[15])
            .field("$16/s0", &self.reg[16])
            .field("$17/s1", &self.reg[17])
            .field("$18/s2", &self.reg[18])
            .field("$19/s3", &self.reg[19])
            .field("$20/s4", &self.reg[20])
            .field("$21/s5", &self.reg[21])
            .field("$22/s6", &self.reg[22])
            .field("$23/s7", &self.reg[23])
            .field("$24/t8", &self.reg[24])
            .field("$25/t9", &self.reg[25])
            .field("$26/k0", &self.reg[26])
            .field("$27/k1", &self.reg[27])
            .field("$28/gp", &self.reg[28])
            .field("$29/sp", &self.reg[29])
            .field("$30/s8/fp", &self.reg[30])
            .field("$31/ra", &self.reg[31])
            .finish()
    }
}

//macros
//jump encoding
macro_rules! jump_immediate_address {
    ($expr:expr) => {
        ((($expr as u32) & 0b00000011111111111111111111111111) << 2)
    };
}

#[allow(unused)]
macro_rules! jump_immediate_offset {
    ($expr:expr) => {
        (($expr as i32) << 6) >> 4
    };
}

//immediate encoding
macro_rules! immediate_immediate_signed_extended {
    ($expr:expr) => {
        ((($expr as i32) << 16) >> 16) as u32
    };
}
macro_rules! immediate_immediate_zero_extended {
    ($expr:expr) => {
        (($expr as u32) & 0xFFFF)
    };
}

macro_rules! immediate_immediate_address {
    ($expr:expr) => {
        (($expr as i32) << 16) >> 14
    };
}

#[allow(unused)]
macro_rules! immediate_immediate_unsigned_hi {
    ($expr:expr) => {
        (($expr as u32) << 16)
    };
}

macro_rules! immediate_s {
    ($expr:expr) => {
        ((($expr as u32) >> 21) & 0b11111) as usize
    };
}

macro_rules! immediate_t {
    ($expr:expr) => {
        ((($expr as u32) >> 16) & 0b11111) as usize
    };
}

macro_rules! register_s {
    ($expr:expr) => {
        ((($expr as u32) >> 21) & 0b11111) as usize
    };
}

macro_rules! register_t {
    ($expr:expr) => {
        ((($expr as u32) >> 16) & 0b11111) as usize
    };
}

macro_rules! register_d {
    ($expr:expr) => {
        ((($expr as u32) >> 11) & 0b11111) as usize
    };
}

macro_rules! register_a {
    ($expr:expr) => {
        (($expr as u32) >> 6) & 0b11111
    };
}

// hack for likely and unlikely
#[inline]
#[cold]
fn cold() {}

#[inline]
fn likely(b: bool) -> bool {
    if !b {
        cold()
    }
    b
}

#[inline]
fn unlikely(b: bool) -> bool {
    if b {
        cold()
    }
    b
}

pub enum TaskRunResult {
    Continue,
    Wait(u32),
    Exit(u32, u32),
}

pub type VmPtr = u32;
pub type VmInstruction = u32;
pub type VmInstructionAddress = VmPtr;

#[derive(Debug)]
pub enum TaskError {
    DivByZeroError(VmInstructionAddress),
    MemoryDoesNotExistError(VmPtr, VmInstructionAddress),
    InvalidOperation(VmInstructionAddress, VmInstruction),
    MemoryAllignmentError(u8, VmInstructionAddress),
    OverflowError(VmInstructionAddress),
}

pub struct TaskMemory<'a> {
    pub ll_bit: &'a mut bool,
    pub mem: [Option<&'a mut Page>; 0x10000],
}

impl<'a> TaskMemory<'a> {
    pub fn new(tmp_bool: &'a mut bool) -> Self {
        const NONE_C: Option<&'static mut Page> = None;
        TaskMemory {
            ll_bit: tmp_bool,
            mem: [NONE_C; 0x10000],
        }
    }
}

impl Task {
    pub fn run(
        &mut self,
        sys: &mut SystemCore,
        mem: &mut TaskMemory<'_>,
        iterations: u32,
    ) -> Result<TaskRunResult, (TaskError, u32)> {
        let mut ins_cache = {
            (
                {
                    match &mut mem.mem[0] {
                        Some(page) => *page as *const [u8; 0x10000],
                        None => {
                            return Err((
                                TaskError::MemoryDoesNotExistError(
                                    self.vm_state.pc,
                                    self.vm_state.pc,
                                ),
                                0,
                            ))
                        }
                    }
                },
                self.vm_state.pc >> 16,
            )
        };

        for ran in 0..iterations {
            macro_rules! set_mem_alligned {
                ($add:expr, $val:expr, $fn_type:ty) => {
                    unsafe {
                        let address = $add;

                        let page = match &mut mem.mem[address as usize >> 16] {
                            Some(page) => page,
                            None => {
                                return Err((
                                    TaskError::MemoryDoesNotExistError(address, self.vm_state.pc),
                                    ran,
                                ))
                            }
                        };
                        let item = page.get_unchecked_mut(address as u16 as usize);
                        *core::mem::transmute::<&mut u8, &mut $fn_type>(item) = $val
                    }
                };
            }

            macro_rules! get_mem_alligned {
                ($add:expr, $fn_type:ty) => {
                    unsafe {
                        let address = $add;

                        let page = match &mut mem.mem[address as usize >> 16] {
                            Some(page) => page,
                            None => {
                                return Err((
                                    TaskError::MemoryDoesNotExistError(address, self.vm_state.pc),
                                    ran,
                                ))
                            }
                        };
                        let item = page.get_unchecked_mut(address as u16 as usize);
                        *core::mem::transmute::<&u8, &$fn_type>(item)
                    }
                };
            }

            macro_rules! system_call {
                ($id:expr) => {
                    interface_call!(system_call, $id);
                };
            }

            macro_rules! breakpoint {
                ($id:expr) => {
                    interface_call!(breakpoint, $id);
                };
            }

            let op: u32 = unsafe {
                if unlikely(self.vm_state.pc >> 16 != ins_cache.1) {
                    ins_cache = (
                        {
                            match &mem.mem[self.vm_state.pc as usize >> 16] {
                                Some(page) => *page as *const [u8; 0x10000],
                                None => {
                                    return Err((
                                        TaskError::MemoryDoesNotExistError(
                                            self.vm_state.pc,
                                            self.vm_state.pc,
                                        ),
                                        ran,
                                    ))
                                }
                            }
                        },
                        self.vm_state.pc >> 16,
                    );
                }

                let item = (*ins_cache.0).get_unchecked(self.vm_state.pc as u16 as usize);
                *core::mem::transmute::<&u8, &u32>(item)
            };
            self.vm_state.pc = self.vm_state.pc.wrapping_add(4);

            macro_rules! interface_call{
                ($kind:ident, $id:expr) => {
                    match sys.$kind($id, self, mem){
                        crate::system::InterfaceCallResult::Continue => {},
                        crate::system::InterfaceCallResult::ImmediateKill(reason) => {
                            if let Some(reason) = reason {
                                return Err((reason, ran))
                            }else{
                                return Err((TaskError::InvalidOperation(self.vm_state.pc, op),ran))
                            }
                        },
                        crate::system::InterfaceCallResult::Exit => {
                            return Ok(TaskRunResult::Exit(ran, 0))
                        }
                        crate::system::InterfaceCallResult::InvalidCall(id) => {
                            return Err((TaskError::InvalidOperation(self.vm_state.pc, id),ran))
                        },
                        crate::system::InterfaceCallResult::MalformedCallArgs => {

                            return Err((TaskError::InvalidOperation(self.vm_state.pc, op), ran))
                        }
                        crate::system::InterfaceCallResult::Wait => {
                            self.vm_state.pc -= 4; //we need to re-run this system call when we try again
                            return Ok(TaskRunResult::Wait(ran))
                        },
                    }
                }
            }

            match op >> 26 {
                0 => {
                    match op & 0b111111 {
                        // REGISTER formatted instructions

                        //special
                        0b001111 => {
                            //sync
                        }

                        //arithmatic
                        0b100000 => {
                            //ADD
                            match (self.vm_state.reg[register_s!(op)] as i32)
                                .checked_add(self.vm_state.reg[register_t!((op))] as i32)
                            {
                                Some(val) => {
                                    self.vm_state.reg[register_d!(op)] = val as u32;
                                }

                                None => {
                                    return Err((TaskError::OverflowError(self.vm_state.pc), ran));
                                }
                            }
                        }
                        0b100001 => {
                            //ADDU
                            self.vm_state.reg[register_d!(op)] = self.vm_state.reg[register_s!(op)]
                                .wrapping_add(self.vm_state.reg[register_t!((op))])
                        }
                        0b100100 => {
                            //AND
                            self.vm_state.reg[register_d!(op)] = self.vm_state.reg[register_s!(op)]
                                & self.vm_state.reg[register_t!((op))]
                        }
                        0b011010 => {
                            //DIV
                            let t = self.vm_state.reg[register_t!(op)] as i32;
                            if likely(t != 0) {
                                let s = self.vm_state.reg[register_s!(op)] as i32;
                                self.vm_state.lo = (s.wrapping_div(t)) as u32;
                                self.vm_state.hi = (s.wrapping_rem(t)) as u32;
                            } else {
                                return Err((TaskError::DivByZeroError(self.vm_state.pc), ran));
                            }
                        }
                        0b011011 => {
                            //DIVU
                            let t = self.vm_state.reg[register_t!(op)];
                            if likely(t != 0) {
                                let s = self.vm_state.reg[register_s!(op)];
                                self.vm_state.lo = s.wrapping_div(t);
                                self.vm_state.hi = s.wrapping_rem(t);
                            } else {
                                return Err((TaskError::DivByZeroError(self.vm_state.pc), ran));
                            }
                        }
                        0b011000 => {
                            //MULT
                            let t = self.vm_state.reg[register_t!(op)] as i32 as i64;
                            let s = self.vm_state.reg[register_s!(op)] as i32 as i64;
                            let result = t.wrapping_mul(s);
                            self.vm_state.lo = (result & 0xFFFFFFFF) as u32;
                            self.vm_state.hi = (result >> 32) as u32;
                        }
                        0b011001 => {
                            //MULTU
                            let t = self.vm_state.reg[register_t!(op)] as u64;
                            let s = self.vm_state.reg[register_s!(op)] as u64;
                            let result = t.wrapping_mul(s);
                            self.vm_state.lo = (result & 0xFFFFFFFF) as u32;
                            self.vm_state.hi = (result >> 32) as u32;
                        }
                        0b100111 => {
                            //NOR
                            self.vm_state.reg[register_d!(op)] = !(self.vm_state.reg
                                [register_s!(op)]
                                | self.vm_state.reg[register_t!(op)]);
                        }
                        0b100101 => {
                            //OR
                            self.vm_state.reg[register_d!(op)] = self.vm_state.reg[register_s!(op)]
                                | self.vm_state.reg[register_t!(op)];
                        }
                        0b100110 => {
                            //XOR
                            self.vm_state.reg[register_d!(op)] = self.vm_state.reg[register_s!(op)]
                                ^ self.vm_state.reg[register_t!(op)];
                        }
                        0b000000 => {
                            //SLL
                            self.vm_state.reg[register_d!(op)] =
                                self.vm_state.reg[register_t!(op)] << register_a!(op);
                        }
                        0b000100 => {
                            //SLLV
                            self.vm_state.reg[register_d!(op)] = (self.vm_state.reg
                                [register_t!(op)])
                                << (0b11111 & self.vm_state.reg[register_s!(op)]);
                        }
                        0b000011 => {
                            //SRA
                            self.vm_state.reg[register_d!(op)] =
                                (self.vm_state.reg[register_t!(op)] as i32 >> register_a!(op))
                                    as u32;
                        }
                        0b000111 => {
                            //SRAV
                            self.vm_state.reg[register_d!(op)] = (self.vm_state.reg[register_t!(op)]
                                as i32
                                >> (0b11111 & self.vm_state.reg[register_s!(op)]))
                                as u32;
                        }
                        0b000010 => {
                            //SRL
                            self.vm_state.reg[register_d!(op)] =
                                (self.vm_state.reg[register_t!(op)] >> register_a!(op)) as u32;
                        }
                        0b000110 => {
                            //SRLV
                            self.vm_state.reg[register_d!(op)] = (self.vm_state.reg
                                [register_t!(op)]
                                >> (0b11111 & self.vm_state.reg[register_s!(op)]))
                                as u32;
                        }
                        0b100010 => {
                            //SUB
                            if let Option::Some(val) = (self.vm_state.reg[register_s!(op)] as i32)
                                .checked_sub(self.vm_state.reg[register_t!(op)] as i32)
                            {
                                self.vm_state.reg[register_d!(op)] = val as u32;
                            } else {
                                return Err((TaskError::OverflowError(self.vm_state.pc), ran));
                            }
                        }
                        0b100011 => {
                            //SUBU
                            self.vm_state.reg[register_d!(op)] = self.vm_state.reg[register_s!(op)]
                                .wrapping_sub(self.vm_state.reg[register_t!(op)]);
                        }

                        //comparason
                        0b101010 => {
                            //SLT
                            self.vm_state.reg[register_d!(op)] = {
                                if (self.vm_state.reg[register_s!(op)] as i32)
                                    < (self.vm_state.reg[register_t!(op)] as i32)
                                {
                                    1
                                } else {
                                    0
                                }
                            }
                        }
                        0b101011 => {
                            //SLTU
                            self.vm_state.reg[register_d!(op)] = {
                                if self.vm_state.reg[register_s!(op)]
                                    < self.vm_state.reg[register_t!(op)]
                                {
                                    1
                                } else {
                                    0
                                }
                            }
                        }

                        //jump
                        0b001001 => {
                            //JALR
                            self.vm_state.reg[31] = self.vm_state.pc;
                            self.vm_state.pc = self.vm_state.reg[register_s!(op)];
                        }
                        0b001000 => {
                            //JR
                            self.vm_state.pc = self.vm_state.reg[register_s!(op)];
                        }

                        //data movement
                        0b010000 => {
                            //MFHI
                            self.vm_state.reg[register_d!(op)] = self.vm_state.hi;
                        }
                        0b010010 => {
                            //MFLO
                            self.vm_state.reg[register_d!(op)] = self.vm_state.lo;
                        }
                        0b010001 => {
                            //MTHI
                            self.vm_state.hi = self.vm_state.reg[register_s!(op)];
                        }
                        0b010011 => {
                            //MTLO
                            self.vm_state.lo = self.vm_state.reg[register_s!(op)];
                        }

                        //special
                        0b001100 => {
                            //syscall
                            let id = (op >> 6) & 0b11111111111111111111;
                            system_call!(id);
                        }
                        0b001101 => {
                            //break
                            let id = (op >> 6) & 0b11111111111111111111;
                            breakpoint!(id);
                        }
                        0b110100 => {
                            //TEQ
                            if self.vm_state.reg[register_s!(op)]
                                == self.vm_state.reg[register_t!(op)]
                            {
                                let id = (op >> 6) & 0b1111111111;
                                system_call!(id);
                            }
                        }
                        0b110000 => {
                            //TGE
                            if self.vm_state.reg[register_s!(op)] as i32
                                >= self.vm_state.reg[register_t!(op)] as i32
                            {
                                let id = (op >> 6) & 0b1111111111;
                                system_call!(id);
                            }
                        }
                        0b110001 => {
                            //TGEU
                            if self.vm_state.reg[register_s!(op)]
                                >= self.vm_state.reg[register_t!(op)]
                            {
                                let id = (op >> 6) & 0b1111111111;
                                system_call!(id);
                            }
                        }
                        0b110010 => {
                            //TIT
                            if (self.vm_state.reg[register_s!(op)] as i32)
                                < self.vm_state.reg[register_t!(op)] as i32
                            {
                                let id = (op >> 6) & 0b1111111111;
                                system_call!(id);
                            }
                        }
                        0b110011 => {
                            //TITU
                            if self.vm_state.reg[register_s!(op)]
                                < self.vm_state.reg[register_t!(op)]
                            {
                                let id = (op >> 6) & 0b1111111111;
                                system_call!(id);
                            }
                        }
                        0b110110 => {
                            //TNE
                            if self.vm_state.reg[register_s!(op)]
                                != self.vm_state.reg[register_t!(op)]
                            {
                                let id = (op >> 6) & 0b1111111111;
                                system_call!(id);
                            }
                        }

                        _ => return Err((TaskError::InvalidOperation(self.vm_state.pc, op), ran)),
                    }
                }
                //Jump instructions
                0b000010 => {
                    //jump
                    self.vm_state.pc = (self.vm_state.pc & 0b11110000000000000000000000000000)
                        | jump_immediate_address!(op);
                }
                0b000011 => {
                    //jal
                    self.vm_state.reg[31] = self.vm_state.pc;
                    self.vm_state.pc = (self.vm_state.pc & 0b11110000000000000000000000000000)
                        | jump_immediate_address!(op);
                }
                // IMMEDIATE formmated instructions

                // arthmetic
                0b001000 => {
                    //ADDI
                    if let Option::Some(val) = (self.vm_state.reg[immediate_s!(op)] as i32)
                        .checked_add(immediate_immediate_signed_extended!(op) as i32)
                    {
                        self.vm_state.reg[immediate_t!(op)] = val as u32;
                    } else {
                        return Err((TaskError::OverflowError(self.vm_state.pc), ran));
                    }
                }
                0b001001 => {
                    //ADDIU
                    self.vm_state.reg[immediate_t!(op)] = (self.vm_state.reg[immediate_s!(op)])
                        .wrapping_add(immediate_immediate_signed_extended!(op));
                }
                0b001100 => {
                    //ANDI
                    self.vm_state.reg[immediate_t!(op)] =
                        self.vm_state.reg[immediate_s!(op)] & immediate_immediate_zero_extended!(op)
                }
                0b001101 => {
                    //ORI
                    self.vm_state.reg[immediate_t!(op)] = self.vm_state.reg[immediate_s!(op)] as u32
                        | immediate_immediate_zero_extended!(op) as u32
                }
                0b001110 => {
                    //XORI
                    self.vm_state.reg[immediate_t!(op)] = self.vm_state.reg[immediate_s!(op)] as u32
                        ^ immediate_immediate_zero_extended!(op) as u32
                }

                // constant manupulating inctructions
                0b001111 => {
                    //LUI
                    self.vm_state.reg[immediate_t!(op)] =
                        immediate_immediate_zero_extended!(op) << 16;
                }

                // comparison Instructions
                0b001010 => {
                    //SLTI
                    self.vm_state.reg[immediate_t!(op)] = {
                        if (self.vm_state.reg[immediate_s!(op)] as i32)
                            < (immediate_immediate_signed_extended!(op) as i32)
                        {
                            1
                        } else {
                            0
                        }
                    }
                }
                0b001011 => {
                    //SLTIU
                    self.vm_state.reg[immediate_t!(op)] = {
                        if (self.vm_state.reg[immediate_s!(op)] as u32)
                            < (immediate_immediate_signed_extended!(op) as u32)
                        {
                            1
                        } else {
                            0
                        }
                    }
                }

                // branch instructions
                0b000100 => {
                    //BEQ
                    if self.vm_state.reg[immediate_s!(op)] == self.vm_state.reg[immediate_t!(op)] {
                        self.vm_state.pc = ((self.vm_state.pc as i32)
                            .wrapping_add(immediate_immediate_address!(op)))
                            as u32;
                    } else {
                        self.vm_state.pc += 4;
                    }
                }
                0b000001 => {
                    match immediate_t!(op) {
                        0b00001 => {
                            //BGEZ
                            if (self.vm_state.reg[immediate_s!(op)] as i32) >= 0 {
                                self.vm_state.pc = ((self.vm_state.pc as i32)
                                    .wrapping_add(immediate_immediate_address!(op)))
                                    as u32;
                            } else {
                                self.vm_state.pc += 4;
                            }
                        }
                        0b00000 => {
                            //BLTZ
                            if (self.vm_state.reg[immediate_s!(op)] as i32) < 0 {
                                self.vm_state.pc = ((self.vm_state.pc as i32)
                                    .wrapping_add(immediate_immediate_address!(op)))
                                    as u32;
                            } else {
                                self.vm_state.pc += 4;
                            }
                        }
                        _ => return Err((TaskError::InvalidOperation(self.vm_state.pc, op), ran)),
                    }
                }
                0b000111 => {
                    //BGTZ
                    if self.vm_state.reg[immediate_s!(op)] as i32 > 0 {
                        self.vm_state.pc = ((self.vm_state.pc as i32)
                            .wrapping_add(immediate_immediate_address!(op)))
                            as u32;
                    } else {
                        self.vm_state.pc += 4;
                    }
                }

                0b000110 => {
                    //BLEZ
                    if self.vm_state.reg[immediate_s!(op)] as i32 <= 0 {
                        self.vm_state.pc = ((self.vm_state.pc as i32)
                            .wrapping_add(immediate_immediate_address!(op)))
                            as u32;
                    } else {
                        self.vm_state.pc += 4;
                    }
                }
                0b000101 => {
                    //BNE
                    if self.vm_state.reg[immediate_s!(op)]
                        != self.vm_state.reg[immediate_t!(op) as usize]
                    {
                        self.vm_state.pc = ((self.vm_state.pc as i32)
                            .wrapping_add(immediate_immediate_address!(op)))
                            as u32;
                    } else {
                        self.vm_state.pc += 4;
                    }
                }

                //load unsinged instructions
                0b100010 => {
                    //LWL
                    let address = ((self.vm_state.reg[immediate_s!(op)] as i32)
                        .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                        as u32;
                    let reg_num = immediate_t!(op);
                    let thing: &mut [u8; 4] =
                        unsafe { core::mem::transmute(&mut self.vm_state.reg[reg_num]) };
                    thing[3] = get_mem_alligned!(address, u8); //self.mem.get_u8(address);
                    thing[2] = get_mem_alligned!(address.wrapping_add(1), u8);
                    //self.mem.get_u8(address + 1);
                }
                0b100110 => {
                    //LWR
                    let address = ((self.vm_state.reg[immediate_s!(op)] as i32)
                        .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                        as u32;
                    let reg_num = immediate_t!(op);
                    let thing: &mut [u8; 4] =
                        unsafe { core::mem::transmute(&mut self.vm_state.reg[reg_num]) };

                    thing[0] = get_mem_alligned!(address, u8); //self.mem.get_u8(address);
                    thing[1] = get_mem_alligned!(address.wrapping_sub(1), u8);
                    //self.mem.get_u8(address.wrapping_sub(1));
                }

                //save unaliged instructions
                0b101010 => {
                    //SWL
                    let address = ((self.vm_state.reg[immediate_s!(op)] as i32)
                        .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                        as u32;
                    let reg_num = immediate_t!(op);
                    let thing: [u8; 4] = self.vm_state.reg[reg_num].to_ne_bytes();

                    set_mem_alligned!(address, thing[3], u8);
                    set_mem_alligned!(address.wrapping_add(1), thing[2], u8);
                }
                0b101110 => {
                    //SWR
                    let address = ((self.vm_state.reg[immediate_s!(op)] as i32)
                        .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                        as u32;
                    let reg_num = immediate_t!(op);
                    let thing: [u8; 4] = self.vm_state.reg[reg_num].to_ne_bytes();

                    set_mem_alligned!(address, thing[0], u8);
                    set_mem_alligned!(address.wrapping_sub(1), thing[1], u8);
                }

                // load instrictions
                0b100000 => {
                    //LB
                    let address = ((self.vm_state.reg[immediate_s!(op)] as i32)
                        .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                        as u32;

                    self.vm_state.reg[immediate_t!(op)] = get_mem_alligned!(address, i8) as u32;
                }
                0b100100 => {
                    //LBU
                    let address = ((self.vm_state.reg[immediate_s!(op)] as i32)
                        .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                        as u32;

                    self.vm_state.reg[immediate_t!(op)] = get_mem_alligned!(address, u8) as u32;
                }
                0b100001 => {
                    //LH
                    let address = ((self.vm_state.reg[immediate_s!(op)] as i32)
                        .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                        as u32;

                    if likely(address & 0b1 == 0) {
                        self.vm_state.reg[immediate_t!(op)] =
                            get_mem_alligned!(address, i16) as u32;
                    //self.mem.get_i16_alligned(address) as u32
                    } else {
                        return Err((TaskError::MemoryAllignmentError(2, self.vm_state.pc), ran));
                    }
                }
                0b100101 => {
                    //LHU
                    let address = ((self.vm_state.reg[immediate_s!(op)] as i32)
                        .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                        as u32;

                    if likely(address & 0b1 == 0) {
                        self.vm_state.reg[immediate_t!(op)] =
                            get_mem_alligned!(address, u16) as u32;
                    //self.mem.get_u16_alligned(address) as u32
                    } else {
                        return Err((TaskError::MemoryAllignmentError(2, self.vm_state.pc), ran));
                    }
                }
                0b100011 => {
                    //LW
                    let address = ((self.vm_state.reg[immediate_s!(op)] as i32)
                        .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                        as u32;

                    if likely(address & 0b11 == 0) {
                        self.vm_state.reg[immediate_t!(op)] = get_mem_alligned!(address, u32);
                    //self.mem.get_u32_alligned(address) as u32
                    } else {
                        return Err((TaskError::MemoryAllignmentError(4, self.vm_state.pc), ran));
                    }
                }

                0b110000 => {
                    //LL
                    let address = ((self.vm_state.reg[immediate_s!(op)] as i32)
                        .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                        as u32;

                    if likely(address & 0b11 == 0) {
                        *mem.ll_bit = true;
                        self.vm_state.reg[immediate_t!(op)] = get_mem_alligned!(address, u32);
                    //self.mem.get_u32_alligned(address) as u32
                    } else {
                        return Err((TaskError::MemoryAllignmentError(4, self.vm_state.pc), ran));
                    }
                }
                0b111000 => {
                    //SC
                    let address = ((self.vm_state.reg[immediate_s!(op)] as i32)
                        .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                        as u32;

                    if likely(address & 0b11 == 0) {
                        if *mem.ll_bit {
                            set_mem_alligned!(address, self.vm_state.reg[immediate_t!(op)], u32);
                            self.vm_state.reg[immediate_t!(op)] = 1;
                        } else {
                            self.vm_state.reg[immediate_t!(op)] = 0;
                        }
                        *mem.ll_bit = false;
                    } else {
                        self.vm_state.reg[immediate_t!(op)] = 0;
                        return Err((TaskError::MemoryAllignmentError(4, self.vm_state.pc), ran));
                    }
                }

                // store instructions
                0b101000 => {
                    //SB
                    let address = ((self.vm_state.reg[immediate_s!(op)] as i32)
                        .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                        as u32;

                    *mem.ll_bit = false;
                    set_mem_alligned!(address, self.vm_state.reg[immediate_t!(op)] as u8, u8);
                }
                0b101001 => {
                    //SH
                    let address = ((self.vm_state.reg[immediate_s!(op)] as i32)
                        .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                        as u32;

                    if likely(address & 0b1 == 0) {
                        *mem.ll_bit = false;
                        set_mem_alligned!(address, self.vm_state.reg[immediate_t!(op)] as u16, u16);
                    } else {
                        return Err((TaskError::MemoryAllignmentError(2, self.vm_state.pc), ran));
                    }
                }
                0b101011 => {
                    //SW
                    let address = ((self.vm_state.reg[immediate_s!(op)] as i32)
                        .wrapping_add(immediate_immediate_signed_extended!(op) as i32))
                        as u32;
                    if likely(address & 0b11 == 0) {
                        *mem.ll_bit = false;
                        set_mem_alligned!(address, self.vm_state.reg[immediate_t!(op)], u32);
                    } else {
                        return Err((TaskError::MemoryAllignmentError(4, self.vm_state.pc), ran));
                    }
                }

                _ => return Err((TaskError::InvalidOperation(self.vm_state.pc, op), ran)),
            }
        }
        Ok(TaskRunResult::Continue)
    }
}
