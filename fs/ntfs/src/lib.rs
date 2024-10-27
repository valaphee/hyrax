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
use zerocopy::{
    little_endian::{U16, U32, U64},
    FromBytes, IntoBytes, KnownLayout,
};

pub struct FileSystemServer<DS: DataStorage> {
    data_storage: DS,
}

impl<DS: DataStorage> FileSystemServer<DS> {
    pub fn new(data_storage: DS) -> Result<Self> {
        Ok(Self { data_storage })
    }
}

impl<DS: DataStorage> FileSystem for FileSystemServer<DS> {
    fn stat(&self, index: u64, buffer: &mut [u8]) -> Result<()> {
        return Err(Error::Unimplemented);
    }

    fn read(&self, index: u64, offset: u64, buffer: &mut [u8]) -> Result<()> {
        return Err(Error::Unimplemented);
    }

    fn write(&self, index: u64, offset: u64, buffer: &[u8]) -> Result<()> {
        return Err(Error::Unimplemented);
    }
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout)]
struct FileRecordSegmentHeader {
    /// The multisector header defined by the cache manager. The
    /// MULTI_SECTOR_HEADER structure always contains the signature "FILE" and a
    /// description of the location and size of the update sequence array.
    multi_sector_header: MultiSectorHeader,
    /// Reserved.
    reserved1: U64,
    /// The sequence number. This value is incremented each time that a file
    /// record segment is freed; it is 0 if the segment is not used. The
    /// SequenceNumber field of a file reference must match the contents of this
    /// field; if they do not match, the file reference is incorrect and
    /// probably obsolete.
    sequence_number: U16,
    /// Reserved.
    reserved2: U16,
    /// The offset of the first attribute record, in bytes.
    first_attribute_offset: U16,
    /// The file flags.
    /// FILE_RECORD_SEGMENT_IN_USE (0x0001)
    /// FILE_FILE_NAME_INDEX_PRESENT (0x0002)
    flags: U16,
    /// Reserved.
    reserved3: [U32; 2],
    /// A file reference to the base file record segment for this file. If this
    /// is the base file record, the value is 0. See MFT_SEGMENT_REFERENCE.
    base_file_record_segment: MftSegmentReference,
    /// Reserved.
    reserved4: U16,
    /// The update sequence array to protect multisector transfers of the file
    /// record segment.
    update_sequence_array: (),
}

/// Represents the multisector header.
#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout)]
struct MultiSectorHeader {
    /// The signature. This value is a convenience to the user.
    signature: [u8; 4],
    /// The offset to the update sequence array, from the start of this
    /// structure. The update sequence array must end before the last USHORT
    /// value in the first sector.
    update_sequence_array_offset: U16,
    /// The size of the update sequence array, in bytes.
    update_sequence_array_size: U16,
}

/// Represents an address in the master file table (MFT). The address is tagged
/// with a circularly reused sequence number that is set at the time the MFT
/// segment reference was valid.
#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout)]
struct MftSegmentReference {
    /// The low part of the segment number.
    segment_number_low_part: U32,
    /// The high part of the segment number.
    segment_number_high_part: U16,
    /// The nonzero sequence number. The value 0 is reserved.
    sequence_number: U16,
}

/// Represents an attribute record.
#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout)]
struct AttributeRecordHeader {
    /// The attribute type code.
    type_code: u8,
    /// The size of the attribute record, in bytes. This value reflects the
    /// required size for the record variant and is always rounded to the
    /// nearest quadword boundary.
    record_length: U32,
    /// The attribute form code.
    form_code: u8,
    /// The size of the optional attribute name, in characters, or 0 if there is
    /// no attribute name. The maximum attribute name length is 255 characters.
    name_length: u8,
    /// The offset of the attribute name from the start of the attribute record,
    /// in bytes. If the NameLength member is 0, this member is undefined.
    name_offset: U16,
    /// The attribute flags.
    /// ATTRIBUTE_FLAG_COMPRESSION_MASK (0x00FF)
    /// ATTRIBUTE_FLAG_SPARSE (0x8000)
    /// ATTRIBUTE_FLAG_ENCRYPTED (0x4000)
    flags: U16,
    /// The unique instance for this attribute in the file record.
    instance: U16,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout)]
struct AttributeRecordHeaderResident {
    /// The attribute type code.
    type_code: u8,
    /// The size of the attribute record, in bytes. This value reflects the
    /// required size for the record variant and is always rounded to the
    /// nearest quadword boundary.
    record_length: U32,
    /// The attribute form code.
    form_code: u8,
    /// The size of the optional attribute name, in characters, or 0 if there is
    /// no attribute name. The maximum attribute name length is 255 characters.
    name_length: u8,
    /// The offset of the attribute name from the start of the attribute record,
    /// in bytes. If the NameLength member is 0, this member is undefined.
    name_offset: U16,
    /// The attribute flags.
    /// ATTRIBUTE_FLAG_COMPRESSION_MASK (0x00FF)
    /// ATTRIBUTE_FLAG_SPARSE (0x8000)
    /// ATTRIBUTE_FLAG_ENCRYPTED (0x4000)
    flags: U16,
    /// The unique instance for this attribute in the file record.
    instance: U16,
    /// The size of the attribute value, in bytes.
    value_length: U32,
    /// The offset to the value from the start of the attribute record, in
    /// bytes.
    value_offset: U16,
    /// Reserved.
    reserved: [u8; 2],
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout)]
struct AttributeRecordHeaderNonresident {
    /// The attribute type code.
    type_code: u8,
    /// The size of the attribute record, in bytes. This value reflects the
    /// required size for the record variant and is always rounded to the
    /// nearest quadword boundary.
    record_length: U32,
    /// The attribute form code.
    form_code: u8,
    /// The size of the optional attribute name, in characters, or 0 if there is
    /// no attribute name. The maximum attribute name length is 255 characters.
    name_length: u8,
    /// The offset of the attribute name from the start of the attribute record,
    /// in bytes. If the NameLength member is 0, this member is undefined.
    name_offset: U16,
    /// The attribute flags.
    /// ATTRIBUTE_FLAG_COMPRESSION_MASK (0x00FF)
    /// ATTRIBUTE_FLAG_SPARSE (0x8000)
    /// ATTRIBUTE_FLAG_ENCRYPTED (0x4000)
    flags: U16,
    /// The unique instance for this attribute in the file record.
    instance: U16,
    /// The lowest virtual cluster number (VCN) covered by this attribute
    /// record.
    lowest_vcn: U64,
    /// The highest VCN covered by this attribute record.
    highest_vcn: U64,
    /// The offset to the mapping pairs array from the start of the attribute
    /// record, in bytes. For more information, see Remarks.
    mapping_pairs_offset: U16,
    /// Reserved.
    reserved: [u8; 6],
    /// The allocated size of the file, in bytes. This value is an even multiple
    /// of the cluster size. This member is not valid if the LowestVcn member is
    /// nonzero.
    allocated_length: U64,
    /// The file size (highest byte that can be read plus 1), in bytes. This
    /// member is not valid if LowestVcn is nonzero.
    file_size: U64,
    /// The valid data length (highest initialized byte plus 1), in bytes. This
    /// value is rounded to the nearest cluster boundary. This member is not
    /// valid if LowestVcn is nonzero.
    valid_data_length: U64,
    /// The total allocated for the file (the sum of the allocated clusters).
    total_allocated: U64,
}
