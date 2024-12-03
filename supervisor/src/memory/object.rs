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

use spin::Mutex;

//==================================================================================================
// Structures
//==================================================================================================

/// Pool of objects
pub struct ObjectPool<T>(Mutex<[Option<T>; 32]>);

//==================================================================================================
// Implementations
//==================================================================================================

impl<T> ObjectPool<T> {
    /// Creates a new memory pool.
    pub const fn new() -> Self {
        Self(Mutex::new([
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None, None, None,
        ]))
    }

    /// Allocates an object from the pool which must be deallocated to be
    /// available again.
    #[allow(clippy::mut_from_ref)]
    pub fn allocate(&self, value: T) -> &mut T {
        let mut pool = self.0.lock();
        let entry = pool.iter_mut().find(|entry| entry.is_none()).unwrap();
        *entry = Some(value);
        unsafe { &mut *(entry.as_mut().unwrap() as *mut _) }
    }

    /// Deallocates an object and makes it available to subsequent
    /// `allocate` operations.
    pub fn deallocate(&self, reference: &mut T) {}
}

//==================================================================================================
// Trait Implementations
//==================================================================================================

unsafe impl<T> Sync for ObjectPool<T> {}
