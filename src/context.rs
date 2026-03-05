use crate::NodeJsBuildpack;
use crate::cleanup::CleanupTask;
use std::cell::RefCell;
use std::ops::Deref;

/// Wrapper around libcnb `BuildContext` that tracks cleanup tasks
/// for non-deterministic build artifacts
pub(crate) struct NodeJsBuildContext {
    inner: libcnb::build::BuildContext<NodeJsBuildpack>,
    cleanup_tasks: RefCell<Vec<CleanupTask>>,
}

impl NodeJsBuildContext {
    pub(crate) fn new(inner: libcnb::build::BuildContext<NodeJsBuildpack>) -> Self {
        Self {
            inner,
            cleanup_tasks: RefCell::new(Vec::new()),
        }
    }

    pub(crate) fn register_cleanup(&self, task: CleanupTask) {
        self.cleanup_tasks.borrow_mut().push(task);
    }

    pub(crate) fn cleanup_tasks(&self) -> Vec<CleanupTask> {
        self.cleanup_tasks.borrow().clone()
    }
}

impl Deref for NodeJsBuildContext {
    type Target = libcnb::build::BuildContext<NodeJsBuildpack>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
