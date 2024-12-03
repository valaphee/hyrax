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

use intrusive_collections::{intrusive_adapter, KeyAdapter, RBTree, RBTreeLink, UnsafeRef};

use crate::memory::{Mapping, ObjectPool};

//==================================================================================================
// Variables
//==================================================================================================

static PROCESS_MEMORY_USED_POOL: ObjectPool<ProcessMemoryUsed> = ObjectPool::new();

//==================================================================================================
// Structures
//==================================================================================================

pub struct ProcessMemory {
    used: RBTree<ProcessMemoryUsedAdapter>,

    mapping: Mapping,
}

struct ProcessMemoryUsed {
    link: RBTreeLink,
    addr: usize,
    size: usize,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl ProcessMemory {
    pub fn allocate(&mut self, addr: Option<usize>, size: usize) -> Option<usize> {
        None
    }

    pub fn deallocate(&mut self, addr: usize, size: usize) {}
}

//==================================================================================================
// Trait Implementations
//==================================================================================================

intrusive_adapter!(ProcessMemoryUsedAdapter = UnsafeRef<ProcessMemoryUsed>: ProcessMemoryUsed { link: RBTreeLink });

impl KeyAdapter<'_> for ProcessMemoryUsedAdapter {
    type Key = usize;

    fn get_key(
        &self,
        value: &'_ <Self::PointerOps as intrusive_collections::PointerOps>::Value,
    ) -> Self::Key {
        value.addr
    }
}
