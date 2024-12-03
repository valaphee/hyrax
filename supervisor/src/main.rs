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

#![no_std]
#![no_main]

//==================================================================================================
// Imports
//==================================================================================================

use core::{hint, panic, slice};

use memory::SystemMemory;
use multiboot::{multiboot_mmap_entry, MULTIBOOT_MEMORY_AVAILABLE};
use zerocopy::FromBytes;

mod memory;
mod process;
mod x86;

//==================================================================================================
// Functions
//==================================================================================================

#[no_mangle]
extern "C" fn main(multiboot_magic: u32, multiboot_info: u32) -> ! {
    let mut system_memory = SystemMemory::new();

    assert!(multiboot_magic == multiboot::MULTIBOOT_BOOTLOADER_MAGIC);
    let multiboot_info = unsafe { &*(multiboot_info as usize as *const multiboot::multiboot_info) };

    assert!(multiboot_info.flags & multiboot::MULTIBOOT_INFO_MEM_MAP != 0);
    let mut multiboot_mmap = unsafe {
        slice::from_raw_parts(
            multiboot_info.mmap_addr as usize as *const u8,
            multiboot_info.mmap_length as usize,
        )
    };
    while !multiboot_mmap.is_empty() {
        let multiboot_mmap_entry = multiboot_mmap_entry::ref_from_prefix(multiboot_mmap)
            .unwrap()
            .0;
        multiboot_mmap = &multiboot_mmap[multiboot_mmap_entry.size as usize + 4..];
        if multiboot_mmap_entry.type_ != MULTIBOOT_MEMORY_AVAILABLE {
            continue;
        }

        system_memory.deallocate(
            multiboot_mmap_entry.addr as usize,
            multiboot_mmap_entry.size as usize,
        );
    }

    loop {
        hint::spin_loop();
    }
}

#[no_mangle]
extern "C" fn main_other() -> ! {
    loop {
        hint::spin_loop();
    }
}

#[panic_handler]
fn panic(info: &panic::PanicInfo) -> ! {
    loop {
        hint::spin_loop();
    }
}
