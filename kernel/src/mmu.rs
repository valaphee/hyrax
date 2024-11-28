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

use core::ops;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

pub trait AddressSpace {
    type Entry: PageTableEntry;

    fn iter(
        &self,
        address_range: impl ops::RangeBounds<VirtualAddress>,
    ) -> impl Iterator<Item = &Self::Entry>;

    fn iter_mut(
        &mut self,
        address_range: impl ops::RangeBounds<VirtualAddress>,
    ) -> impl Iterator<Item = &mut Self::Entry>;
}

pub trait PageTableEntry {
    fn address(&self) -> PhysicalAddress;

    fn is_valid(&self) -> bool;

    fn is_table(&self) -> bool;
}
