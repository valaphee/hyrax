// Copyright 2024 Kevin Ludwig
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//==================================================================================================
// Imports
//==================================================================================================

use intrusive_collections::{intrusive_adapter, Bound, KeyAdapter, RBTree, RBTreeLink, UnsafeRef};

use crate::memory::ObjectPool;

//======================================================================================================================
// Variables
//======================================================================================================================

static SYSTEM_MEMORY_FREE_POOL: ObjectPool<SystemMemoryFree> = ObjectPool::new();

//==================================================================================================
// Structures
//==================================================================================================

/// System memory bookkeeping
///
/// Doesn't retain any information about the allocations themself.
pub struct SystemMemory {
    free: RBTree<SystemMemoryFreeAdapter>,
}

struct SystemMemoryFree {
    link: RBTreeLink,
    addr: usize,
    size: usize,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl SystemMemory {
    /// Creates a new system memory bookkeeping structure with the default state
    /// being that there is no available memory.
    pub const fn new() -> Self {
        Self {
            free: RBTree::new(SystemMemoryFreeAdapter::NEW),
        }
    }

    /// Allocates a chunk of memory which must be deallocated to be available
    /// again.
    ///
    /// The operation may return `None` if the requested `addr` is not available
    /// or not large enough to encompass `size`.
    pub fn allocate(&mut self, addr: Option<usize>, size: usize) -> Option<usize> {
        if let Some(addr) = addr {
            let mut cursor = self.free.upper_bound_mut(Bound::Included(&addr));
            let chunk = cursor.get()?;
            if chunk.addr + chunk.size >= addr + size {
                let mut chunk = unsafe { &mut *UnsafeRef::into_raw(cursor.remove().unwrap()) };

                // trim before
                if chunk.addr != addr {
                    let mut chunk_size = chunk.size;
                    chunk.size = addr - chunk.addr;
                    chunk_size -= chunk.size;
                    cursor.insert_before(unsafe { UnsafeRef::from_raw(chunk) });

                    chunk = SYSTEM_MEMORY_FREE_POOL.allocate(SystemMemoryFree {
                        link: Default::default(),
                        addr: addr + size,
                        size: chunk_size,
                    });
                }

                // trim after
                chunk.size -= size;
                if chunk.size != 0 {
                    chunk.addr = addr + size;
                    cursor.insert_before(unsafe { UnsafeRef::from_raw(chunk) });
                } else {
                    SYSTEM_MEMORY_FREE_POOL.deallocate(chunk);
                }

                return Some(addr);
            }
        } else {
            let mut cursor = self.free.lower_bound_mut(Bound::Unbounded);
            while let Some(chunk) = cursor.get() {
                // first-fit
                if chunk.size >= size {
                    let chunk = unsafe { &mut *UnsafeRef::into_raw(cursor.remove().unwrap()) };
                    chunk.size -= size;
                    let addr = chunk.addr + chunk.size;
                    if chunk.size != 0 {
                        cursor.insert_before(unsafe { UnsafeRef::from_raw(chunk) });
                    } else {
                        SYSTEM_MEMORY_FREE_POOL.deallocate(chunk);
                    }
                    return Some(addr);
                }
                cursor.move_prev();
            }
        }

        None
    }

    /// Deallocates a chunk of memory and makes it available to subsequent
    /// `allocate` operations.
    pub fn deallocate(&mut self, addr: usize, size: usize) {
        let mut cursor = self.free.upper_bound_mut(Bound::Included(&addr));

        // coalesce before
        if let Some(chunk) = cursor.get() {
            if chunk.addr + chunk.size == addr {
                let chunk = unsafe { &mut *UnsafeRef::into_raw(cursor.remove().unwrap()) };
                chunk.size += size;

                // coalesce in-between
                if let Some(next_chunk) = cursor.get() {
                    if next_chunk.addr == chunk.addr + chunk.size {
                        let next_chunk = cursor.remove().unwrap();
                        chunk.size += next_chunk.size;
                    }
                }

                cursor.insert_before(unsafe { UnsafeRef::from_raw(chunk) });
                return;
            }
        }

        // coalesce after
        if let Some(chunk) = cursor.peek_next().get() {
            if chunk.addr == addr + size {
                cursor.move_next();
                let chunk = unsafe { &mut *UnsafeRef::into_raw(cursor.remove().unwrap()) };
                chunk.addr = addr;
                chunk.size += size;
                cursor.insert_before(unsafe { UnsafeRef::from_raw(chunk) });
                return;
            }
        }

        let chunk = SYSTEM_MEMORY_FREE_POOL.allocate(SystemMemoryFree {
            link: Default::default(),
            addr,
            size,
        });
        cursor.insert(unsafe { UnsafeRef::from_raw(chunk) });
    }
}

//==================================================================================================
// Trait Implementations
//==================================================================================================

intrusive_adapter!(SystemMemoryFreeAdapter = UnsafeRef<SystemMemoryFree>: SystemMemoryFree { link: RBTreeLink });

impl KeyAdapter<'_> for SystemMemoryFreeAdapter {
    type Key = usize;

    fn get_key(
        &self,
        value: &'_ <Self::PointerOps as intrusive_collections::PointerOps>::Value,
    ) -> Self::Key {
        value.addr
    }
}
