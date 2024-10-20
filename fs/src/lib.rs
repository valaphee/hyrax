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

use thiserror::Error;

pub trait FileSystem {
    fn stat(&self, index: u64, buffer: &mut [u8]) -> FileSystemResult<()>;

    fn read(&self, index: u64, offset: u64, buffer: &mut [u8]) -> FileSystemResult<usize>;

    fn write(&self, index: u64, offset: u64, buffer: &[u8]) -> FileSystemResult<()>;
}

#[derive(Error, Debug)]
pub enum FileSystemError {
    #[error("Read-only")]
    ReadOnly,
}

pub type FileSystemResult<T> = Result<T, FileSystemError>;
