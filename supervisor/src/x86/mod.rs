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

use core::{arch, mem};

#[cfg(target_arch = "x86")]
arch::global_asm!(include_str!("x86.S"));
#[cfg(target_arch = "x86_64")]
arch::global_asm!(include_str!("x86_64.S"));

//==================================================================================================
// Constants
//==================================================================================================

#[allow(non_snake_case)]
mod SegmentDescriptorAccess {
    pub const A: u8 = 1 << 0;
    pub const RW: u8 = 1 << 1;
    pub const DC: u8 = 1 << 2;
    pub const E: u8 = 1 << 3;
    pub const S: u8 = 1 << 4;
    pub const P: u8 = 1 << 7;
}

#[allow(non_snake_case)]
mod SegmentDescriptorFlags {
    pub const L: u8 = 1 << 5;
    pub const DB: u8 = 1 << 6;
    pub const G: u8 = 1 << 7;
}

//==================================================================================================
// Variables
//==================================================================================================

#[no_mangle]
static GDT: [SegmentDescriptor; 7] = [
    // NULL
    unsafe { SegmentDescriptor::zeroed() },
    // KCODE
    SegmentDescriptor::new(
        0x00000000,
        0xFFFFF,
        SegmentDescriptorAccess::A
            | SegmentDescriptorAccess::RW
            | SegmentDescriptorAccess::E
            | SegmentDescriptorAccess::S
            | SegmentDescriptorAccess::P,
        0,
        #[cfg(target_arch = "x86")]
        (SegmentDescriptorFlags::DB | SegmentDescriptorFlags::G),
        #[cfg(target_arch = "x86_64")]
        (SegmentDescriptorFlags::L | SegmentDescriptorFlags::G),
    ),
    // KDATA
    SegmentDescriptor::new(
        0x00000000,
        0xFFFFF,
        SegmentDescriptorAccess::A
            | SegmentDescriptorAccess::RW
            | SegmentDescriptorAccess::S
            | SegmentDescriptorAccess::P,
        0,
        SegmentDescriptorFlags::DB | SegmentDescriptorFlags::G,
    ),
    // UCODE
    SegmentDescriptor::new(
        0x00000000,
        0xFFFFF,
        SegmentDescriptorAccess::A
            | SegmentDescriptorAccess::RW
            | SegmentDescriptorAccess::E
            | SegmentDescriptorAccess::S
            | SegmentDescriptorAccess::P,
        3,
        #[cfg(target_arch = "x86")]
        (SegmentDescriptorFlags::DB | SegmentDescriptorFlags::G),
        #[cfg(target_arch = "x86_64")]
        (SegmentDescriptorFlags::L | SegmentDescriptorFlags::G),
    ),
    // UDATA
    SegmentDescriptor::new(
        0x00000000,
        0xFFFFF,
        SegmentDescriptorAccess::A
            | SegmentDescriptorAccess::RW
            | SegmentDescriptorAccess::S
            | SegmentDescriptorAccess::P,
        3,
        SegmentDescriptorFlags::DB | SegmentDescriptorFlags::G,
    ),
    // TSS
    unsafe { SegmentDescriptor::zeroed() },
    // TSS64 / GS
    unsafe { SegmentDescriptor::zeroed() },
];

//==================================================================================================
// Structures
//==================================================================================================

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

//==================================================================================================
// Implementations
//==================================================================================================

impl SegmentDescriptor {
    const unsafe fn zeroed() -> Self {
        mem::MaybeUninit::zeroed().assume_init()
    }

    const fn new(base: u32, limit: u32, access: u8, dpl: u8, flags: u8) -> Self {
        Self {
            limit_0_15: limit as u16,
            base_0_15: base as u16,
            base_16_23: (base >> 16) as u8,
            access: access | dpl << 5,
            flags_and_limit_16_19: (limit >> 16) as u8 | flags,
            base_24_31: (base >> 24) as u8,
        }
    }
}
