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

use core::marker;

//==================================================================================================
// Structures
//==================================================================================================

pub struct ObjectPool<T> {
    r#type: marker::PhantomData<T>,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl<T> ObjectPool<T> {
    pub const fn new() -> Self {
        Self {
            r#type: marker::PhantomData,
        }
    }

    pub fn allocate(&mut self, value: T) -> &mut T {}

    pub fn deallocate(&mut self, reference: &mut T) {}
}
