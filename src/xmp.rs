// Copyright © 2024-2026 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io;
use std::io::Cursor;

use quick_xml::events::BytesStart;
use quick_xml::events::Event;
use quick_xml::Reader;
use quick_xml::Writer;

use crate::endian::Endian;
use crate::exif_tag::ExifTag;
use crate::rational::iR64;
use crate::rational::uR64;

/// Namespace URI used in JPEG APP1 to identify XMP segments (includes NUL terminator).
/// 29 bytes total.
pub const XMP_NAMESPACE_URI: &[u8] = b"http://ns.adobe.com/xap/1.0/\0";

/// Minimal valid XMP packet with no fields.
const XMP_TEMPLATE: &str = concat!(
	"<?xpacket begin=\"\" id=\"W5M0MpCehiHzreSzNTczkc9d\"?>",
	"<x:xmpmeta xmlns:x=\"adobe:ns:meta/\">",
	"<rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">",
	"</rdf:RDF>",
	"</x:xmpmeta>",
	"<?xpacket end=\"w\"?>"
);

/// Holds a raw XMP XML packet (UTF-8 bytes).
#[derive(Clone, Debug)]
pub struct XmpData
{
	pub packet: Vec<u8>,
}

impl
XmpData
{
	/// Creates a minimal valid XMP packet with no fields.
	pub fn
	new
	()
	-> Self
	{
		XmpData { packet: XMP_TEMPLATE.as_bytes().to_vec() }
	}

	/// Wraps existing raw XMP bytes without parsing them.
	pub fn
	from_raw
	(
		packet: Vec<u8>
	)
	-> Self
	{
		XmpData { packet }
	}

	/// Returns a reference to the raw XMP bytes.
	pub fn
	as_bytes
	(
		&self
	)
	-> &[u8]
	{
		&self.packet
	}

	/// Parses exif: namespace attributes from the XMP packet and returns them
	/// as ExifTag values. Handles both inline attributes and child elements.
	pub fn
	get_exif_tags
	(
		&self,
		endian: &Endian
	)
	-> Vec<ExifTag>
	{
		parse_exif_from_xmp(&self.packet, endian)
	}

	/// Replaces the exif: section in the XMP packet with the given ExifTag values.
	///
	/// 1. Strips any existing exif: elements/attributes.
	/// 2. Builds a new `rdf:Description` element with `exif:` attributes.
	/// 3. Inserts it before `</rdf:RDF>`.
	pub fn
	set_exif_tags
	(
		&mut self,
		tags:   &[ExifTag],
		endian: &Endian
	)
	-> io::Result<()>
	{
		let clean = remove_exif_from_xmp(&self.packet)
			.map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

		if tags.is_empty()
		{
			self.packet = clean;
			return Ok(());
		}

		let description = build_exif_description(tags, endian);
		let marker      = b"</rdf:RDF>";

		if let Some(pos) = find_subsequence(&clean, marker)
		{
			let mut new_packet = clean[..pos].to_vec();
			new_packet.extend_from_slice(description.as_bytes());
			new_packet.extend_from_slice(&clean[pos..]);
			self.packet = new_packet;
		}
		else
		{
			// Fallback: append without RDF wrapper
			self.packet = clean;
			self.packet.extend_from_slice(description.as_bytes());
		}

		Ok(())
	}
}

impl
Default
for
XmpData
{
	fn default() -> Self { Self::new() }
}

// ─── Internal helpers ────────────────────────────────────────────────────────

fn
find_subsequence
(
	haystack: &[u8],
	needle:   &[u8]
)
-> Option<usize>
{
	haystack.windows(needle.len()).position(|w| w == needle)
}

fn
xml_escape
(
	s: &str
)
-> String
{
	s.replace('&', "&amp;")
	 .replace('"',  "&quot;")
	 .replace('<',  "&lt;")
	 .replace('>',  "&gt;")
}

fn
rational_u_to_xmp
(
	r: &uR64
)
-> String
{
	format!("{}/{}", r.nominator, r.denominator)
}

fn
rational_i_to_xmp
(
	r: &iR64
)
-> String
{
	format!("{}/{}", r.nominator, r.denominator)
}

fn
xmp_int_to_u16
(
	s: &str
)
-> Option<u16>
{
	s.trim().parse().ok()
}

fn
xmp_int_to_u32
(
	s: &str
)
-> Option<u32>
{
	s.trim().parse().ok()
}

fn
xmp_rational_to_ur64
(
	s: &str
)
-> Option<uR64>
{
	let s = s.trim();
	if let Some(pos) = s.find('/')
	{
		let nom: u32 = s[..pos].trim().parse().ok()?;
		let den: u32 = s[pos+1..].trim().parse().ok()?;
		if den == 0 { return None; }
		Some(uR64 { nominator: nom, denominator: den })
	}
	else
	{
		let f: f64 = s.parse().ok()?;
		Some(uR64::from(f))
	}
}

fn
xmp_rational_to_ir64
(
	s: &str
)
-> Option<iR64>
{
	let s = s.trim();
	if let Some(pos) = s.find('/')
	{
		let nom: i32 = s[..pos].trim().parse().ok()?;
		let den: i32 = s[pos+1..].trim().parse().ok()?;
		if den == 0 { return None; }
		Some(iR64 { nominator: nom, denominator: den })
	}
	else
	{
		let f: f64 = s.parse().ok()?;
		Some(iR64::from(f))
	}
}

fn
xmp_property_to_exif_tag
(
	name:   &str,
	value:  &str,
	_endian: &Endian
)
-> Option<ExifTag>
{
	match name
	{
		"ImageDescription" => Some(ExifTag::ImageDescription(value.to_string())),
		"Make"             => Some(ExifTag::Make(value.to_string())),
		"Model"            => Some(ExifTag::Model(value.to_string())),
		"Software"         => Some(ExifTag::Software(value.to_string())),
		// XMP exif:DateTime corresponds to EXIF ModifyDate (0x0132)
		"DateTime"         => Some(ExifTag::ModifyDate(value.to_string())),
		"Artist"           => Some(ExifTag::Artist(value.to_string())),
		"Copyright"        => Some(ExifTag::Copyright(value.to_string())),
		"DateTimeOriginal" => Some(ExifTag::DateTimeOriginal(value.to_string())),
		// XMP exif:DateTimeDigitized corresponds to EXIF CreateDate (0x9004)
		"DateTimeDigitized" => Some(ExifTag::CreateDate(value.to_string())),

		"Orientation"           => xmp_int_to_u16(value).map(|v| ExifTag::Orientation(vec![v])),
		"ExposureProgram"       => xmp_int_to_u16(value).map(|v| ExifTag::ExposureProgram(vec![v])),
		// exif:ISOSpeedRatings — XMP standard name for ISO (0x8827)
		"ISOSpeedRatings"       => xmp_int_to_u16(value).map(|v| ExifTag::ISO(vec![v])),
		"MeteringMode"          => xmp_int_to_u16(value).map(|v| ExifTag::MeteringMode(vec![v])),
		"Flash"                 => xmp_int_to_u16(value).map(|v| ExifTag::Flash(vec![v])),
		"ColorSpace"            => xmp_int_to_u16(value).map(|v| ExifTag::ColorSpace(vec![v])),
		"ExposureMode"          => xmp_int_to_u16(value).map(|v| ExifTag::ExposureMode(vec![v])),
		"WhiteBalance"          => xmp_int_to_u16(value).map(|v| ExifTag::WhiteBalance(vec![v])),
		"FocalLengthIn35mmFilm" => xmp_int_to_u16(value).map(|v| ExifTag::FocalLengthIn35mmFormat(vec![v])),
		"SceneCaptureType"      => xmp_int_to_u16(value).map(|v| ExifTag::SceneCaptureType(vec![v])),

		"ExposureTime"      => xmp_rational_to_ur64(value).map(|v| ExifTag::ExposureTime(vec![v])),
		"FNumber"           => xmp_rational_to_ur64(value).map(|v| ExifTag::FNumber(vec![v])),
		"FocalLength"       => xmp_rational_to_ur64(value).map(|v| ExifTag::FocalLength(vec![v])),
		"ApertureValue"     => xmp_rational_to_ur64(value).map(|v| ExifTag::ApertureValue(vec![v])),

		"ShutterSpeedValue" => xmp_rational_to_ir64(value).map(|v| ExifTag::ShutterSpeedValue(vec![v])),
		// XMP exif:ExposureBiasValue corresponds to EXIF ExposureCompensation (0x9204)
		"ExposureBiasValue" => xmp_rational_to_ir64(value).map(|v| ExifTag::ExposureCompensation(vec![v])),

		"PixelXDimension"   => xmp_int_to_u32(value).map(|v| ExifTag::ExifImageWidth(vec![v])),
		"PixelYDimension"   => xmp_int_to_u32(value).map(|v| ExifTag::ExifImageHeight(vec![v])),

		_ => None,
	}
}

fn
exif_tag_to_xmp_attribute
(
	tag: &ExifTag
)
-> Option<(&'static str, String)>
{
	match tag
	{
		ExifTag::ImageDescription(s) => Some(("ImageDescription", s.clone())),
		ExifTag::Make(s)             => Some(("Make",             s.clone())),
		ExifTag::Model(s)            => Some(("Model",            s.clone())),
		ExifTag::Software(s)         => Some(("Software",         s.clone())),
		// EXIF ModifyDate (0x0132) → XMP exif:DateTime
		ExifTag::ModifyDate(s)       => Some(("DateTime",         s.clone())),
		ExifTag::Artist(s)           => Some(("Artist",           s.clone())),
		ExifTag::Copyright(s)        => Some(("Copyright",        s.clone())),
		ExifTag::DateTimeOriginal(s) => Some(("DateTimeOriginal", s.clone())),
		// EXIF CreateDate (0x9004) → XMP exif:DateTimeDigitized
		ExifTag::CreateDate(s)       => Some(("DateTimeDigitized", s.clone())),

		ExifTag::Orientation(v)             if !v.is_empty() => Some(("Orientation",           v[0].to_string())),
		ExifTag::ExposureProgram(v)         if !v.is_empty() => Some(("ExposureProgram",        v[0].to_string())),
		ExifTag::ISO(v)                     if !v.is_empty() => Some(("ISOSpeedRatings",        v[0].to_string())),
		ExifTag::MeteringMode(v)            if !v.is_empty() => Some(("MeteringMode",           v[0].to_string())),
		ExifTag::Flash(v)                   if !v.is_empty() => Some(("Flash",                  v[0].to_string())),
		ExifTag::ColorSpace(v)              if !v.is_empty() => Some(("ColorSpace",             v[0].to_string())),
		ExifTag::ExposureMode(v)            if !v.is_empty() => Some(("ExposureMode",           v[0].to_string())),
		ExifTag::WhiteBalance(v)            if !v.is_empty() => Some(("WhiteBalance",           v[0].to_string())),
		ExifTag::FocalLengthIn35mmFormat(v) if !v.is_empty() => Some(("FocalLengthIn35mmFilm",  v[0].to_string())),
		ExifTag::SceneCaptureType(v)        if !v.is_empty() => Some(("SceneCaptureType",       v[0].to_string())),

		ExifTag::ExposureTime(v)   if !v.is_empty() => Some(("ExposureTime",     rational_u_to_xmp(&v[0]))),
		ExifTag::FNumber(v)        if !v.is_empty() => Some(("FNumber",          rational_u_to_xmp(&v[0]))),
		ExifTag::FocalLength(v)    if !v.is_empty() => Some(("FocalLength",      rational_u_to_xmp(&v[0]))),
		ExifTag::ApertureValue(v)  if !v.is_empty() => Some(("ApertureValue",    rational_u_to_xmp(&v[0]))),

		ExifTag::ShutterSpeedValue(v)    if !v.is_empty() => Some(("ShutterSpeedValue", rational_i_to_xmp(&v[0]))),
		// EXIF ExposureCompensation (0x9204) → XMP exif:ExposureBiasValue
		ExifTag::ExposureCompensation(v) if !v.is_empty() => Some(("ExposureBiasValue", rational_i_to_xmp(&v[0]))),

		ExifTag::ExifImageWidth(v)  if !v.is_empty() => Some(("PixelXDimension", v[0].to_string())),
		ExifTag::ExifImageHeight(v) if !v.is_empty() => Some(("PixelYDimension", v[0].to_string())),

		_ => None,
	}
}

fn
build_exif_description
(
	tags:   &[ExifTag],
	_endian: &Endian
)
-> String
{
	let mut attrs = String::from(" xmlns:exif=\"http://ns.adobe.com/exif/1.0/\"");

	for tag in tags
	{
		if let Some((name, value)) = exif_tag_to_xmp_attribute(tag)
		{
			attrs.push_str(&format!(" exif:{}=\"{}\"", name, xml_escape(&value)));
		}
	}

	format!("<rdf:Description rdf:about=\"\"{}/>\n", attrs)
}

fn
parse_exif_from_xmp
(
	data:   &[u8],
	endian: &Endian
)
-> Vec<ExifTag>
{
	let mut tags   = Vec::new();
	let mut reader = Reader::from_reader(data);
	let mut buf    = Vec::new();

	// Track whether we are inside an <exif:PropName> child element
	let mut current_exif_prop: Option<String> = None;

	loop
	{
		match reader.read_event_into(&mut buf)
		{
			Ok(Event::Start(ref e)) =>
			{
				let elem_name = String::from_utf8_lossy(e.name().0).into_owned();

				if elem_name.starts_with("exif:")
				{
					// Child-element form: <exif:Make>...</exif:Make>
					current_exif_prop = Some(elem_name["exif:".len()..].to_owned());
				}
				else
				{
					// Attribute form: exif:Make="..." on other elements (e.g. rdf:Description)
					collect_exif_attrs(e, endian, &mut tags);
				}
			}

			Ok(Event::Empty(ref e)) =>
			{
				let elem_name = String::from_utf8_lossy(e.name().0).into_owned();
				if !elem_name.starts_with("exif:")
				{
					collect_exif_attrs(e, endian, &mut tags);
				}
			}

			Ok(Event::Text(ref e)) =>
			{
				if let Some(ref prop) = current_exif_prop.take()
				{
					let value = String::from_utf8_lossy(&e.to_vec()).into_owned();
					if let Some(tag) = xmp_property_to_exif_tag(prop, value.trim(), endian)
					{
						tags.push(tag);
					}
				}
			}

			Ok(Event::End(_)) =>
			{
				current_exif_prop = None;
			}

			Ok(Event::Eof) | Err(_) => break,

			_ => {}
		}

		buf.clear();
	}

	tags
}

/// Collects all `exif:*` attributes from an element into the given tag vec.
fn
collect_exif_attrs
(
	elem:   &BytesStart<'_>,
	endian: &Endian,
	tags:   &mut Vec<ExifTag>
)
{
	for attr in elem.attributes().filter_map(Result::ok)
	{
		let key = match std::str::from_utf8(attr.key.as_ref())
		{
			Ok(k) => k.to_owned(),
			Err(_) => continue,
		};

		if !key.starts_with("exif:") { continue; }

		let prop_name = key["exif:".len()..].to_owned();

		// Try to unescape XML entities; fall back to raw bytes on failure
		let value = attr.unescape_value()
			.map(|v| v.into_owned())
			.unwrap_or_else(|_| String::from_utf8_lossy(&attr.value).into_owned());

		if let Some(tag) = xmp_property_to_exif_tag(&prop_name, &value, endian)
		{
			tags.push(tag);
		}
	}
}

// ─── Remove exif: data from XMP (kept for PNG compatibility) ─────────────────

/// Removes all exif: elements and attributes from raw XMP data.
/// Used by PNG's `clear_exif_from_xmp_metadata` when clearing EXIF metadata.
pub(crate) fn
remove_exif_from_xmp
(
	data: &[u8]
)
-> Result<Vec<u8>, Box<dyn std::error::Error>>
{
	let mut reader = Reader::from_reader(data);
	let mut writer = Writer::new(Cursor::new(Vec::new()));

	// Needed by the reader
	let mut read_buffer = Vec::new();

	// Needed for skipping stuff like
	// <exif:Description>Hi</exif:Description>\n
	let mut skip_depth   = 0u32;
	let mut skip_next_nl = false;

	loop
	{
		// Read in the event
		let read_event = reader.read_event_into(&mut read_buffer);

		match read_event
		{
			Ok(Event::Start(ref event)) => {
				let event_name = String::from_utf8(event.name().0.to_vec())?;

				if event_name.starts_with("exif:")
				{
					skip_depth += 1;
				}
				else if skip_depth == 0
				{
					writer.write_event(Event::Start(get_exif_filtered_event(event)?))?;
				}
			}

			Ok(Event::Empty(ref event)) => {
				let event_name = String::from_utf8(event.name().0.to_vec())?;

				if event_name.starts_with("exif:")
				{
					// do nothing
				}
				else if skip_depth == 0
				{
					writer.write_event(Event::Empty(get_exif_filtered_event(event)?))?;
				}
			}

			Ok(Event::End(ref event)) => {
				if skip_depth > 0
				{
					skip_depth  -= 1;
					skip_next_nl = true;
				}
				else
				{
					writer.write_event(Event::End(event.clone()))?;
				}
			}

			Ok(Event::Eof) => {
				assert_eq!(skip_depth, 0);
				break;
			}

			Ok(Event::Text(ref event)) => {
				let event_string = String::from_utf8(event.to_vec())?;

				let characters = event_string.chars()
					.filter(|c| *c == '\n' || !c.is_whitespace())
					.collect::<Vec<char>>();

				if characters == vec!['\n'] && skip_next_nl
				{
					skip_next_nl = false;
				}
				else if skip_depth == 0
				{
					writer.write_event(Event::Text(event.clone()))?;
				}
			}

			Ok(other_event) => {
				if skip_depth == 0 { writer.write_event(other_event)?; }
			}

			Err(error_message) => {
				log::error!(
					"Error at position {}: {:?}",
					reader.buffer_position(),
					error_message
				);
				break;
			}
		};

		read_buffer.clear();
	}

	return Ok(writer.into_inner().into_inner());
}

fn
get_exif_filtered_event<'a>
(
	event: &'a BytesStart<'a>
)
-> Result<BytesStart<'a>, Box<dyn std::error::Error>>
{
	let mut new_event = BytesStart::new(
		std::str::from_utf8(event.name().0)?
	);

	new_event.extend_attributes(
		event.attributes()
			.filter_map(Result::ok)
			.filter(|attribute|
				{
					if let Ok(key) = std::str::from_utf8(
						attribute.key.as_ref()
					)
					{
						!key.starts_with("exif:")
					} else {
						true
					}
				}
			),
	);

	return Ok(new_event);
}
