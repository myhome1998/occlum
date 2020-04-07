use super::super::task::Task;
use super::super::thread::{ThreadBuilder, ThreadId};
use super::super::{FileTableRef, FsViewRef, ProcessRef, ProcessVMRef, ResourceLimitsRef};
use super::{Process, ProcessInner};
use crate::prelude::*;

#[derive(Debug)]
pub struct ProcessBuilder {
    tid: Option<ThreadId>,
    thread_builder: Option<ThreadBuilder>,
    // Mandatory fields
    vm: Option<ProcessVMRef>,
    // Optional fields, which have reasonable default values
    exec_path: Option<String>,
    parent: Option<ProcessRef>,
    no_parent: bool,
}

impl ProcessBuilder {
    pub fn new() -> Self {
        let thread_builder = ThreadBuilder::new();
        Self {
            tid: None,
            thread_builder: Some(thread_builder),
            vm: None,
            exec_path: None,
            parent: None,
            no_parent: false,
        }
    }

    pub fn tid(mut self, tid: ThreadId) -> Self {
        self.tid = Some(tid);
        self
    }

    pub fn exec_path(mut self, exec_path: &str) -> Self {
        self.exec_path = Some(exec_path.to_string());
        self
    }

    pub fn parent(mut self, parent: ProcessRef) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn no_parent(mut self, no_parent: bool) -> Self {
        self.no_parent = no_parent;
        self
    }

    pub fn task(mut self, task: Task) -> Self {
        self.thread_builder(|tb| tb.task(task))
    }

    pub fn vm(mut self, vm: ProcessVMRef) -> Self {
        self.thread_builder(|tb| tb.vm(vm))
    }

    pub fn fs(mut self, fs: FsViewRef) -> Self {
        self.thread_builder(|tb| tb.fs(fs))
    }

    pub fn files(mut self, files: FileTableRef) -> Self {
        self.thread_builder(|tb| tb.files(files))
    }

    pub fn rlimits(mut self, rlimits: ResourceLimitsRef) -> Self {
        self.thread_builder(|tb| tb.rlimits(rlimits))
    }

    pub fn build(mut self) -> Result<ProcessRef> {
        // Process's pid == Main thread's tid
        let tid = self.tid.take().unwrap_or_else(|| ThreadId::new());
        let pid = tid.as_u32() as pid_t;

        // Check whether parent is given as expected
        if self.no_parent != self.parent.is_none() {
            return_errno!(
                EINVAL,
                "parent and no_parent config contradicts with one another"
            );
        }

        // Build a new process
        let new_process = {
            let exec_path = self.exec_path.take().unwrap_or_default();
            let parent = self.parent.take().map(|parent| SgxRwLock::new(parent));
            let inner = SgxMutex::new(ProcessInner::new());
            Arc::new(Process {
                pid,
                exec_path,
                parent,
                inner,
            })
        };

        // Build the main thread of the new process
        let mut self_ = self.thread_builder(|tb| tb.tid(tid).process(new_process.clone()));
        let main_thread = self_.thread_builder.take().unwrap().build()?;

        // Associate the new process with its parent
        if !self_.no_parent {
            new_process
                .parent()
                .inner()
                .children_mut()
                .unwrap()
                .push(new_process.clone());
        }

        Ok(new_process)
    }

    fn thread_builder<F>(mut self, f: F) -> Self
    where
        F: FnOnce(ThreadBuilder) -> ThreadBuilder,
    {
        let thread_builder = self.thread_builder.take().unwrap();
        self.thread_builder = Some(f(thread_builder));
        self
    }
}
