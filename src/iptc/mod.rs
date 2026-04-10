// Copyright © 2024-2026 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io;

/// A single IPTC-IIM field identified by a record number and dataset number.
///
/// Wire format per field: `[0x1C][record][dataset][length: u16 BE][data...]`
#[derive(Clone, Debug, PartialEq)]
pub struct
IptcField
{
    pub record:  u8,
    pub dataset: u8,
    pub data:    Vec<u8>,
}

/// Collection of IPTC-IIM fields read from or written to an image file.
///
/// # Common dataset numbers (record 2)
/// - 2:05  Object Name (title)
/// - 2:25  Keywords (multi-value)
/// - 2:80  By-line (author)
/// - 2:116 Copyright Notice
#[derive(Clone, Debug, PartialEq)]
pub struct
IptcData
{
    pub fields: Vec<IptcField>,
}

impl
IptcData
{
    pub fn
    new()
    -> Self
    {
        IptcData { fields: Vec::new() }
    }

    /// Returns all fields matching the given record and dataset numbers.
    pub fn
    get_fields
    (
        &self,
        record:  u8,
        dataset: u8,
    )
    -> Vec<&IptcField>
    {
        self.fields.iter()
            .filter(|f| f.record == record && f.dataset == dataset)
            .collect()
    }

    /// Replaces any existing field(s) with the given record/dataset with a
    /// single new entry. For multi-value datasets use `add_field` instead.
    pub fn
    set_field
    (
        &mut self,
        record:  u8,
        dataset: u8,
        data:    Vec<u8>,
    )
    {
        self.remove_fields(record, dataset);
        self.fields.push(IptcField { record, dataset, data });
    }

    /// Appends a new field without removing existing ones with the same
    /// record/dataset. Use this for multi-value datasets such as Keywords (2:25).
    pub fn
    add_field
    (
        &mut self,
        record:  u8,
        dataset: u8,
        data:    Vec<u8>,
    )
    {
        self.fields.push(IptcField { record, dataset, data });
    }

    /// Removes all fields with the given record and dataset numbers.
    pub fn
    remove_fields
    (
        &mut self,
        record:  u8,
        dataset: u8,
    )
    {
        self.fields.retain(|f| !(f.record == record && f.dataset == dataset));
    }

    /// Encodes all fields as raw IPTC-IIM bytes.
    /// Each field is encoded as: `[0x1C][record][dataset][len_hi][len_lo][data...]`
    pub fn
    encode
    (
        &self
    )
    -> Vec<u8>
    {
        let mut out = Vec::new();
        for field in &self.fields
        {
            let len = field.data.len() as u16;
            out.push(0x1C);
            out.push(field.record);
            out.push(field.dataset);
            out.push((len >> 8) as u8);
            out.push(len as u8);
            out.extend_from_slice(&field.data);
        }
        out
    }

    /// Parses raw IPTC-IIM bytes into an `IptcData` struct.
    /// Returns `InvalidData` if a tag marker byte other than `0x1C` is encountered.
    pub fn
    decode
    (
        data: &[u8]
    )
    -> Result<Self, io::Error>
    {
        let mut fields = Vec::new();
        let mut pos    = 0usize;

        while pos < data.len()
        {
            // Each record must start with the tag marker 0x1C
            if data[pos] != 0x1C
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "IptcData::decode: expected tag marker 0x1C at position {pos}, got 0x{:02X}",
                        data[pos]
                    ),
                ));
            }
            pos += 1;

            // Need at least 4 more bytes: record, dataset, len_hi, len_lo
            if pos + 4 > data.len()
            {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "IptcData::decode: truncated field header",
                ));
            }

            let record  = data[pos];     pos += 1;
            let dataset = data[pos];     pos += 1;
            let len     = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
            pos += 2;

            if pos + len > data.len()
            {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "IptcData::decode: field data extends beyond buffer",
                ));
            }

            let field_data = data[pos..pos + len].to_vec();
            pos += len;

            fields.push(IptcField { record, dataset, data: field_data });
        }

        Ok(IptcData { fields })
    }
}
