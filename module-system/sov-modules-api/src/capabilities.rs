#![deny(missing_docs)]

//! The rollup capabilities module defines "capabilities" that rollup must
//! provide if they wish to use the standard app template.
//! If you don't want to provide these capabilities,
//! you can bypass the Sovereign module-system completely
//! and write a state transition function from scratch.
//! [See here for docs](https://github.com/Sovereign-Labs/sovereign-sdk/blob/nightly/examples/demo-stf/README.md)

use sov_modules_core::{Context, WorkingSet};
use sov_rollup_interface::da::{BlobReaderTrait, DaSpec};

/// Container type for mixing borrowed and owned blobs.
#[derive(Debug)]
pub enum BlobRefOrOwned<'a, B: BlobReaderTrait> {
    /// Mutable reference
    Ref(&'a mut B),
    /// Owned blob
    Owned(B),
}

impl<'a, B: BlobReaderTrait> AsRef<B> for BlobRefOrOwned<'a, B> {
    fn as_ref(&self) -> &B {
        match self {
            BlobRefOrOwned::Ref(r) => r,
            BlobRefOrOwned::Owned(blob) => blob,
        }
    }
}

impl<'a, B: BlobReaderTrait> BlobRefOrOwned<'a, B> {
    /// Convenience method to get mutable reference to the blob
    pub fn as_mut_ref(&mut self) -> &mut B {
        match self {
            BlobRefOrOwned::Ref(r) => r,
            BlobRefOrOwned::Owned(ref mut blob) => blob,
        }
    }
}

impl<'a, B: BlobReaderTrait> From<B> for BlobRefOrOwned<'a, B> {
    fn from(value: B) -> Self {
        BlobRefOrOwned::Owned(value)
    }
}

impl<'a, B: BlobReaderTrait> From<&'a mut B> for BlobRefOrOwned<'a, B> {
    fn from(value: &'a mut B) -> Self {
        BlobRefOrOwned::Ref(value)
    }
}

/// The kernel is responsible for managing the inputs to the `apply_blob` method.
/// A simple implementation will simply process all blobs in the order that they appear,
/// while a second will support a "preferred sequencer" with some limited power to reorder blobs
/// in order to give out soft confirmations.
pub trait Kernel<C: Context, Da: DaSpec>: BlobSelector<Da, Context = C> + Default {}

/// BlobSelector decides which blobs to process in a current slot.
pub trait BlobSelector<Da: DaSpec> {
    /// Context type
    type Context: Context;

    /// It takes two arguments.
    /// 1. `current_blobs` - blobs that were received from the network for the current slot.
    /// 2. `working_set` - the working to access storage.
    /// It returns a vector containing a mix of borrowed and owned blobs.
    fn get_blobs_for_this_slot<'a, I>(
        &self,
        current_blobs: I,
        working_set: &mut WorkingSet<Self::Context>,
    ) -> anyhow::Result<Vec<BlobRefOrOwned<'a, Da::BlobTransaction>>>
    where
        I: IntoIterator<Item = &'a mut Da::BlobTransaction>;
}
