use crate::NodeJsBuildpack;
use crate::layer_cleanup::LayerCleanupTarget;
use std::cell::RefCell;
use std::ops::Deref;

/// Wrapper around libcnb `BuildContext` that tracks layers needing cleanup
/// of non-deterministic build artifacts (Python bytecode, Makefiles)
pub(crate) struct NodeJsBuildContext {
    inner: libcnb::build::BuildContext<NodeJsBuildpack>,
    cleanup_registry: RefCell<Vec<LayerCleanupTarget>>,
}

impl NodeJsBuildContext {
    pub(crate) fn new(inner: libcnb::build::BuildContext<NodeJsBuildpack>) -> Self {
        Self {
            inner,
            cleanup_registry: RefCell::new(Vec::new()),
        }
    }

    pub(crate) fn register_layer_for_cleanup(&self, target: LayerCleanupTarget) {
        self.cleanup_registry.borrow_mut().push(target);
    }

    pub(crate) fn layers_to_cleanup(&self) -> Vec<LayerCleanupTarget> {
        self.cleanup_registry.borrow().iter().cloned().collect()
    }
}

impl Deref for NodeJsBuildContext {
    type Target = libcnb::build::BuildContext<NodeJsBuildpack>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
