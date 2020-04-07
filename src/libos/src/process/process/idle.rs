use super::super::task::Task;
use super::super::thread::ThreadId;
use super::{ProcessBuilder, ThreadRef};
/// Process 0, a.k.a, the idle process.
///
/// The idle process has no practical use except making process 1 (a.k.a, the init proess)
/// having a parent.
use crate::prelude::*;
use crate::vm::ProcessVM;

lazy_static! {
    pub static ref IDLE: ThreadRef =
        { create_idle_thread().expect("creating the idle process should never fail") };
}

fn create_idle_thread() -> Result<ThreadRef> {
    // Create dummy values for the mandatory fields
    let dummy_tid = ThreadId::zero();
    let dummy_vm = Arc::new(SgxMutex::new(ProcessVM::default()));
    let dummy_task = Task::default();

    // Assemble the idle process
    let idle_process = ProcessBuilder::new()
        .tid(dummy_tid)
        .vm(dummy_vm)
        .task(dummy_task)
        .no_parent(true)
        .build()?;
    debug_assert!(idle_process.pid() == 0);

    let idle_thread = idle_process.main_thread().unwrap();
    debug_assert!(idle_thread.tid() == 0);

    // We do not add the idle process/thread to the process/thread table.
    // This ensures that the idle process is not accessible from the user space.

    Ok(idle_thread)
}
