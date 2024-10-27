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
use hyrax_fs::{Entry, Error, FileSystem, Result};
use zerocopy::{
    little_endian::{U16, U32},
    transmute_mut, FromBytes, IntoBytes, KnownLayout,
};

pub struct FileSystemServer<DS: DataStorage> {
    data_storage: DS,

    bytes_per_cluster_log2: u8,
    fat_offset: u64,
    cluster_heap_offset: u64,
    first_cluster_of_root_directory: u32,
}

impl<DS: DataStorage> FileSystemServer<DS> {
    pub fn new(data_storage: DS) -> Result<Self> {
        let mut boot_sector: BootSector = unsafe { MaybeUninit::uninit().assume_init() };
        data_storage.read(0, boot_sector.as_mut_bytes())?;

        let bytes_per_sector = boot_sector.bpb_bytspersec.get() as u32;
        if !is_power_of_two(bytes_per_sector) {
            return Err(Error::Unimplemented);
        }
        let bytes_per_sector_log2 = bytes_per_sector.ilog2() as u8;
        let sectors_per_cluster = boot_sector.bpb_secperclus as u32;
        if !is_power_of_two(sectors_per_cluster) {
            return Err(Error::Unimplemented);
        }
        let sectors_per_cluster_log2 = sectors_per_cluster.ilog2() as u8;
        let bytes_per_cluster_log2 = bytes_per_sector_log2 + sectors_per_cluster_log2;

        let fat_offset = boot_sector.bpb_rsvdseccnt.get() as u32;
        let number_of_fats = boot_sector.bpb_numfats as u32;
        if number_of_fats != 1 && number_of_fats != 2 {
            return Err(Error::Unimplemented);
        }
        let fat_length = if boot_sector.bpb_fatsz16 != 0 {
            boot_sector.bpb_fatsz16.get() as u32
        } else {
            boot_sector.bpb_fatsz32.get()
        };

        let cluster_heap_offset = fat_offset + fat_length * number_of_fats;

        let first_cluster_of_root_directory = boot_sector.bpb_rootclus.get();

        Ok(Self {
            data_storage,
            bytes_per_cluster_log2,
            fat_offset: (fat_offset as u64) << bytes_per_sector_log2,
            cluster_heap_offset: (cluster_heap_offset as u64) << bytes_per_sector_log2,
            first_cluster_of_root_directory,
        })
    }
}

impl<DS: DataStorage> FileSystem for FileSystemServer<DS> {
    fn stat(&self, index: u64, mut buffer: &mut [u8]) -> Result<()> {
        let first_cluster = if index == 0 {
            self.first_cluster_of_root_directory
        } else {
            let mut dir_entry: DirEntry = unsafe { MaybeUninit::uninit().assume_init() };
            self.data_storage.read(
                self.cluster_heap_offset + index * size_of::<DirEntry>() as u64,
                dir_entry.as_mut_bytes(),
            )?;
            if dir_entry.dir_attr & 0x10 == 0 {
                return Err(Error::Unimplemented);
            }

            (dir_entry.dir_fstcluslo.get() as u32) | (dir_entry.dir_fstclushi.get() as u32) << 16
        };

        let mut name_length = 0;
        for cluster in ClusterChain(self, first_cluster) {
            let cluster = cluster?;
            let offset = (cluster as u64) << self.bytes_per_cluster_log2;
            for offset in
                (offset..offset + 1 << self.bytes_per_cluster_log2).step_by(size_of::<DirEntry>())
            {
                let mut dir_entry: DirEntry = unsafe { MaybeUninit::uninit().assume_init() };
                self.data_storage
                    .read(self.cluster_heap_offset + offset, dir_entry.as_mut_bytes())?;
                if dir_entry.dir_name[0] == 0x00 {
                    break;
                }
                if dir_entry.dir_name[0] == 0xE5 {
                    continue;
                }

                if dir_entry.dir_attr != 0x0F {
                    if name_length == 0 {
                        let (name, extension) = dir_entry.dir_name.split_at(8);
                        for &c in extension.iter().rev().skip_while(|&&c| c == 0x20) {
                            name_length += 1;
                            buffer[buffer.len() - name_length as usize] = c;
                        }
                        if name_length != 0 {
                            name_length += 1;
                            buffer[buffer.len() - name_length as usize] = b'.';
                        }
                        for &c in name.iter().rev().skip_while(|&&c| c == 0x20) {
                            name_length += 1;
                            buffer[buffer.len() - name_length as usize] = c;
                        }
                    }

                    let buffer_len = buffer.len();
                    buffer = &mut buffer[..buffer_len - name_length as usize];
                    *Entry::mut_from_bytes(&mut buffer[..size_of::<Entry>()]).unwrap() = Entry {
                        index: (offset / size_of::<DirEntry>() as u64),
                        data_length: dir_entry.dir_filesize.get() as u64,
                        name_offset: buffer.len() as u32,
                        name_length,
                        padding: [0u8; 3],
                    };
                    buffer = &mut buffer[size_of::<Entry>()..];
                    name_length = 0;
                } else {
                    let ldir_entry: &mut LongNameDirEntry = transmute_mut!(&mut dir_entry);
                    for c in char::decode_utf16(
                        ldir_entry
                            .ldir_name1
                            .iter()
                            .chain(ldir_entry.ldir_name2.iter())
                            .chain(ldir_entry.ldir_name3.iter())
                            .map(|c| c.get())
                            .rev()
                            .skip_while(|&c| c == 0x0000 || c == 0xFFFF),
                    ) {
                        let c = c.unwrap();
                        name_length += c.len_utf8() as u8;
                        let buffer_len = buffer.len();
                        c.encode_utf8(&mut buffer[buffer_len - name_length as usize..]);
                    }
                }
            }
        }

        Ok(())
    }

    fn read(&self, index: u64, offset: u64, mut buffer: &mut [u8]) -> Result<()> {
        let mut dir_entry: DirEntry = unsafe { MaybeUninit::uninit().assume_init() };
        self.data_storage.read(
            self.cluster_heap_offset + index * size_of::<DirEntry>() as u64,
            dir_entry.as_mut_bytes(),
        )?;
        if dir_entry.dir_attr & 0x18 != 0 {
            return Err(Error::Unimplemented);
        }

        let first_cluster =
            (dir_entry.dir_fstcluslo.get() as u32) | (dir_entry.dir_fstclushi.get() as u32) << 16;
        let mut cluster_chain = ClusterChain(self, first_cluster)
            .skip((offset >> self.bytes_per_cluster_log2) as usize);
        if let Some(cluster) = cluster_chain.next() {
            let cluster = cluster?;
            let offset = offset & (1 << self.bytes_per_cluster_log2);
            let buffer_end = buffer
                .len()
                .min((1 << self.bytes_per_cluster_log2) - offset as usize);
            self.data_storage.read(
                self.cluster_heap_offset
                    + ((cluster as u64) << self.bytes_per_cluster_log2)
                    + offset,
                &mut buffer[..buffer_end],
            )?;
            buffer = &mut buffer[buffer_end..]
        }
        for (cluster, buffer) in
            cluster_chain.zip(buffer.chunks_mut(1 << self.bytes_per_cluster_log2))
        {
            let cluster = cluster?;
            self.data_storage.read(
                self.cluster_heap_offset + ((cluster as u64) << self.bytes_per_cluster_log2),
                buffer,
            )?;
        }

        Ok(())
    }

    fn write(&self, index: u64, offset: u64, mut buffer: &[u8]) -> Result<()> {
        let mut dir_entry: DirEntry = unsafe { MaybeUninit::uninit().assume_init() };
        self.data_storage.read(
            self.cluster_heap_offset + index * size_of::<DirEntry>() as u64,
            dir_entry.as_mut_bytes(),
        )?;
        if dir_entry.dir_attr & 0x18 != 0 {
            return Err(Error::Unimplemented);
        }

        let first_cluster =
            (dir_entry.dir_fstcluslo.get() as u32) | (dir_entry.dir_fstclushi.get() as u32) << 16;
        let mut cluster_chain = ClusterChain(self, first_cluster)
            .skip((offset >> self.bytes_per_cluster_log2) as usize);
        if let Some(cluster) = cluster_chain.next() {
            let cluster = cluster?;
            let offset = offset & (1 << self.bytes_per_cluster_log2);
            let buffer_end = buffer
                .len()
                .min((1 << self.bytes_per_cluster_log2) - offset as usize);
            self.data_storage.write(
                self.cluster_heap_offset
                    + ((cluster as u64) << self.bytes_per_cluster_log2)
                    + offset,
                &buffer[..buffer_end],
            )?;
            buffer = &buffer[buffer_end..]
        }
        for (cluster, buffer) in cluster_chain.zip(buffer.chunks(1 << self.bytes_per_cluster_log2))
        {
            let cluster = cluster?;
            self.data_storage.write(
                self.cluster_heap_offset + ((cluster as u64) << self.bytes_per_cluster_log2),
                buffer,
            )?;
        }

        Ok(())
    }
}

struct ClusterChain<'fs, DS: DataStorage>(&'fs FileSystemServer<DS>, u32);

impl<'fs, DS: DataStorage> Iterator for ClusterChain<'fs, DS> {
    type Item = Result<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.1;
        if entry <= 0x0000001 || entry >= 0xFFFFFF7 {
            return None;
        }

        let mut next_entry: U32 = unsafe { MaybeUninit::uninit().assume_init() };
        if let Err(error) = self.0.data_storage.read(
            self.0.fat_offset + entry as u64 * size_of::<u32>() as u64,
            next_entry.as_mut_bytes(),
        ) {
            return Some(Err(error));
        }
        self.1 = next_entry.get();

        Some(Ok(entry - 2))
    }
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout)]
struct BootSector {
    /// Jump instruction to boot code. This field has two allowed forms:
    ///
    /// jmpBoot[0] = 0xEB, jmpBoot[1] = 0x??,
    /// jmpBoot[2] = 0x90
    ///
    /// and
    ///
    /// jmpBoot[0] = 0xE9, jmpBoot[1] = 0x??,
    /// jmpBoot[2] = 0x??
    ///
    /// 0x?? indicates that any 8-bit value is allowed in that byte. What this
    /// forms is a three-byte Intel x86 unconditional branch (jump) instruction
    /// that jumps to the start of the operating system bootstrap code. This
    /// code typically occupies the rest of sector 0 of the volume following the
    /// BPB and possibly other sectors. Either of these forms is acceptable.
    /// JmpBoot[0] = 0xEB is the more frequently used format.
    bs_jmpboot: [u8; 3],
    /// OEM Name Identifier. Can be set by a FAT implementation to any desired
    /// value. Typically this is some indication of what system formatted the
    /// volume.
    bs_oemname: [u8; 8],

    /// Count of bytes per sector. This value may take on only the following
    /// values: 512, 1024, 2048 or 4096.
    bpb_bytspersec: U16,
    /// Number of sectors per allocation unit. This value must be a power of 2
    /// that is greater than 0. The legal values are 1, 2, 4, 8, 16, 32, 64, and
    /// 128.
    bpb_secperclus: u8,
    /// Number of reserved sectors in the reserved region of the volume starting
    /// at the first sector of the volume. This field is used to align the start
    /// of the data area to integral multiples of the cluster size with respect
    /// to the start of the partition/media.
    ///
    /// This field must not be 0 and can be any non-zero value.
    ///
    /// This field should typically be used to align the start of the data area
    /// (cluster #2) to the desired alignment unit, typically cluster size.
    bpb_rsvdseccnt: U16,
    /// The count of file allocation tables (FATs) on the volume. A value of 2
    /// is recommended although a value of 1 is acceptable.
    bpb_numfats: u8,
    /// For FAT12 and FAT16 volumes, this field contains the count of 32-byte
    /// directory entries in the root directory. For FAT32 volumes, this field
    /// must be set to 0. For FAT12 and FAT16 volumes, this value should always
    /// specify a count that when multiplied by 32 results in an even multiple
    /// of BPB_BytsPerSec.
    ///
    /// For maximum compatibility, FAT16 volumes should use the value 512.
    bpb_rootentcnt: U16,
    /// This field is the old 16-bit total count of sectors on the volume. This
    /// count includes the count of all sectors in all four regions of the
    /// volume.
    ///
    /// This field can be 0; if it is 0, then BPB_TotSec32 must be non-zero. For
    /// FAT32 volumes, this field must be 0.
    ///
    /// For FAT12 and FAT16 volumes, this field contains the sector count, and
    /// BPB_TotSec32 is 0 if the total sector count “fits” (is less than
    /// 0x10000).
    bpb_totsec16: U16,
    /// The legal values for this field are 0xF0, 0xF8, 0xF9, 0xFA, 0xFB, 0xFC,
    /// 0xFD, 0xFE, and 0xFF
    ///
    /// 0xF8 is the standard value for “fixed” (non-removable) media. For
    /// removable media, 0xF0 is frequently used.
    bpb_media: u8,
    /// This field is the FAT12/FAT16 16-bit count of sectors occupied by one
    /// FAT. On FAT32 volumes this field must be 0, and BPB_FATSz32 contains the
    /// FAT size count.
    bpb_fatsz16: U16,

    /// Sectors per track for interrupt 0x13.
    ///
    /// This field is only relevant for media that have a geometry (volume is
    /// broken down into tracks by multiple heads and cylinders) and are visible
    /// on interrupt 0x13.
    bpb_secpertrk: U16,
    /// Number of heads for interrupt 0x13. This field is relevant as discussed
    /// earlier for BPB_SecPerTrk.
    ///
    /// This field contains the one based “count of heads”. For example, on a
    /// 1.44 MB 3.5-inch floppy drive this value is 2.
    bpb_numheads: U16,
    /// Count of hidden sectors preceding the partition that contains this FAT
    /// volume. This field is generally only relevant for media visible on
    /// interrupt 0x13.
    ///
    /// This field must always be zero on media that are not partitioned.
    ///
    /// NOTE: Attempting to utilize this field to align the start of data area
    /// is incorrect.
    bpb_hiddsec: U32,

    /// This field is the new 32-bit total count of sectors on the volume. This
    /// count includes the count of all sectors in all four regions of the
    /// volume.
    bpb_totsec32: U32,

    /// This field is the FAT32 32-bit count of sectors occupied by one FAT.
    ///
    /// Note that BPB_FATSz16 must be 0 for media formatted FAT32.
    bpb_fatsz32: U32,
    /// Set as described below:
    ///
    /// Bits 0-3 -- Zero-based number of active FAT. Only valid if mirroring is
    ///             disabled.
    /// Bits 4-6 -- Reserved.
    /// Bit    7 -- 0 means the FAT is mirrored at runtime into all FATs.
    ///          -- 1 means only one FAT is active; it is the one referenced in
    ///             bits 0-3
    /// Bits 8-15 -- Reserved.
    bpb_extflags: U16,
    /// High byte is major revision number. Low byte is minor revision number.
    /// This is the version number of the FAT32 volume.
    ///
    /// Must be set to 0x0.
    bpb_fsver: U16,
    /// This is set to the cluster number of the first cluster of the root
    /// directory,
    ///
    /// This value should be 2 or the first usable (not bad) cluster available
    /// thereafter.
    bpb_rootclus: U32,
    /// Sector number of FSINFO structure in the reserved area of the FAT32
    /// volume. Usually 1.
    ///
    /// NOTE: There is a copy of the FSINFO structure in the sequence of backup
    /// boot sectors, but only the copy pointed to by this field is kept up to
    /// date (i.e., both the primary and backup boot record point to the same
    /// FSINFO sector).
    bpb_fsinfo: U16,
    /// Set to 0 or 6
    ///
    /// If non-zero, indicates the sector number in the reserved area of the
    /// volume of a copy of the boot record.
    bpb_bkbootsec: U16,
    /// Reserved. Must be set to 0x0.
    bpb_reserved: [u8; 12],

    /// Interrupt 0x13 drive number. Set value to 0x80 or 0x00.
    bs_drvnum: u8,
    /// Reserved. Set value to 0x0.
    bs_reserved1: u8,
    /// Extended boot signature. Set value to 0x29 if either of the following
    /// two fields are non-zero.
    ///
    /// This is a signature byte that indicates that the following three fields
    /// in the boot sector are present.
    bs_bootsig: u8,
    /// Volume serial number.
    ///
    /// This field, together with BS_VolLab, supports volume tracking on
    /// removable media. These values allow FAT file system drivers to detect
    /// that the wrong disk is inserted in a removable drive.
    ///
    /// This ID should be generated by simply combining the current date and
    /// time into a 32-bit value.
    bs_volid: U32,
    /// Volume label. This field matches the 11-byte volume label recorded in
    /// the root directory.
    ///
    /// NOTE: FAT file system drivers must ensure that they update this field
    /// when the volume label file in the root directory has its name changed or
    /// created. The setting for this field when there is no volume label is the
    /// string “NO NAME ”.
    bs_vollab: [u8; 11],
    /// One of the strings “FAT12 ”, “FAT16 ”, or “FAT   ”.
    ///
    /// NOTE: This string is informational only and does not determine the FAT
    /// type.
    bs_filsystype: [u8; 8],

    /// Set to 0x00
    bs_boot: [u8; 420],
    /// Set to 0x55 (at byte offset 510) and 0xAA (at byte offset 511)
    signature_word: [u8; 2],
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout)]
struct DirEntry {
    /// “Short” file name limited to 11 characters (8.3 format).
    dir_name: [u8; 11],
    /// Legal file attribute types are as defined below:
    ///
    /// ATTR_READ_ONLY 0x01
    /// ATTR_HIDDEN 0x02
    /// ATTR_SYSTEM 0x04
    /// ATTR_VOLUME_ID 0x08
    /// ATTR_DIRECTORY 0x10
    /// ATTR_ARCHIVE 0x20
    ///
    /// ATTR_LONG_NAME is defined as follows: (ATTR_READ_ONLY | ATTR_HIDDEN |
    /// ATTR_SYSTEM | ATTR_VOLUME_ID)
    ///
    /// The upper two bits of the attribute byte are reserved and must always be
    /// set to 0 when a file is created. These bits are not interpreted.
    dir_attr: u8,
    /// Reserved. Must be set to 0.
    dir_ntres: u8,
    /// Component of the file creation time. Count of tenths of a second. Valid
    /// range is:
    ///
    /// 0 <= DIR_CrtTimeTenth <= 199
    dir_crttimetenth: u8,
    /// Creation time. Granularity is 2 seconds.
    dir_crttime: U16,
    /// Creation date.
    dir_crtdate: U16,
    /// Last access date. Last access is defined as a read or write operation
    /// performed on the file/directory described by this entry.
    ///
    /// This field must be updated on file modification (write operation) and
    /// the date value must be equal to DIR_WrtDate.
    dir_lstaccdate: U16,
    /// High word of first data cluster number for file/directory described by
    /// this entry.
    ///
    /// Only valid for volumes formatted FAT32. Must be set to 0 on volumes
    /// formatted FAT12/FAT16.
    dir_fstclushi: U16,
    /// Last modification (write) time. Value must be equal to DIR_CrtTime at
    /// file creation.
    dir_wrttime: U16,
    /// Last modification (write) date. Value must be equal to DIR_CrtDate at
    /// file creation.
    dir_wrtdate: U16,
    /// Low word of first data cluster number for file/directory described by
    /// this entry.
    dir_fstcluslo: U16,
    /// 32-bit quantity containing size in bytes of file/directory described by
    /// this entry.
    dir_filesize: U32,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout)]
struct LongNameDirEntry {
    /// The order of this entry in the sequence of long name directory entries
    /// (each containing components of the long file name) associated with the
    /// corresponding short name directory entry.
    ///
    /// The contents of this field must be masked with 0x40 (LAST_LONG_ENTRY)
    /// for the last long directory name entry in the set. Therefore, each
    /// sequence of long name directory entries begins with the contents of this
    /// field masked with LAST_LONG_ENTRY.
    ldir_ord: u8,
    ///  Contains characters 1 through 5 constituting a portion of the long
    /// name.
    ldir_name1: [U16; 5],
    /// Attributes – must be set to ATTR_LONG_NAME
    /// defined as below:
    /// ATTR_LONG_NAME = (ATTR_READ_ONLY | ATTR_HIDDEN | ATTR_SYSTEM |
    /// ATTR_VOLUME_ID)
    ///
    /// NOTE: A mask to determine whether a directory entry is part of the set
    /// of a long name directory entries is defined below: #define
    /// ATTR_LONG_NAME_MASK = (ATTR_READ_ONLY | ATTR_HIDDEN | ATTR_SYSTEM |
    /// ATTR_VOLUME_ID | ATTR_DIRECTORY | ATTR_ARCHIVE)
    ldir_attr: u8,
    /// Reserved. Must be set to 0.
    ldir_type: u8,
    /// Checksum of name in the associated short name directory entry at the end
    /// of the long name directory entry set.
    ldir_chksum: u8,
    /// Contains characters 6 through 11 constituting a portion of the long
    /// name.
    ldir_name2: [U16; 6],
    /// Must be set to 0.
    ldir_fstcluslo: U16,
    /// Contains characters 12 through 13 constituting a portion of the long
    /// name.
    ldir_name3: [U16; 2],
}

fn is_power_of_two(value: u32) -> bool {
    value != 0 && value & (value - 1) == 0
}
