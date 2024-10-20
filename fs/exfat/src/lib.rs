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

use std::mem::MaybeUninit;

use hyrax_ds::DataStorage;
use hyrax_fs::{FileSystem, FileSystemResult};
use zerocopy::{
    little_endian::{U16, U32, U64},
    FromBytes, IntoBytes, KnownLayout,
};

pub struct FileSystemImpl {
    data_storage: Box<dyn DataStorage>,

    bytes_per_sector_log2: u8,
    bytes_per_cluster_log2: u8,
    cluster_heap_offset: u64,
    first_cluster_of_root_directory: u32,
}

impl FileSystemImpl {
    pub fn new(data_storage: Box<dyn DataStorage>) -> FileSystemResult<Self> {
        let mut boot_sector: BootSector = unsafe { MaybeUninit::uninit().assume_init() };
        data_storage.read(0, boot_sector.as_mut_bytes()).unwrap();

        Ok(Self { data_storage })
    }
}

impl FileSystem for FileSystemImpl {
    fn stat(&self, index: u64, buffer: &mut [u8]) -> FileSystemResult<()> {
        let first_cluster = if index == 0 {
            self.first_cluster_of_root_directory
        } else {
            let mut dir_entry: FileDirEntry = unsafe { MaybeUninit::uninit().assume_init() };
            self.data_storage
                .read(index, dir_entry.as_mut_bytes())
                .unwrap();

            0
        };

        for cluster in ClusterChain {
            let mut dir_entry: FileDirEntry = unsafe { MaybeUninit::uninit().assume_init() };
            self.data_storage
                .read(
                    self.cluster_heap_offset + ((cluster as u64) << self.bytes_per_cluster_log2),
                    dir_entry.as_mut_bytes(),
                )
                .unwrap();
        }

        Ok(())
    }

    fn read(&self, index: u64, offset: u64, mut buffer: &mut [u8]) -> FileSystemResult<usize> {
        let (first_cluster, data_length) = {
            let mut dir_entry: FileDirEntry = unsafe { MaybeUninit::uninit().assume_init() };
            self.data_storage
                .read(index, dir_entry.as_mut_bytes())
                .unwrap();

            (0, 0)
        };

        let mut cluster_chain = ClusterChain.skip((offset >> self.bytes_per_cluster_log2) as usize);
        if let Some(cluster) = cluster_chain.next() {
            let offset = offset & (1 << self.bytes_per_cluster_log2);
            let buffer_end = buffer
                .len()
                .min((1 << self.bytes_per_cluster_log2) - offset as usize);
            self.data_storage
                .read(
                    self.cluster_heap_offset
                        + ((cluster as u64) << self.bytes_per_cluster_log2)
                        + offset,
                    &mut buffer[..buffer_end],
                )
                .unwrap();
            buffer = &mut buffer[buffer_end..]
        }
        for (cluster, buffer) in
            cluster_chain.zip(buffer.chunks_mut(1 << self.bytes_per_cluster_log2))
        {
            self.data_storage
                .read(
                    self.cluster_heap_offset + ((cluster as u64) << self.bytes_per_cluster_log2),
                    buffer,
                )
                .unwrap();
        }

        Ok(buffer.len().min(data_length as usize - offset as usize))
    }

    fn write(&self, index: u64, offset: u64, mut buffer: &[u8]) -> FileSystemResult<()> {
        let first_cluster = {
            let mut dir_entry: FileDirEntry = unsafe { MaybeUninit::uninit().assume_init() };
            self.data_storage
                .read(index, dir_entry.as_mut_bytes())
                .unwrap();
        };

        let mut cluster_chain = ClusterChain.skip((offset >> self.bytes_per_cluster_log2) as usize);
        if let Some(cluster) = cluster_chain.next() {
            let offset = offset & (1 << self.bytes_per_cluster_log2);
            let buffer_end = buffer
                .len()
                .min((1 << self.bytes_per_cluster_log2) - offset as usize);
            self.data_storage
                .write(
                    self.cluster_heap_offset
                        + ((cluster as u64) << self.bytes_per_cluster_log2)
                        + offset,
                    &buffer[..buffer_end],
                )
                .unwrap();
            buffer = &buffer[buffer_end..]
        }
        for (cluster, buffer) in cluster_chain.zip(buffer.chunks(1 << self.bytes_per_cluster_log2))
        {
            self.data_storage
                .write(
                    self.cluster_heap_offset + ((cluster as u64) << self.bytes_per_cluster_log2),
                    buffer,
                )
                .unwrap();
        }

        Ok(())
    }
}

struct ClusterChain;

impl Iterator for ClusterChain {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout)]
struct BootSector {
    /// The JumpBoot field shall contain the jump instruction for CPUs common in
    /// personal computers, which, when executed, "jumps" the CPU to execute the
    /// boot-strapping instructions in the BootCode field.
    ///
    /// The valid value for this field is (in order of low-order byte to
    /// high-order byte) EBh 76h 90h.
    jump_boot: [u8; 3],
    /// The FileSystemName field shall contain the name of the file system on
    /// the volume.
    ///
    /// The valid value for this field is, in ASCII characters, "EXFAT ", which
    /// includes three trailing white spaces.
    file_system_name: [u8; 8],
    /// The MustBeZero field shall directly correspond with the range of bytes
    /// the packed BIOS parameter block consumes on FAT12/16/32 volumes.
    ///
    /// The valid value for this field is 0, which helps to prevent FAT12/16/32
    /// implementations from mistakenly mounting an exFAT volume.
    must_be_zero: [u8; 53],
    /// The PartitionOffset field shall describe the media-relative sector
    /// offset of the partition which hosts the given exFAT volume. This field
    /// aids boot-strapping from the volume using extended INT 13h on personal
    /// computers.
    ///
    /// All possible values for this field are valid; however, the value 0
    /// indicates implementations shall ignore this field.
    partition_offset: U64,
    /// The VolumeLength field shall describe the size of the given exFAT volume
    /// in sectors.
    ///
    /// The valid range of values for this field shall be:
    /// - At least 220/ 2BytesPerSectorShift, which ensures the smallest volume
    ///   is no less than 1MB
    /// - At most 264- 1, the largest value this field can describe.
    ///
    /// However, if the size of the Excess Space sub-region is 0, then the
    /// largest value of this field is ClusterHeapOffset + (232- 11) *
    /// 2^SectorsPerClusterShift.
    volume_length: U64,
    /// The FatOffset field shall describe the volume-relative sector offset of
    /// the First FAT. This field enables implementations to align the First FAT
    /// to the characteristics of the underlying storage meu64dia.
    ///
    /// The valid range of values for this field shall be:
    /// - At least 24, which accounts for the sectors the Main Boot and Backup
    ///   Boot regions consume
    /// - At most ClusterHeapOffset - (FatLength * NumberOfFats), which accounts
    ///   for the sectors the Cluster Heap consumes
    fat_offset: U32,
    /// The FatLength field shall describe the length, in sectors, of each FAT
    /// table (the volume may contain up to two FATs).
    ///
    /// The valid range of values for this field shall be:
    /// - At least (ClusterCount + 2) * 22/ 2BytesPerSectorShiftrounded up to
    ///   the nearest integer, which ensures each FAT has sufficient space for
    ///   describing all the clusters in the Cluster Heap
    /// - At most (ClusterHeapOffset - FatOffset) / NumberOfFats rounded down to
    ///   the nearest integer, which ensures the FATs exist before the Cluster
    ///   Heap
    ///
    /// This field may contain a value in excess of its lower bound (as
    /// described above) to enable the Second FAT, if present, to also be
    /// aligned to the characteristics of the underlying storage media. The
    /// contents of the space which exceeds what the FAT itself requires, if
    /// any, are undefined.
    fat_length: U32,
    /// The ClusterHeapOffset field shall describe the volume-relative sector
    /// offset of the Cluster Heap. This field enables implementations to align
    /// the Cluster Heap to the characteristics of the underlying storage media.
    /// The valid range of values for this field shall be:
    /// - At least FatOffset + FatLength * NumberOfFats, to account for the
    ///   sectors all the preceding regions consume
    /// - At most 232- 1 or VolumeLength - (ClusterCount *
    ///   2SectorsPerClusterShift), whichever calculation is less
    cluster_heap_offset: U32,
    /// The ClusterCount field shall describe the number of clusters the Cluster
    /// Heap contains.
    ///
    /// The valid value for this field shall be the lesser of the following:
    /// - (VolumeLength - ClusterHeapOffset) / 2SectorsPerClusterShiftrounded
    ///   down to the nearest integer, which is exactly the number of clusters
    ///   which can fit between the beginning of the Cluster Heap and the end of
    ///   the volume
    /// - 2^32- 11, which is the maximum number of clusters a FAT can describe
    ///
    /// The value of the ClusterCount field determines the minimum size of a
    /// FAT. To avoid extremely large FATs, implementations can control the
    /// number of clusters in the Cluster Heap by increasing the cluster size
    /// (via the SectorsPerClusterShift field). This specification recommends no
    /// more than 224- 2 clusters in the Cluster Heap. However, implementations
    /// shall be able to handle volumes with up to 232- 11 clusters in the
    /// Cluster Heap.
    cluster_count: U32,
    /// The FirstClusterOfRootDirectory field shall contain the cluster index of
    /// the first cluster of the root directory. Implementations should make
    /// every effort to place the first cluster of the root directory in the
    /// first non-bad cluster after the clusters the Allocation Bitmap and
    /// Up-case Table consume.
    ///
    /// The valid range of values for this field shall be:
    /// - At least 2, the index of the first cluster in the Cluster Heap
    /// - At most ClusterCount + 1, the index of the last cluster in the Cluster
    ///   Heap
    first_cluster_of_root_directory: U32,
    /// The VolumeSerialNumber field shall contain a unique serial number. This
    /// assists implementations to distinguish among different exFAT volumes.
    /// Implementations should generate the serial number by combining the date
    /// and time of formatting the exFAT volume. The mechanism for combining
    /// date and time to form a serial number is implementation-specific.
    ///
    /// All possible values for this field are valid.
    volume_serial_number: U32,
    /// The FileSystemRevision field shall describe the major and minor revision
    /// numbers of the exFAT structures on the given volume.
    ///
    /// The high-order byte is the major revision number and the low-order byte
    /// is the minor revision number. For example, if the high-order byte
    /// contains the value 01h and if the low-order byte contains the value 05h,
    /// then the FileSystemRevision field describes the revision number 1.05.
    /// Likewise, if the high-order byte contains the value 0Ah and if the
    /// low-order byte contains the value 0Fh, then the FileSystemRevision field
    /// describes the revision number 10.15.
    ///
    /// The valid range of values for this field shall be:
    /// - At least 0 for the low-order byte and 1 for the high-order byte
    /// - At most 99 for the low-order byte and 99 for the high-order byte
    ///
    /// The revision number of exFAT this specification describes is 1.00.
    /// Implementations of this specification should mount any exFAT volume with
    /// major revision number 1 and shall not mount any exFAT volume with any
    /// other major revision number. Implementations shall honor the minor
    /// revision number and shall not perform operations or create any file
    /// system structures not described in the given minor revision number's
    /// corresponding specification.
    file_system_revision: U16,
    /// The VolumeFlags field shall contain flags which indicate the status of
    /// various file system structures on the exFAT volume (see Table 5).
    ///
    /// Implementations shall not include this field when computing its
    /// respective Main Boot or Backup Boot region checksum. When referring to
    /// the Backup Boot Sector, implementations shall treat this field as stale.
    volume_flags: U16,
    /// The BytesPerSectorShift field shall describe the bytes per sector
    /// expressed as log2(N), where N is the number of bytes per sector. For
    /// example, for 512 bytes per sector, the value of this field is 9.
    ///
    /// The valid range of values for this field shall be:
    /// -At least 9 (sector size of 512 bytes), which is the smallest sector
    /// possible for an exFAT volume -At most 12 (sector size of 4096
    /// bytes), which is the memory page size of CPUs common in personal
    /// computers
    bytes_per_sector_shift: u8,
    /// The SectorsPerClusterShift field shall describe the sectors per cluster
    /// expressed as log2(N), where N is number of sectors per cluster. For
    /// example, for 8 sectors per cluster, the value of this field is 3.
    ///
    /// The valid range of values for this field shall be:
    /// - At least 0 (1 sector per cluster), which is the smallest cluster
    ///   possible
    /// - At most 25 - BytesPerSectorShift, which evaluates to a cluster size of
    ///   32MB
    sectors_per_cluster_shift: u8,
    /// The NumberOfFats field shall describe the number of FATs and Allocation
    /// Bitmaps the volume contains.
    ///
    /// The valid range of values for this field shall be:
    /// - 1, which indicates the volume only contains the First FAT and First
    ///   Allocation Bitmap
    /// - 2, which indicates the volume contains the First FAT, Second FAT,
    ///   First Allocation Bitmap, and Second Allocation Bitmap; this value is
    ///   only valid for TexFAT volumes
    number_of_fats: u8,
    /// The DriveSelect field shall contain the extended INT 13h drive number,
    /// which aids boot-strapping from this volume using extended INT 13h on
    /// personal computers.
    ///
    /// All possible values for this field are valid. Similar fields in previous
    /// FAT-based file systems frequently contained the value 80h.
    drive_select: u8,
    /// The PercentInUse field shall describe the percentage of clusters in the
    /// Cluster Heap which are allocated.
    ///
    /// The valid range of values for this field shall be:
    /// - Between 0 and 100 inclusively, which is the percentage of allocated
    ///   clusters in the Cluster Heap, rounded down to the nearest integer
    /// - Exactly FFh, which indicates the percentage of allocated clusters in
    ///   the Cluster Heap is not available
    ///
    /// Implementations shall change the value of this field to reflect changes
    /// in the allocation of clusters in the Cluster Heap or shall change it to
    /// FFh.
    ///
    /// Implementations shall not include this field when computing its
    /// respective Main Boot or Backup Boot region checksum. When referring to
    /// the Backup Boot Sector, implementations shall treat this field as stale.
    percent_in_use: u8,
    reserved: [u8; 7],
    /// The BootCode field shall contain boot-strapping instructions.
    /// Implementations may populate this field with the CPU instructions
    /// necessary for boot-strapping a computer system. Implementations which
    /// don't provide boot-strapping instructions shall initialize each byte in
    /// this field to F4h (the halt instruction for CPUs common in personal
    /// computers) as part of their format operation.
    boot_code: [u8; 390],
    /// The BootSignature field shall describe whether the intent of a given
    /// sector is for it to be a Boot Sector or not.
    ///
    /// The valid value for this field is AA55h. Any other value in this field
    /// invalidates its respective Boot Sector. Implementations should verify
    /// the contents of this field prior to depending on any other field in its
    /// respective Boot Sector.
    boot_signature: [u8; 2],
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout)]
struct DirEntry {
    /// The EntryType field has three modes of usage which the value of the
    /// field defines (see list below).
    ///
    /// 00h, which is an end-of-directory marker and the following conditions
    /// apply:
    /// - All other fields in the given DirectoryEntry are actually reserved
    /// - All subsequent directory entries in the given directory also are
    ///   end-of-directory markers
    /// - End-of-directory markers are only valid outside directory entry sets
    /// - Implementations may overwrite end-of-directory markers as necessary
    ///
    /// Between 01h and 7Fh inclusively, which is an unused-directory-entry
    /// marker and the following conditions apply:
    /// - All other fields in the given DirectoryEntry are actually undefined
    /// - Unused directory entries are only valid outside of directory entry
    ///   sets
    /// - Implementations may overwrite unused directory entries as necessary
    /// - This range of values corresponds to the InUse field (see Section
    ///   6.2.1.4) containing the value 0
    ///
    /// Between 81h and FFh inclusively, which is a regular directory entry and
    /// the following conditions apply:
    /// - The contents of the EntryType field (see Table 15) determine the
    ///   layout of the remainder of the DirectoryEntry structure
    /// - This range of values, and only this range of values, are valid inside
    ///   a directory entry set
    /// - This range of values directly corresponds to the InUse field (see
    ///   Section 6.2.1.4) containing the value 1
    ///
    /// To prevent modifications to the InUse field (see Section 6.2.1.4)
    /// erroneously resulting in an end-of-directory marker, the value 80h is
    /// invalid.
    entry_type: u8,
    /// This field is mandatory and structures which derive from this template
    /// may define its contents.
    custom_defined: [u8; 19],
    /// The FirstCluster field shall contain the index of the first cluster of
    /// an allocation in the Cluster Heap associated with the given directory
    /// entry.
    ///
    /// The valid range of values for this field shall be:
    /// - Exactly 0, which means no cluster allocation exists
    /// - Between 2 and ClusterCount + 1, which is the range of valid cluster
    ///   indices
    ///
    /// Structures which derive from this template may redefine both the
    /// FirstCluster and DataLength fields, if a cluster allocation is not
    /// compatible with the derivative structure.
    first_cluster: U32,
    /// The DataLength field describes the size, in bytes, of the data the
    /// associated cluster allocation contains.
    ///
    /// The valid range of value for this field is:
    /// - At least 0; if the FirstCluster field contains the value 0, then this
    ///   field's only valid value is 0
    /// - At most ClusterCount * 2SectorsPerClusterShift* 2BytesPerSectorShift
    ///
    /// Structures which derive from this template may redefine both the
    /// FirstCluster and DataLength fields, if a cluster allocation is not
    /// possible for the derivative structure.
    data_length: U64,
}

/// In the exFAT file system, a FAT does not describe the allocation state of
/// clusters; rather, an Allocation Bitmap does. Allocation Bitmaps exist in the
/// Cluster Heap (see Section 7.1.5) and have corresponding critical primary
/// directory entries in the root directory (see Table 20).
///
/// The NumberOfFats field determines the number of valid Allocation Bitmap
/// directory entries in the root directory. If the NumberOfFats field contains
/// the value 1, then the only valid number of Allocation Bitmap directory
/// entries is 1. Further, the one Allocation Bitmap directory entry is only
/// valid if it describes the First Allocation Bitmap (see Section 7.1.2.1). If
/// the NumberOfFats field contains the value 2, then the only valid number of
/// Allocation Bitmap directory entries is 2. Further, the two Allocation Bitmap
/// directory entries are only valid if one describes the First Allocation
/// Bitmap and the other describes the Second Allocation Bitmap.
#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout)]
struct AllocationBitmapDirEntry {
    /// The EntryType field shall conform to the definition provided in the
    /// Generic Primary DirectoryEntry template (see Section 6.3.1).
    entry_type: u8,
    /// The BitmapFlags field contains flags (see Table 21).
    bitmap_flags: u8,
    /// This field is mandatory and its contents are reserved.
    reserved: [u8; 18],
    /// The FirstCluster field shall conform to the definition provided in the
    /// Generic Primary DirectoryEntry template (see Section 6.3.5).
    ///
    /// This field contains the index of the first cluster of the cluster chain,
    /// as the FAT describes, which hosts the Allocation Bitmap.
    first_cluster: U32,
    /// The DataCluster field shall conform to the definition provided in the
    /// Generic Primary DirectoryEntry template (see Section 6.3.6).
    data_length: U64,
}

/// The Up-case Table defines the conversion from lower-case to upper-case
/// characters. This is important due to the File Name directory entry (see
/// Section 7.7) using Unicode characters and the exFAT file system being case
/// insensitive and case preserving. The Up-case Table exists in the Cluster
/// Heap (see Section 7.2.5) and has a corresponding critical primary directory
/// entry in the root directory (see Table 23). The valid number of Up-case
/// Table directory entries is 1.
///
/// Due to the relationship between the Up-case Table and file names,
/// implementations should not modify the Up-case Table, except as a result of
/// format operations.
#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout)]
struct UpcaseTableDirEntry {
    /// The EntryType field shall conform to the definition provided in the
    /// Generic Primary DirectoryEntry template (see Section 6.3.1).
    entry_type: u8,
    /// This field is mandatory and its contents are reserved.
    reserved: [u8; 3],
    /// The TableChecksum field contains the checksum of the Up-case Table
    /// (which the FirstCluster and DataLength fields describe). Implementations
    /// shall verify the contents of this field are valid prior to using the
    /// Up-case Table.
    table_checksum: U32,
    /// This field is mandatory and its contents are reserved.
    reserved2: [u8; 12],
    /// The FirstCluster field shall conform to the definition provided in the
    /// Generic Primary DirectoryEntry template (see Section 6.3.5).
    ///
    /// This field contains the index of the first cluster of the cluster chain,
    /// as the FAT describes, which hosts the Up-case Table.
    first_cluster: U32,
    /// The DataCluster field shall conform to the definition provided in the
    /// Generic Primary DirectoryEntry template (see Section 6.3.6).
    data_length: U64,
}

/// The Volume Label is a Unicode string which enables end users to distinguish
/// their storage volumes. In the exFAT file system, the Volume Label exists as
/// a critical primary directory entry in the root directory (see Table 26). The
/// valid number of Volume Label directory entries ranges from 0 to 1.
#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout)]
struct VolumeLabelDirEntry {
    /// The EntryType field shall conform to the definition provided in the
    /// Generic Primary DirectoryEntry template (see Section 6.3.1).
    entry_type: u8,
    /// The CharacterCount field shall contain the length of the Unicode string
    /// the VolumeLabel field contains.
    ///
    /// The valid range of values for this field shall be:
    /// - At least 0, which means the Unicode string is 0 characters long (which
    ///   is the equivalent of no volume label)
    /// - At most 11, which means the Unicode string is 11 characters long
    character_count: u8,
    /// The VolumeLabel field shall contain a Unicode string, which is the
    /// user-friendly name of the volume. The VolumeLabel field has the same set
    /// of invalid characters as the FileName field of the File Name directory
    /// entry (see Section 7.7.3).
    volume_label: [U16; 11],
    /// This field is mandatory and its contents are reserved.
    reserved: [u8; 8],
}

/// File directory entries describe files and directories. They are critical
/// primary directory entries and any directory may contain zero or more File
/// directory entries (see Table 27). For a File directory entry to be valid,
/// exactly one Stream Extension directory entry and at least one File Name
/// directory entry must immediately follow the File directory entry (see
/// Section 7.6 and Section 7.7, respectively).
#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout)]
struct FileDirEntry {
    /// The EntryType field shall conform to the definition provided in the
    /// Generic Primary DirectoryEntry template (see Section 6.3.1).
    entry_type: u8,
    /// The SecondaryCount field shall conform to the definition provided in the
    /// Generic Primary DirectoryEntry template (see Section 6.3.2).
    secondary_count: u8,
    /// The SetChecksum field shall conform to the definition provided in the
    /// Generic Primary DirectoryEntry template (see Section 6.3.3).
    set_checksum: U16,
    /// The FileAttributes field contains flags (see Table 28).
    file_attributes: U16,
    /// This field is mandatory and its contents are reserved.
    reserved1: [u8; 2],
    /// Timestamp fields describe both local date and time, down to a two-second
    /// resolution (see Table 29).
    create_timestamp: U32,
    /// Timestamp fields describe both local date and time, down to a two-second
    /// resolution (see Table 29).
    last_modified_timestamp: U32,
    /// Timestamp fields describe both local date and time, down to a two-second
    /// resolution (see Table 29).
    last_accessed_timestamp: U32,
    /// 10msIncrement fields shall provide additional time resolution to their
    /// corresponding Timestamp fields in ten-millisecond multiples.
    ///
    /// The valid range of values for these fields shall be:
    /// - At least 0, which represents 0 milliseconds
    /// - At most 199, which represents 1990 milliseconds
    create_10ms_increment: u8,
    /// 10msIncrement fields shall provide additional time resolution to their
    /// corresponding Timestamp fields in ten-millisecond multiples.
    ///
    /// The valid range of values for these fields shall be:
    /// - At least 0, which represents 0 milliseconds
    /// - At most 199, which represents 1990 milliseconds
    last_modified_10ms_increment: u8,
    /// UtcOffset fields (see Table 30) shall describe the offset from UTC to
    /// the local date and time their corresponding Timestamp and 10msIncrement
    /// fields describe. The offset from UTC to the local date and time includes
    /// the effects of time zones and other date-time adjustments, such as
    /// daylight saving and regional summer time changes.
    create_utc_offset: u8,
    /// UtcOffset fields (see Table 30) shall describe the offset from UTC to
    /// the local date and time their corresponding Timestamp and 10msIncrement
    /// fields describe. The offset from UTC to the local date and time includes
    /// the effects of time zones and other date-time adjustments, such as
    /// daylight saving and regional summer time changes.
    last_modified_utc_offset: u8,
    /// UtcOffset fields (see Table 30) shall describe the offset from UTC to
    /// the local date and time their corresponding Timestamp and 10msIncrement
    /// fields describe. The offset from UTC to the local date and time includes
    /// the effects of time zones and other date-time adjustments, such as
    /// daylight saving and regional summer time changes.
    last_accessed_utc_offset: u8,
    /// This field is mandatory and its contents are reserved.
    reserved2: [u8; 7],
}

/// The Volume GUID directory entry contains a GUID which enables
/// implementations to uniquely and programmatically distinguish volumes. The
/// Volume GUID exists as a benign primary directory entry in the root directory
/// (see Table 32). The valid number of Volume GUID directory entries ranges
/// from 0 to 1.
#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout)]
struct VolumeGuidDirEntry {
    /// The EntryType field shall conform to the definition provided in the
    /// Generic Primary DirectoryEntry template (see Section 6.3.1).
    entry_type: u8,
    /// The SecondaryCount field shall conform to the definition provided in the
    /// Generic Primary DirectoryEntry template (see Section 6.3.2).
    ///
    /// For the Volume GUID directory entry, the valid value for this field is
    /// 0.
    secondary_count: u8,
    /// The SetChecksum field shall conform to the definition provided in the
    /// Generic Primary DirectoryEntry template (see Section 6.3.3).
    set_checksum: U16,
    /// The GeneralPrimaryFlags field shall conform to the definition provided
    /// in the Generic Primary DirectoryEntry template (see Section 6.3.4) and
    /// defines the contents of the CustomDefined field to be reserved.
    general_primary_flags: U16,
    /// The VolumeGuid field shall contain a GUID which uniquely identifies the
    /// given volume.
    ///
    /// All possible values for this field are valid, except the null GUID,
    /// which is {00000000-0000-0000-0000-000000000000}.
    volume_guid: [u8; 16],
    /// This field is mandatory and its contents are reserved.
    reserved: [u8; 10],
}

/// The Stream Extension directory entry is a critical secondary directory entry
/// in File directory entry sets (see Table 33). The valid number of Stream
/// Extension directory entries in a File directory entry set is 1. Further,
/// this directory entry is valid only if it immediately follows the File
/// directory entry.
#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout)]
struct StreamExtensionDirEntry {
    /// The EntryType field shall conform to the definition provided in the
    /// Generic Secondary DirectoryEntry template (see Section 6.4.1).
    entry_type: u8,
    /// The GeneralSecondaryFlags field shall conform to the definition provided
    /// in the Generic Secondary DirectoryEntry template (see Section 6.4.2) and
    /// defines the contents of the CustomDefined field to be reserved.
    general_secondary_flags: u8,
    /// This field is mandatory and its contents are reserved.
    reserved1: u8,
    /// The NameLength field shall contain the length of the Unicode string the
    /// subsequent File Name directory entries (see Section 7.7) collectively
    /// contain.
    ///
    /// The valid range of values for this field shall be:
    /// - At least 1, which is the shortest possible file name
    /// - At most 255, which is the longest possible file name
    ///
    /// The value of the NameLength field also affects the number File Name
    /// Directory Entries (see Section 7.7).
    name_length: u8,
    /// The NameHash field shall contain a 2-byte hash (see Figure 4) of the
    /// up-cased file name. This enables implementations to perform a quick
    /// comparison when searching for a file by name. Importantly, the NameHash
    /// provides a sure verification of a mismatch. Implementations shall verify
    /// all NameHash matches with a comparison of the up-cased file name.
    name_hash: U16,
    /// This field is mandatory and its contents are reserved.
    reserved2: [u8; 2],
    /// The ValidDataLength field shall describe how far into the data stream
    /// user data has been written. Implementations shall update this field as
    /// they write data further out into the data stream. On the storage media,
    /// the data between the valid data length and the data length of the data
    /// stream is undefined. Implementations shall return zeroes for read
    /// operations beyond the valid data length.
    ///
    /// If the corresponding File directory entry describes a directory, then
    /// the only valid value for this field is equal to the value of the
    /// DataLength field. Otherwise, the range of valid values for this field
    /// shall be:
    /// - At least 0, which means no user data has been written out to the data
    ///   stream
    /// - At most DataLength, which means user data has been written out to the
    ///   entire length of the data stream
    valid_data_length: U64,
    /// This field is mandatory and its contents are reserved.
    reserved3: [u8; 4],
    /// The FirstCluster field shall conform to the definition provided in the
    /// Generic Secondary DirectoryEntry template (see Section 6.4.3).
    ///
    /// This field shall contain the index of the first cluster of the data
    /// stream, which hosts the user data.
    first_cluster: U32,
    /// The DataLength field shall conform to the definition provided in the
    /// Generic Secondary DirectoryEntry template (see Section 6.4.4).
    ///
    /// If the corresponding File directory entry describes a directory, then
    /// the valid value for this field is the entire size of the associated
    /// allocation, in bytes, which may be 0. Further, for directories, the
    /// maximum value for this field is 256MB.
    data_length: U64,
}

/// File Name directory entries are critical secondary directory entries in File
/// directory entry sets (see Table 34). The valid number of File Name directory
/// entries in a File directory entry set is NameLength / 15, rounded up to the
/// nearest integer. Further, File Name directory entries are valid only if they
/// immediately follow the Stream Extension directory entry as a consecutive
/// series. File Name directory entries combine to form the file name for the
/// File directory entry set.
///
/// All children of a given directory entry shall have unique File Name
/// Directory Entry Sets. That is to say there can be no duplicate file or
/// directory names after up-casing within any one directory.
#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout)]
struct FileNameDirEntry {
    /// The EntryType field shall conform to the definition provided in the
    /// Generic Secondary DirectoryEntry template (see Section 6.4.1).
    entry_type: u8,
    /// The GeneralSecondaryFlags field shall conform to the definition provided
    /// in the Generic Secondary DirectoryEntry template (see Section 6.4.2) and
    /// defines the contents of the CustomDefined field to be reserved.
    general_secondary_flags: u8,
    /// The FileName field shall contain a Unicode string, which is a portion of
    /// the file name. In the order File Name directory entries exist in a File
    /// directory entry set, FileName fields concatenate to form the file name
    /// for the File directory entry set. Given the length of the FileName
    /// field, 15 characters, and the maximum number of File Name directory
    /// entries, 17, the maximum length of the final, concatenated file name is
    /// 255.
    ///
    /// The concatenated file name has the same set of illegal characters as
    /// other FAT-based file systems (see Table 35). Implementations should set
    /// the unused characters of FileName fields to the value 0000h.
    file_name: [U16; 15],
}

/// The Vendor Extension directory entry is a benign secondary directory entry
/// in File directory entry sets (see Table 36). A File directory entry set may
/// contain any number of Vendor Extension directory entries, up to the limit of
/// secondary directory entries, less the number of other secondary directory
/// entries. Further, Vendor Extension directory entries are valid only if they
/// do not precede the required Stream Extension and File Name directory
/// entries.
///
/// Vendor Extension directory entries enable vendors to have unique,
/// vendor-specific directory entries in individual File directory entry sets
/// via the VendorGuid field (see Table 36). Unique directory entries
/// effectively enable vendors to extend the exFAT file system. Vendors may
/// define the contents of the VendorDefined field (see Table 36). Vendor
/// implementations may maintain the contents of the VendorDefined field and may
/// provide vendor-specific functionality.
///
/// Implementations which do not recognize the GUID of a Vendor Extension
/// directory entry shall treat the directory entry the same as any other
/// unrecognized benign secondary directory entry (see Section 8.2).
#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout)]
struct VendorExtensionDirEntry {
    /// The EntryType field shall conform to the definition provided in the
    /// Generic Secondary DirectoryEntry template (see Section 6.4.1).
    entry_type: u8,
    /// The GeneralSecondaryFlags field shall conform to the definition provided
    /// in the Generic Secondary DirectoryEntry template (see Section 6.4.2) and
    /// defines the contents of the CustomDefined field to be reserved.
    general_secondary_flags: u8,
    /// The VendorGuid field shall contain a GUID which uniquely identifies the
    /// given Vendor Extension.
    ///
    /// All possible values for this field are valid, except the null GUID,
    /// which is {00000000-0000-0000-0000-000000000000}. However, vendors should
    /// use a GUID-generating tool, such as GuidGen.exe, to select a GUID when
    /// defining their extensions.
    ///
    /// The value of this field determines the vendor-specific structure of the
    /// VendorDefined field.
    vendor_guid: [u8; 16],
    /// This field is mandatory and vendors may define its contents.
    vendor_defined: [u8; 14],
}

/// The Vendor Allocation directory entry is a benign secondary directory entry
/// in File directory entry sets (see Table 37). A File directory entry set may
/// contain any number of Vendor Allocation directory entries, up to the limit
/// of secondary directory entries, less the number of other secondary directory
/// entries. Further, Vendor Allocation directory entries are valid only if they
/// do not precede the required Stream Extension and File Name directory
/// entries.
///
/// Vendor Allocation directory entries enable vendors to have unique,
/// vendor-specific directory entries in individual File directory entry sets
/// via the VendorGuid field (see Table 37). Unique directory entries
/// effectively enable vendors to extend the exFAT file system. Vendors may
/// define the contents of the associated clusters, if any exist. Vendor
/// implementations may maintain the contents of the associated clusters, if
/// any, and may provide vendor-specific functionality.
///
/// Implementations which do not recognize the GUID of a Vendor Allocation
/// directory entry shall treat the directory entry the same as any other
/// unrecognized benign secondary directory entry (see Section 8.2).
#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout)]
struct VendorAllocationDirEntry {
    /// The EntryType field shall conform to the definition provided in the
    /// Generic Secondary DirectoryEntry template (see Section 6.4.1).
    entry_type: u8,
    /// The GeneralSecondaryFlags field shall conform to the definition provided
    /// in the Generic Secondary DirectoryEntry template (see Section 6.4.2) and
    /// defines the contents of the CustomDefined field to be reserved.
    general_secondary_flags: u8,
    /// The VendorGuid field shall contain a GUID which uniquely identifies the
    /// given Vendor Allocation.
    ///
    /// All possible values for this field are valid, except the null GUID,
    /// which is {00000000-0000-0000-0000-000000000000}. However, vendors should
    /// use a GUID-generating tool, such as GuidGen.exe, to select a GUID when
    /// defining their extensions.
    ///
    /// The value of this field determines the vendor-specific structure of the
    /// contents of the associated clusters, if any exist.
    vendor_guid: [u8; 16],
    /// This field is mandatory and vendors may define its contents.
    vendor_defined: [u8; 2],
    /// The FirstCluster field shall conform to the definition provided in the
    /// Generic Secondary DirectoryEntry template (see Section 6.4.3).
    first_cluster: U32,
    /// The DataLength field shall conform to the definition provided in the
    /// Generic Secondary DirectoryEntry template (see Section 6.4.4).
    data_length: U64,
}
