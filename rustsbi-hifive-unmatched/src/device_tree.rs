const DEVICE_TREE_MAGIC: u32 = 0xD00DFEED;

const FDT_BEGIN_NODE: u32 = 0x1;
const FDT_END_NODE: u32 = 0x2;
const FDT_PROP: u32 = 0x3;
const FDT_NOP: u32 = 0x4; 
const FDT_END: u32 = 0x9;

const SUPPORTED_VERSION: u32 = 17;

#[repr(C)]
struct Header {
    magic: u32,
    total_size: u32,
    off_dt_struct: u32,
    off_dt_strings: u32,
    off_mem_rsvmap: u32,
    version: u32,
    last_comp_version: u32,
    boot_cpuid_phys: u32,
    size_dt_strings: u32,
    size_dt_struct: u32,
}

const HEADER_LEN: u32 = core::mem::size_of::<Header>() as u32;

struct DeviceTree {
    header: Header,
    data: [u8]
}

impl DeviceTree {
    pub unsafe fn try_read<'a>(ptr: *const u8) -> Result<&'a DeviceTree, Error> {
        let header = &*(ptr as *const Header);
        let magic = u32::from_be(header.magic);
        if magic != DEVICE_TREE_MAGIC {
            return Err(Error::InvalidMagic)
        }
        let last_comp_version = u32::from_be(header.last_comp_version);
        if last_comp_version > SUPPORTED_VERSION {
            return Err(Error::UnsupportedVersion)
        }
        let total_size = u32::from_be(header.total_size);
        if total_size < HEADER_LEN {
            return Err(Error::HeaderTooShort)
        }
        let off_dt_struct = u32::from_be(header.off_dt_struct);
        if off_dt_struct < HEADER_LEN {
            return Err(Error::HeaderStructureOffset)
        }
        let size_dt_struct = u32::from_be(header.size_dt_struct);
        if off_dt_struct + size_dt_struct > total_size {
            return Err(Error::HeaderStructureLength)
        }
        let off_dt_strings = u32::from_be(header.off_dt_strings);
        if off_dt_strings < HEADER_LEN {
            return Err(Error::HeaderStringTableOffset)
        }
        let size_dt_strings = u32::from_be(header.size_dt_strings);
        if off_dt_struct + size_dt_strings > total_size {
            return Err(Error::HeaderStringTableLength)
        }
        let raw_data_len = (total_size - HEADER_LEN) as usize;
        let ans_ptr = core::ptr::from_raw_parts(ptr as *const (), raw_data_len);
        Ok(&*ans_ptr)
    }

    pub fn tags(&self) -> Tags {
        let structure_addr = (u32::from_be(self.header.off_dt_struct) - HEADER_LEN) as usize;
        let structure_len = u32::from_be(self.header.size_dt_struct) as usize;
        let strings_addr = (u32::from_be(self.header.off_dt_strings) - HEADER_LEN) as usize;
        let strings_len = u32::from_be(self.header.size_dt_strings) as usize;
        Tags { 
            structure: &self.data[structure_addr..structure_addr + structure_len], 
            string_table: &self.data[strings_addr..strings_addr + strings_len], 
            cur: 0 
        }
    }
}

#[derive(Debug)]
pub struct Tags<'a> {
    structure: &'a [u8],
    string_table: &'a [u8],
    cur: usize,
}

impl<'a> Tags<'a> {
    #[inline] fn read_cur_u32(&mut self) -> u32 {
        let ans = u32::from_be_bytes([
            self.structure[self.cur],
            self.structure[self.cur + 1],
            self.structure[self.cur + 2],
            self.structure[self.cur + 3],
        ]);
        self.cur += 4;
        ans
    }
    #[inline] fn read_string0_align(&mut self) -> Result<&'a [u8], Error> {
        let begin = self.cur;
        while self.cur < self.structure.len() {
            if self.structure[self.cur] == b'\0' {
                let end = self.cur;
                self.cur = align_up_u32(end + 1);
                return Ok(&self.structure[begin..end]);
            }
            self.cur += 1;
        }
        Err(Error::StringUnexpectedEndOfData)
    }
    #[inline] fn read_slice_align(&mut self, len: u32) -> Result<&'a [u8], Error> {
        let begin = self.cur;
        let end = self.cur + len as usize;
        if end > self.structure.len() {
            return Err(Error::SliceUnexpectedEndOfData)
        }
        self.cur = align_up_u32(end);
        Ok(&self.structure[begin..end])
    }
    #[inline] fn read_table_string(&mut self, pos: u32) -> Result<&'a [u8], Error> {
        let begin = pos as usize;
        if begin >= self.string_table.len() {
            return Err(Error::TableStringOffset)
        }
        let mut cur = begin;
        while cur < self.string_table.len() {
            if self.string_table[cur] == b'\0' {
                return Ok(&self.string_table[begin..cur]);
            }
            cur += 1;
        }
        return Err(Error::TableStringOffset)
    }
}

impl<'a> Iterator for Tags<'a> {
    type Item = Result<Tag<'a>, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur > self.structure.len() - core::mem::size_of::<u32>() {
            return Some(Err(Error::TagUnexpectedEndOfData))
        }
        loop {
            match self.read_cur_u32() {
                FDT_BEGIN_NODE => match self.read_string0_align() {
                    Ok(name) => {
                        // println!("cur = {}", self.cur);
                        break Some(Ok(Tag::Begin(name)))
                    },
                    Err(e) => break Some(Err(e)),
                },
                FDT_PROP  => {
                    let val_size = self.read_cur_u32();
                    let name_offset = self.read_cur_u32();
                    // println!("size {}, off {}", val_size, name_offset);
                    // get value slice
                    let val = match self.read_slice_align(val_size) {
                        Ok(slice) => slice,
                        Err(e) => break Some(Err(e)),
                    };

                    // lookup name in strings table
                    let prop_name = match self.read_table_string(name_offset) {
                        Ok(slice) => slice,
                        Err(e) => break Some(Err(e)),
                    };
                    break Some(Ok(Tag::Prop(val, prop_name)))
                }
                FDT_END_NODE => {
                    break Some(Ok(Tag::End))
                },
                FDT_NOP => self.cur += 4,
                FDT_END => break None,
                _ => break Some(Err(Error::StructureBegin))
            }
        }
    }
}

#[derive(Debug)]
pub enum Tag<'a> {
    Begin(&'a [u8]),
    Prop(&'a [u8], &'a [u8]),
    End,
}

#[derive(Debug)]
pub enum Error {
    InvalidMagic,
    HeaderTooShort,
    UnsupportedVersion,
    HeaderStructureOffset,
    HeaderStructureLength,
    HeaderStringTableOffset,
    HeaderStringTableLength,
    StructureBegin,
    StringUnexpectedEndOfData,
    TagUnexpectedEndOfData,
    SliceUnexpectedEndOfData,
    TableStringOffset,
}

#[inline]
pub fn align_up_u32(val: usize) -> usize {
    val + (4 - (val % 4)) % 4
}

use crate::peripheral::Uart;

pub struct ChosenDevice {
    pub stdio: Uart,
}

pub unsafe fn parse_device_tree(dtb_pa: usize) -> Result<ChosenDevice, Error> {
    let tree = DeviceTree::try_read(dtb_pa as *const u8)?;
    use crate::console::println;
    for tag in tree.tags() {
        match tag? {
            Tag::Begin(name) => {
                println!("Begin {}", core::str::from_utf8_unchecked(name));
            },
            Tag::Prop(val, name) => {
                println!("Prop {}, {}", 
                core::str::from_utf8_unchecked(name),
                core::str::from_utf8_unchecked(val)
            );
                
            },
            Tag::End => {
                println!("End");
            },
        }
    }
    todo!()
}
