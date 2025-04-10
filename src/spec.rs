use crate::result::{ZipError, ZipResult};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io;
use std::io::prelude::*;

pub const LOCAL_FILE_HEADER_SIGNATURE: u32 = 0x04034b50;
pub const LOCAL_ENC_FILE_HEADER_SIGNATURE: u32 = 0x14034b50;
pub const CENTRAL_DIRECTORY_HEADER_SIGNATURE: u32 = 0x02014b50;
const CENTRAL_DIRECTORY_END_SIGNATURE: u32 = 0x06054b50;
pub const ZIP64_CENTRAL_DIRECTORY_END_SIGNATURE: u32 = 0x06064b50;
const ZIP64_CENTRAL_DIRECTORY_END_LOCATOR_SIGNATURE: u32 = 0x07064b50;

pub struct CentralDirectoryEnd {
    pub disk_number: u16,
    pub disk_with_central_directory: u16,
    pub number_of_files_on_this_disk: u16,
    pub number_of_files: u16,
    pub central_directory_size: u32,
    pub central_directory_offset: u32,
    pub zip_file_comment: Vec<u8>,
}

pub fn find_sig_loc<T: Read + io::Seek>(
    reader: &mut T,
    signature: u32,
    end_loc: i64,
    min_block_size: i64,
    max_var_data: i64,
) -> Option<u64> {
    let pos = end_loc - min_block_size;
    if pos < 0 {
        return None;
    }

    let give_up_marker = (pos - max_var_data).max(0) as u64;
    let mut pos = pos as u64;

    loop  {
        if pos < give_up_marker {
            return None;
        }
        pos -= 1;
        reader.seek(io::SeekFrom::Start(pos)).unwrap();
        
        if reader.read_u32::<LittleEndian>().unwrap() == signature as u32 {
            return Some(pos);
        }
    }
}

impl CentralDirectoryEnd {
    pub fn parse<T: Read>(reader: &mut T) -> ZipResult<CentralDirectoryEnd> {
        let magic = reader.read_u32::<LittleEndian>()?;
        if magic != CENTRAL_DIRECTORY_END_SIGNATURE {
            return Err(ZipError::InvalidArchive("Invalid digital signature header"));
        }
        let disk_number = reader.read_u16::<LittleEndian>()?;
        let disk_with_central_directory = reader.read_u16::<LittleEndian>()?;
        let number_of_files_on_this_disk = reader.read_u16::<LittleEndian>()?;
        let number_of_files = reader.read_u16::<LittleEndian>()?;
        let central_directory_size = reader.read_u32::<LittleEndian>()?;
        let central_directory_offset = reader.read_u32::<LittleEndian>()?;
        let zip_file_comment_length = reader.read_u16::<LittleEndian>()? as usize;
        let mut zip_file_comment = vec![0; zip_file_comment_length];
        reader.read_exact(&mut zip_file_comment)?;

        Ok(CentralDirectoryEnd {
            disk_number,
            disk_with_central_directory,
            number_of_files_on_this_disk,
            number_of_files,
            central_directory_size,
            central_directory_offset,
            zip_file_comment,
        })
    }

    pub fn find_and_parse<T: Read + io::Seek>(
        reader: &mut T,
    ) -> ZipResult<(CentralDirectoryEnd, u64)> {
        const HEADER_SIZE: u64 = 22;
        const BYTES_BETWEEN_MAGIC_AND_COMMENT_SIZE: u64 = HEADER_SIZE - 6;
        let file_length = reader.seek(io::SeekFrom::End(0))?;

        let search_upper_bound = file_length.saturating_sub(HEADER_SIZE + ::std::u16::MAX as u64);

        if file_length < HEADER_SIZE {
            return Err(ZipError::InvalidArchive("Invalid zip header"));
        }

        let mut pos = file_length - HEADER_SIZE;
        while pos >= search_upper_bound {
            reader.seek(io::SeekFrom::Start(pos as u64))?;
            if reader.read_u32::<LittleEndian>()? == CENTRAL_DIRECTORY_END_SIGNATURE {
                reader.seek(io::SeekFrom::Current(
                    BYTES_BETWEEN_MAGIC_AND_COMMENT_SIZE as i64,
                ))?;
                let cde_start_pos = reader.seek(io::SeekFrom::Start(pos as u64))?;
                return CentralDirectoryEnd::parse(reader).map(|cde| (cde, cde_start_pos));
            }
            pos = match pos.checked_sub(1) {
                Some(p) => p,
                None => break,
            };
        }
        Err(ZipError::InvalidArchive(
            "Could not find central directory end",
        ))
    }

    pub fn write<T: Write>(&self, writer: &mut T) -> ZipResult<()> {
        writer.write_u32::<LittleEndian>(CENTRAL_DIRECTORY_END_SIGNATURE)?;
        writer.write_u16::<LittleEndian>(self.disk_number)?;
        writer.write_u16::<LittleEndian>(self.disk_with_central_directory)?;
        writer.write_u16::<LittleEndian>(self.number_of_files_on_this_disk)?;
        writer.write_u16::<LittleEndian>(self.number_of_files)?;
        writer.write_u32::<LittleEndian>(self.central_directory_size)?;
        writer.write_u32::<LittleEndian>(self.central_directory_offset)?;
        writer.write_u16::<LittleEndian>(self.zip_file_comment.len() as u16)?;
        writer.write_all(&self.zip_file_comment)?;
        Ok(())
    }
}

pub struct Zip64CentralDirectoryEndLocator {
    pub disk_with_central_directory: u32,
    pub end_of_central_directory_offset: u64,
    pub number_of_disks: u32,
}

impl Zip64CentralDirectoryEndLocator {
    pub fn find_and_parse<T: Read + io::Seek>(
        reader: &mut T,
        cde_start_pos: u64,
    ) -> ZipResult<Zip64CentralDirectoryEndLocator> {
        find_sig_loc(
            reader,
            ZIP64_CENTRAL_DIRECTORY_END_LOCATOR_SIGNATURE,
            cde_start_pos as i64,
            0,
            0x0100,
        ).ok_or(ZipError::InvalidArchive("Invalid zip64 locator digital signature header"))?;

        let disk_with_central_directory = reader.read_u32::<LittleEndian>()?;
        let end_of_central_directory_offset = reader.read_u64::<LittleEndian>()?;
        let number_of_disks = reader.read_u32::<LittleEndian>()?;

        Ok(Zip64CentralDirectoryEndLocator {
            disk_with_central_directory,
            end_of_central_directory_offset,
            number_of_disks,
        })
    }

    pub fn write<T: Write>(&self, writer: &mut T) -> ZipResult<()> {
        writer.write_u32::<LittleEndian>(ZIP64_CENTRAL_DIRECTORY_END_LOCATOR_SIGNATURE)?;
        writer.write_u32::<LittleEndian>(self.disk_with_central_directory)?;
        writer.write_u64::<LittleEndian>(self.end_of_central_directory_offset)?;
        writer.write_u32::<LittleEndian>(self.number_of_disks)?;
        Ok(())
    }
}

pub struct Zip64CentralDirectoryEnd {
    pub version_made_by: u16,
    pub version_needed_to_extract: u16,
    pub disk_number: u32,
    pub disk_with_central_directory: u32,
    pub number_of_files_on_this_disk: u64,
    pub number_of_files: u64,
    pub central_directory_size: u64,
    pub central_directory_offset: u64,
    //pub extensible_data_sector: Vec<u8>, <-- We don't do anything with this at the moment.
}

impl Zip64CentralDirectoryEnd {
    pub fn find_and_parse<T: Read + io::Seek>(
        reader: &mut T,
        nominal_offset: u64,
        search_upper_bound: u64,
    ) -> ZipResult<(Zip64CentralDirectoryEnd, u64)> {
        let mut pos = nominal_offset;

        while pos <= search_upper_bound {
            reader.seek(io::SeekFrom::Start(pos))?;

            if reader.read_u32::<LittleEndian>()? == ZIP64_CENTRAL_DIRECTORY_END_SIGNATURE {
                let archive_offset = pos - nominal_offset;

                let _record_size = reader.read_u64::<LittleEndian>()?;
                // We would use this value if we did anything with the "zip64 extensible data sector".

                let version_made_by = reader.read_u16::<LittleEndian>()?;
                let version_needed_to_extract = reader.read_u16::<LittleEndian>()?;
                let disk_number = reader.read_u32::<LittleEndian>()?;
                let disk_with_central_directory = reader.read_u32::<LittleEndian>()?;
                let number_of_files_on_this_disk = reader.read_u64::<LittleEndian>()?;
                let number_of_files = reader.read_u64::<LittleEndian>()?;
                let central_directory_size = reader.read_u64::<LittleEndian>()?;
                let central_directory_offset = reader.read_u64::<LittleEndian>()?;

                return Ok((
                    Zip64CentralDirectoryEnd {
                        version_made_by,
                        version_needed_to_extract,
                        disk_number,
                        disk_with_central_directory,
                        number_of_files_on_this_disk,
                        number_of_files,
                        central_directory_size,
                        central_directory_offset,
                    },
                    archive_offset,
                ));
            }

            pos += 1;
        }

        Err(ZipError::InvalidArchive(
            "Could not find ZIP64 central directory end",
        ))
    }

    pub fn write<T: Write>(&self, writer: &mut T) -> ZipResult<()> {
        writer.write_u32::<LittleEndian>(ZIP64_CENTRAL_DIRECTORY_END_SIGNATURE)?;
        writer.write_u64::<LittleEndian>(44)?; // record size
        writer.write_u16::<LittleEndian>(self.version_made_by)?;
        writer.write_u16::<LittleEndian>(self.version_needed_to_extract)?;
        writer.write_u32::<LittleEndian>(self.disk_number)?;
        writer.write_u32::<LittleEndian>(self.disk_with_central_directory)?;
        writer.write_u64::<LittleEndian>(self.number_of_files_on_this_disk)?;
        writer.write_u64::<LittleEndian>(self.number_of_files)?;
        writer.write_u64::<LittleEndian>(self.central_directory_size)?;
        writer.write_u64::<LittleEndian>(self.central_directory_offset)?;
        Ok(())
    }
}
