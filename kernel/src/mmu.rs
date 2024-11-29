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

use core::{cell, marker, mem, ops};

type PhysicalAddress = usize;
type VirtualAddress = usize;

pub struct AddressSpace(PhysicalAddress);

impl AddressSpace {
    fn iter(
        &self,
        address_range: impl ops::RangeBounds<VirtualAddress>,
    ) -> impl Iterator<Item = &PageTableEntry> {
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

        AddressSpaceIter {
            this: Default::default(),
        }
    }

    fn iter_mut(
        &mut self,
        address_range: impl ops::RangeBounds<VirtualAddress>,
    ) -> impl Iterator<Item = &mut PageTableEntry> {
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

        AddressSpaceIterMut {
            this: Default::default(),
        }
    }
}

struct AddressSpaceIter<'this> {
    this: marker::PhantomData<&'this ()>,
}

impl<'this> Iterator for AddressSpaceIter<'this> {
    type Item = &'this PageTableEntry;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

struct AddressSpaceIterMut<'this> {
    this: marker::PhantomData<&'this ()>,
}

impl<'this> Iterator for AddressSpaceIterMut<'this> {
    type Item = &'this mut PageTableEntry;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry(usize);

impl PageTableEntry {}

#[no_mangle]
static GDT: cell::SyncUnsafeCell<[SegmentDescriptor; 7]> = cell::SyncUnsafeCell::new([
    // NULL
    unsafe { SegmentDescriptor::zeroed() },
    // KCODE
    SegmentDescriptor::new(
        0x00000000,
        0xFFFFF,
        SegmentDescriptorAccess::A
            .union(SegmentDescriptorAccess::RW)
            .union(SegmentDescriptorAccess::E)
            .union(SegmentDescriptorAccess::S)
            .union(SegmentDescriptorAccess::P),
        0,
        #[cfg(target_arch = "x86")]
        SegmentDescriptorFlags::DB.union(SegmentDescriptorFlags::G),
        #[cfg(target_arch = "x86_64")]
        SegmentDescriptorFlags::L.union(SegmentDescriptorFlags::G),
    ),
    // KDATA
    SegmentDescriptor::new(
        0x00000000,
        0xFFFFF,
        SegmentDescriptorAccess::A
            .union(SegmentDescriptorAccess::RW)
            .union(SegmentDescriptorAccess::S)
            .union(SegmentDescriptorAccess::P),
        0,
        SegmentDescriptorFlags::DB.union(SegmentDescriptorFlags::G),
    ),
    // UCODE
    SegmentDescriptor::new(
        0x00000000,
        0xFFFFF,
        SegmentDescriptorAccess::A
            .union(SegmentDescriptorAccess::RW)
            .union(SegmentDescriptorAccess::E)
            .union(SegmentDescriptorAccess::S)
            .union(SegmentDescriptorAccess::P),
        3,
        #[cfg(target_arch = "x86")]
        SegmentDescriptorFlags::DB.union(SegmentDescriptorFlags::G),
        #[cfg(target_arch = "x86_64")]
        SegmentDescriptorFlags::L.union(SegmentDescriptorFlags::G),
    ),
    // UDATA
    SegmentDescriptor::new(
        0x00000000,
        0xFFFFF,
        SegmentDescriptorAccess::A
            .union(SegmentDescriptorAccess::RW)
            .union(SegmentDescriptorAccess::S)
            .union(SegmentDescriptorAccess::P),
        3,
        SegmentDescriptorFlags::DB.union(SegmentDescriptorFlags::G),
    ),
    // TSS
    unsafe { SegmentDescriptor::zeroed() },
    // TSS64 / GS
    unsafe { SegmentDescriptor::zeroed() },
]);

#[repr(C, packed(2))]
struct SegmentDescriptorTableRegister {
    size: u16,
    offset: *mut [SegmentDescriptor],
}

#[repr(C)]
struct SegmentDescriptor {
    limit_0_15: u16,
    base_0_15: u16,
    base_16_23: u8,
    access: u8,
    flags_and_limit_16_19: u8,
    base_24_31: u8,
}

impl SegmentDescriptor {
    const unsafe fn zeroed() -> Self {
        mem::MaybeUninit::zeroed().assume_init()
    }

    const fn new(
        base: u32,
        limit: u32,
        access: SegmentDescriptorAccess,
        dpl: u8,
        flags: SegmentDescriptorFlags,
    ) -> Self {
        Self {
            limit_0_15: limit as u16,
            base_0_15: base as u16,
            base_16_23: (base >> 16) as u8,
            access: access.bits() | dpl << 5,
            flags_and_limit_16_19: (limit >> 16) as u8 | flags.bits(),
            base_24_31: (base >> 24) as u8,
        }
    }
}

bitflags::bitflags! {
    struct SegmentDescriptorAccess: u8 {
        const A = 1 << 0;
        const RW = 1 << 1;
        const DC = 1 << 2;
        const E = 1 << 3;
        const S = 1 << 4;
        const P = 1 << 7;
    }

    #[derive(Copy, Clone, Debug)]
    struct SegmentDescriptorFlags: u8 {
        const L = 1 << 5;
        const DB = 1 << 6;
        const G = 1 << 7;
    }
}
