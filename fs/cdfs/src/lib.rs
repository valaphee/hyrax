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

use hyrax_ds::DataStorage;
use hyrax_fs::{Error, FileSystem, Result};

pub struct FileSystemServer<DS: DataStorage> {
    data_storage: DS,
}

impl<DS: DataStorage> FileSystemServer<DS> {
    pub fn new(data_storage: DS) -> Result<Self> {
        Ok(Self { data_storage })
    }
}

impl<DS: DataStorage> FileSystem for FileSystemServer<DS> {
    fn stat(&self, index: u64, offset: u64, buffer: &mut [u8]) -> Result<u64> {
        return Err(Error::Unimplemented);
    }

    fn read(&self, index: u64, offset: u64, buffer: &mut [u8]) -> Result<()> {
        return Err(Error::Unimplemented);
    }

    fn write(&self, index: u64, offset: u64, buffer: &[u8]) -> Result<()> {
        return Err(Error::Unimplemented);
    }
}
