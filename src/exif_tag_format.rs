// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use crate::rational::*;

pub type INT8U          = Vec<u8>;
pub type STRING         = String;
pub type INT16U         = Vec<u16>;
pub type INT32U         = Vec<u32>;
pub type RATIONAL64U    = Vec<uR64>;
pub type INT8S          = Vec<i8>;
pub type UNDEF          = Vec<u8>;      // got no better idea for this atm
pub type INT16S         = Vec<i16>;
pub type INT32S         = Vec<i32>;
pub type RATIONAL64S    = Vec<iR64>;
pub type FLOAT          = Vec<f32>;
pub type DOUBLE         = Vec<f64>;

/// A UTF-16LE encoded string. Used for Windows XP tags (XPTitle, XPKeywords,
/// XPSubject) and the `UnknownUTF16` variant. The inner `String` is always
/// stored as UTF-8 in memory; encoding/decoding to/from UTF-16LE bytes is
/// handled by the `U8conversion` implementation.
#[derive(Clone, Debug, PartialEq)]
pub struct Utf16String(pub String);

pub type UTF16 = Utf16String;

impl Utf16String
{
	pub fn new() -> Self { Utf16String(String::new()) }

	/// Returns the byte length of the full UTF-16LE encoding, **including**
	/// the two-byte null terminator. This is used as the EXIF component count.
	pub fn len(&self) -> usize
	{
		(self.0.encode_utf16().count() + 1) * 2
	}
}

impl Default for Utf16String
{
	fn default() -> Self { Utf16String::new() }
}

impl From<String> for Utf16String
{
	fn from(s: String) -> Self { Utf16String(s) }
}

impl From<&str> for Utf16String
{
	fn from(s: &str) -> Self { Utf16String(s.to_owned()) }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum
ExifTagFormat
{
	INT8U,          // unsigned byte        int8u
	STRING,         // ascii string         string
	INT16U,         // unsigned short       int16u
	INT32U,         // unsigned long        int32u
	RATIONAL64U,    // unsigned rational    rational64u
	INT8S,          // signed byte          int8s
	UNDEF,          // undefined            undef
	INT16S,         // signed short         int16s
	INT32S,         // signed long          int32s
	RATIONAL64S,    // signed rational      rational64s
	FLOAT,          // single float         float
	DOUBLE,         // double float         double
	UTF16,          // utf-16le string, stored as BYTE (0x0001) in EXIF files
}

impl 
ExifTagFormat
{

	pub fn
	as_u16
	(
		&self
	)
	-> u16
	{
		match *self
		{
			ExifTagFormat::INT8U        => 0x0001,
			ExifTagFormat::STRING       => 0x0002,
			ExifTagFormat::INT16U       => 0x0003,
			ExifTagFormat::INT32U       => 0x0004,
			ExifTagFormat::RATIONAL64U  => 0x0005,
			ExifTagFormat::INT8S        => 0x0006,
			ExifTagFormat::UNDEF        => 0x0007,
			ExifTagFormat::INT16S       => 0x0008,
			ExifTagFormat::INT32S       => 0x0009,
			ExifTagFormat::RATIONAL64S  => 0x000a,
			ExifTagFormat::FLOAT        => 0x000b,
			ExifTagFormat::DOUBLE       => 0x000c,
			// UTF-16LE strings are stored as BYTE in EXIF files
			ExifTagFormat::UTF16        => 0x0001,
		}
	}

	pub fn
	from_u16
	(
		hex_code: u16
	)
	-> Option<ExifTagFormat>
	{
		match hex_code
		{
			0x0001  => Some(ExifTagFormat::INT8U),
			0x0002  => Some(ExifTagFormat::STRING),
			0x0003  => Some(ExifTagFormat::INT16U),
			0x0004  => Some(ExifTagFormat::INT32U),
			0x0005  => Some(ExifTagFormat::RATIONAL64U),
			0x0006  => Some(ExifTagFormat::INT8S),
			0x0007  => Some(ExifTagFormat::UNDEF),
			0x0008  => Some(ExifTagFormat::INT16S),
			0x0009  => Some(ExifTagFormat::INT32S),
			0x000a  => Some(ExifTagFormat::RATIONAL64S),
			0x000b  => Some(ExifTagFormat::FLOAT),
			0x000c  => Some(ExifTagFormat::DOUBLE),
			_       => None,
		}
	}


	pub fn
	bytes_per_component
	(
		&self
	)
	-> u32
	{
		match self
		{
			ExifTagFormat::INT8U        => 1,
			ExifTagFormat::STRING       => 1,
			ExifTagFormat::INT16U       => 2,
			ExifTagFormat::INT32U       => 4,
			ExifTagFormat::RATIONAL64U  => 8,
			ExifTagFormat::INT8S        => 1,
			ExifTagFormat::UNDEF        => 1,
			ExifTagFormat::INT16S       => 2,
			ExifTagFormat::INT32S       => 4,
			ExifTagFormat::RATIONAL64S  => 8,
			ExifTagFormat::FLOAT        => 4,
			ExifTagFormat::DOUBLE       => 8,
			ExifTagFormat::UTF16        => 1,
		}
	}
}
