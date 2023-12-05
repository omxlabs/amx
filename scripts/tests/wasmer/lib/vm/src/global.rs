use crate::{store::MaybeInstanceOwned, vmcontext::VMGlobalDefinition};
use std::{cell::UnsafeCell, ptr::NonNull};
use wasmer_types::GlobalType;

/// A Global instance
pub struct VMGlobal {
    ty: GlobalType,
    vm_global_definition: MaybeInstanceOwned<VMGlobalDefinition>,
}

impl VMGlobal {
    /// Create a new, zero bit-pattern initialized global from a [`GlobalType`].
    pub fn new(global_type: GlobalType) -> Self {
        Self {
            ty: global_type,
            // TODO: Currently all globals are host-owned, we should inline the
            // VMGlobalDefinition in VMContext for instance-defined globals.
            vm_global_definition: MaybeInstanceOwned::Host(Box::new(UnsafeCell::new(
                VMGlobalDefinition::new(),
            ))),
        }
    }

    /// Get the type of the global.
    pub fn ty(&self) -> &GlobalType {
        &self.ty
    }

    /// Get a pointer to the underlying definition used by the generated code.
    pub fn vmglobal(&self) -> NonNull<VMGlobalDefinition> {
        self.vm_global_definition.as_ptr()
    }
}
