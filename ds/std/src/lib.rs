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

use std::{fs::File, os::unix::fs::FileExt};

use hyrax_ds::{DataStorage, Result};

pub struct DataStorageServer {
    file: File,
}

impl DataStorageServer {
    pub fn new(file_path: &str) -> Self {
        Self {
            file: File::open(file_path).unwrap(),
        }
    }
}

impl DataStorage for DataStorageServer {
    fn read(&self, offset: u64, buffer: &mut [u8]) -> Result<()> {
        self.file.read_exact_at(buffer, offset).unwrap();
        Ok(())
    }

    fn write(&self, offset: u64, buffer: &[u8]) -> Result<()> {
        self.file.write_all_at(buffer, offset).unwrap();
        Ok(())
    }
}
