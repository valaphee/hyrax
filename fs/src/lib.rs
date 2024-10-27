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

pub use hyrax_err::*;

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

pub trait FileSystem {
    fn stat(&self, index: u64, buffer: &mut [u8]) -> Result<()>;

    fn read(&self, index: u64, offset: u64, buffer: &mut [u8]) -> Result<()>;

    fn write(&self, index: u64, offset: u64, buffer: &[u8]) -> Result<()>;
}

pub struct FileSystemClient {}

impl FileSystem for FileSystemClient {
    fn stat(&self, index: u64, buffer: &mut [u8]) -> Result<()> {
        Err(Error::Unimplemented)
    }

    fn read(&self, index: u64, offset: u64, buffer: &mut [u8]) -> Result<()> {
        Err(Error::Unimplemented)
    }

    fn write(&self, index: u64, offset: u64, buffer: &[u8]) -> Result<()> {
        Err(Error::Unimplemented)
    }
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct Entry {
    pub index: u64,
    pub data_length: u64,
    pub name_offset: u32,
    pub name_length: u8,
    pub padding: [u8; 3],
}
