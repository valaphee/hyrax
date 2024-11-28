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
#![feature(sync_unsafe_cell)]

use core::{arch, hint};

mod cpu;
mod mmu;

#[cfg(target_arch = "x86")]
arch::global_asm!(include_str!("main32.S"));
#[cfg(target_arch = "x86_64")]
arch::global_asm!(include_str!("main64.S"));

#[no_mangle]
extern "C" fn main(multiboot_magic: u32, multiboot_info: u32) -> ! {
    loop {
        hint::spin_loop();
    }
}
