//Basic mips stuff

/// Does not take reguments nor return anything
///
/// Halts the VM
pub const HALT: u32 = 0;

/// Print a 2's complement i32 to standard output
///
/// Register 4: i32 value
pub const PRINT_DEC_NUMBER: u32 = 1;

/// Print a C-String ending in a \0 byte.
///
/// Register 4: ptr to begining of string
pub const PRINT_C_STRING: u32 = 4;

/// Print a char to standard output
///
/// Register 4: the char to print
pub const PRINT_CHAR: u32 = 5;

/// Sleep for x ms
///
/// Register 4: the number of ms to sleep for
pub const SLEEP_MS: u32 = 50;

/// Sleep for delta x ms
///
/// Register 4: the number of ms to sleep for munis the time it took since the last call
pub const SLEEP_D_MS: u32 = 51;

/// Current time nanos
///
/// Register 2: lower half of nanos
/// Register 3: upper half of nanos
pub const CURRENT_TIME_NANOS: u32 = 60;

/// Generate a random number between xi32 and yi32
///
/// Register 4: xi32 lower bound
/// Register 4: yi32 upper bound
///
/// Register 2: generated random number
pub const GENERATE_THREAD_RANDOM_NUMBER: u32 = 99;

/// Start a new thread
///
/// Register 4: Pointer to thread entry
/// Register 5: Pointer to thread arguments
///
/// Register 2: Non zero Id of created thread (if zero an error occured)
pub const START_NEW_THREAD: u32 = 100;

/// Sleep nanoseconds
///
/// Register 4/5: number of nano seconds to sleep for
pub const SLEEP_NANOS: u32 = 101;

/// Basically stop exicuting until the scheduler decides this task should run next
pub const WAIT_CONTINUE: u32 = 102;

/// Wake n tasks for the futex
///
/// Register 4: Pointer to the futex
/// Register 5: Number of tasks to wake
///
/// Register 2: Number of tasks actually woken
pub const FUTEX_WAKE: u32 = 200;

/// Sleep on the futex condition
///
/// Register 4: Pointer to the futex
/// Register 5: The condition the futex must be in to wait
///
/// Register 2: 1 if the condition was met, 0 otherwise
pub const FUTEX_WAIT: u32 = 201;
