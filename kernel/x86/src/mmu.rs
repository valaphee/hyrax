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

use core::{marker, ops, ptr};

use kernel::mmu::*;

pub struct AddressSpaceX86(PhysicalAddress);

impl AddressSpace for AddressSpaceX86 {
    type Entry = PageTableEntryX86;

    fn iter(
        &self,
        address_range: impl ops::RangeBounds<VirtualAddress>,
    ) -> impl Iterator<Item = &Self::Entry> {
        let address_begin = match address_range.start_bound() {
            ops::Bound::Included(value) => *value,
            ops::Bound::Excluded(value) => *value + 1,
            ops::Bound::Unbounded => 0,
        };
        let address_end = match address_range.end_bound() {
            ops::Bound::Included(value) => *value + 1,
            ops::Bound::Excluded(value) => *value,
            ops::Bound::Unbounded => todo!(),
        };

        let next = [ptr::null(); 4];
        let next_last = [ptr::null(); 4];
        let last = [0; 3];

        AddressSpaceIter {
            this: Default::default(),
            next,
            next_last,
            last,
        }
    }

    fn iter_mut(
        &mut self,
        address_range: impl ops::RangeBounds<VirtualAddress>,
    ) -> impl Iterator<Item = &mut Self::Entry> {
        let mut address_begin = match address_range.start_bound() {
            ops::Bound::Included(value) => *value,
            ops::Bound::Excluded(value) => *value + 1,
            ops::Bound::Unbounded => 0,
        };
        let mut address_end = match address_range.end_bound() {
            ops::Bound::Included(value) => *value + 1,
            ops::Bound::Excluded(value) => *value,
            ops::Bound::Unbounded => todo!(),
        };

        let next = [ptr::null_mut(); 4];
        let next_last = [ptr::null_mut(); 4];
        let last = [0; 3];

        AddressSpaceIterMut {
            this: Default::default(),
            next,
            next_last,
            last,
        }
    }
}

struct AddressSpaceIter<'this> {
    this: marker::PhantomData<&'this ()>,
    next: [*const PageTableEntryX86; 4],
    next_last: [*const PageTableEntryX86; 4],
    last: [usize; 3],
}

impl<'this> Iterator for AddressSpaceIter<'this> {
    type Item = &'this PageTableEntryX86;

    fn next(&mut self) -> Option<Self::Item> {
        for level in 0..4 {
            if !self.next[level].is_null() {
                let value = unsafe { &*self.next[level] };
                self.next[level] = unsafe { self.next[level].add(1) };
                if self.next[level] > self.next_last[level] {
                    self.next[level] = ptr::null_mut();
                }
                if level != 0 && value.is_table() {
                    self.next[level - 1] = value.address() as *mut PageTableEntryX86;
                    self.next_last[level - 1] = unsafe {
                        self.next[level - 1].add(if self.next[level].is_null() {
                            self.last[level]
                        } else {
                            512
                        })
                    }
                }
                return Some(value);
            }
        }
        None
    }
}

struct AddressSpaceIterMut<'this> {
    this: marker::PhantomData<&'this ()>,
    next: [*mut PageTableEntryX86; 4],
    next_last: [*mut PageTableEntryX86; 4],
    last: [usize; 3],
}

impl<'this> Iterator for AddressSpaceIterMut<'this> {
    type Item = &'this mut PageTableEntryX86;

    fn next(&mut self) -> Option<Self::Item> {
        for level in 0..4 {
            if !self.next[level].is_null() {
                let value = unsafe { &mut *self.next[level] };
                self.next[level] = unsafe { self.next[level].add(1) };
                if self.next[level] > self.next_last[level] {
                    self.next[level] = ptr::null_mut();
                }
                if level != 0 {
                    if !value.is_valid() {
                    } else if value.is_table() {
                        self.next[level - 1] = value.address() as *mut PageTableEntryX86;
                        self.next_last[level - 1] = unsafe {
                            self.next[level - 1].add(if self.next[level].is_null() {
                                self.last[level]
                            } else {
                                512
                            })
                        }
                    }
                }
                return Some(value);
            }
        }
        None
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntryX86(usize);

impl PageTableEntry for PageTableEntryX86 {
    fn address(&self) -> PhysicalAddress {
        self.0 >> 12 << 12
    }

    fn is_valid(&self) -> bool {
        const P: usize = 1 << 0;
        return self.0 & P != 0;
    }

    fn is_table(&self) -> bool {
        const PS: usize = 1 << 7;
        return self.is_valid() && self.0 & PS == 0;
    }
}