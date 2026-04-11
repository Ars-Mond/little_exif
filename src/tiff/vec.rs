// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Cursor;

use crate::exif_tag::ExifTag;
use crate::ifd::ExifTagGroup;
use crate::metadata::Metadata;

use super::generic_read_metadata;
use super::generic_write_metadata;

pub(crate) fn
read_metadata
(
	file_buffer: &Vec<u8>
)
-> Result<Vec<u8>, std::io::Error>
{
	let mut cursor = Cursor::new(file_buffer);
	return generic_read_metadata(&mut cursor);
}

pub(crate) fn
clear_metadata
(
	file_buffer: &mut Vec<u8>
)
-> Result<(), std::io::Error>
{
	// Create cursor
	let mut cursor           = Cursor::new(file_buffer);
	let     cursor_start_pos = cursor.position();

	// Read in the data
	let     raw_data = generic_read_metadata(&mut cursor);
	let mut data     = Metadata::general_decoding_wrapper(raw_data)?;

	// Remove all IFDs that aren't required
	data.reduce_to_a_minimum();

	// Write the reduced data back to the backup cursor
	cursor.set_position(cursor_start_pos);
	return generic_write_metadata(&mut cursor, &data);
}

pub(crate) fn
write_metadata
(
	file_buffer: &mut Vec<u8>,
	metadata:    &Metadata
)
-> Result<(), std::io::Error>
{
	let mut cursor = Cursor::new(file_buffer);

	// If IPTC or XMP data is present, inject them as TIFF tags before encoding
	if metadata.get_iptc().is_some() || metadata.get_xmp().is_some()
	{
		let mut m = metadata.clone();
		if let Some(raw) = metadata.get_iptc().map(|d| d.encode())
		{
			m.get_ifd_mut(ExifTagGroup::GENERIC, 0)
				.set_tag(ExifTag::UnknownUNDEF(raw, 0x83BB, ExifTagGroup::GENERIC));
		}
		if let Some(raw) = metadata.get_xmp().map(|d| d.as_bytes().to_vec())
		{
			m.get_ifd_mut(ExifTagGroup::GENERIC, 0)
				.set_tag(ExifTag::UnknownUNDEF(raw, 0x02BC, ExifTagGroup::GENERIC));
		}
		return generic_write_metadata(&mut cursor, &m);
	}

	return generic_write_metadata(&mut cursor, metadata);
}