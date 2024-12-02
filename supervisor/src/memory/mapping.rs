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

use core::{marker, ops};

//==================================================================================================
// Structures
//==================================================================================================

pub struct Mapping {}

pub struct MappingIter<'this> {
    this: marker::PhantomData<&'this ()>,
}

pub struct MappingIterMut<'this> {
    this: marker::PhantomData<&'this ()>,
}

pub struct PageTableEntry(usize);

//==================================================================================================
// Implementations
//==================================================================================================

impl Mapping {
    fn iter(&self, addr: impl ops::RangeBounds<usize>) -> MappingIter {
        MappingIter {
            this: Default::default(),
        }
    }

    fn iter_mut(&self, addr: impl ops::RangeBounds<usize>) -> MappingIterMut {
        MappingIterMut {
            this: Default::default(),
        }
    }
}

impl PageTableEntry {}

//==================================================================================================
// Trait Implementations
//==================================================================================================

impl<'this> Iterator for MappingIter<'this> {
    type Item = &'this PageTableEntry;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl<'this> Iterator for MappingIterMut<'this> {
    type Item = &'this mut PageTableEntry;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
