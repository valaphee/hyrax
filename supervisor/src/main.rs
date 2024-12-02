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

use core::{hint, panic};

mod memory;
mod process;
mod x86;

//==================================================================================================
// Functions
//==================================================================================================

#[no_mangle]
extern "C" fn main(_multiboot_magic: u32, _multiboot_info: u32) -> ! {
    loop {
        hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &panic::PanicInfo) -> ! {
    loop {
        hint::spin_loop();
    }
}
