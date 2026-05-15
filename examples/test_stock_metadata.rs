// Copyright © 2024-2026 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details
//
// Reads "stock photo" metadata (title, description, keywords, author,
// copyright, etc.) from images. Stocks usually populate these via IPTC-IIM
// and/or XMP (dc:, photoshop:, Iptc4xmpCore:), with EXIF carrying the
// camera/shooting fields.
//
// Targets examples/test.png and examples/test.webp.

use std::path::Path;

use little_exif::exif_tag::ExifTag;
use little_exif::iptc::IptcData;
use little_exif::metadata::Metadata;
use little_exif::xmp::XmpData;

fn main()
{
    let png_path  = Path::new("examples/test.png");
    let webp_path = Path::new("examples/test.webp");

    inspect(png_path);
    println!();
    inspect(webp_path);
}

fn inspect(path: &Path)
{
    println!("==============================================================");
    println!("File: {}", path.display());
    println!("==============================================================");

    let metadata = match Metadata::new_from_path(path)
    {
        Ok(m)  => m,
        Err(e) =>
        {
            println!("  [ERROR] Failed to read metadata: {e}");
            return;
        }
    };

    print_exif_section(&metadata);
    print_iptc_section(metadata.get_iptc());
    print_xmp_section(metadata.get_xmp());
}

// -----------------------------------------------------------------------
// EXIF — picks out the stock-relevant string and shooting tags
// -----------------------------------------------------------------------
fn print_exif_section(metadata: &Metadata)
{
    println!("\n--- EXIF ---");

    let mut found_any = false;
    for tag in metadata
    {
        match tag
        {
            ExifTag::ImageDescription(v)  => { found_any = true; println!("  ImageDescription : {v}"); }
            ExifTag::Artist(v)            => { found_any = true; println!("  Artist           : {v}"); }
            ExifTag::Copyright(v)         => { found_any = true; println!("  Copyright        : {v}"); }
            ExifTag::Software(v)          => { found_any = true; println!("  Software         : {v}"); }
            ExifTag::Make(v)              => { found_any = true; println!("  Make             : {v}"); }
            ExifTag::Model(v)             => { found_any = true; println!("  Model            : {v}"); }
            ExifTag::DateTimeOriginal(v)  => { found_any = true; println!("  DateTimeOriginal : {v}"); }
            ExifTag::CreateDate(v)        => { found_any = true; println!("  CreateDate       : {v}"); }
            ExifTag::ModifyDate(v)        => { found_any = true; println!("  ModifyDate       : {v}"); }
            ExifTag::XPTitle(v)           => { found_any = true; println!("  XPTitle          : {}", v.0); }
            ExifTag::XPSubject(v)         => { found_any = true; println!("  XPSubject        : {}", v.0); }
            ExifTag::XPKeywords(v)        => { found_any = true; println!("  XPKeywords       : {}", v.0); }
            ExifTag::XPComment(v)         => { found_any = true; println!("  XPComment        : {}", v.0); }
            ExifTag::XPAuthor(v)          => { found_any = true; println!("  XPAuthor         : {}", v.0); }
            ExifTag::Orientation(v)       => { found_any = true; println!("  Orientation      : {v:?}"); }
            ExifTag::ISO(v)               => { found_any = true; println!("  ISO              : {v:?}"); }
            ExifTag::FNumber(v)           => { found_any = true; println!("  FNumber          : {v:?}"); }
            ExifTag::ExposureTime(v)      => { found_any = true; println!("  ExposureTime     : {v:?}"); }
            ExifTag::FocalLength(v)       => { found_any = true; println!("  FocalLength      : {v:?}"); }
            _ => {}
        }
    }

    if !found_any
    {
        println!("  (no stock-relevant EXIF tags found)");
    }
}

// -----------------------------------------------------------------------
// IPTC-IIM — the classic stock fields live in record 2
// -----------------------------------------------------------------------
fn print_iptc_section(iptc: Option<&IptcData>)
{
    println!("\n--- IPTC-IIM ---");

    let Some(iptc) = iptc else
    {
        println!("  (no IPTC data)");
        return;
    };

    if iptc.fields.is_empty()
    {
        println!("  (empty IPTC block)");
        return;
    }

    print_iptc_field (iptc, 2,   5, "Title (Object Name)");
    print_iptc_field (iptc, 2, 120, "Description (Caption)");
    print_iptc_fields(iptc, 2,  25, "Keywords");
    print_iptc_field (iptc, 2,  80, "By-line (Author)");
    print_iptc_field (iptc, 2,  85, "By-line Title");
    print_iptc_field (iptc, 2, 116, "Copyright Notice");
    print_iptc_field (iptc, 2, 105, "Headline");
    print_iptc_field (iptc, 2, 110, "Credit");
    print_iptc_field (iptc, 2, 115, "Source");
    print_iptc_field (iptc, 2,  90, "City");
    print_iptc_field (iptc, 2,  95, "Province/State");
    print_iptc_field (iptc, 2, 101, "Country");
    print_iptc_field (iptc, 2,  55, "Date Created");

    // Anything not covered above
    let well_known = [5u8, 120, 25, 80, 85, 116, 105, 110, 115, 90, 95, 101, 55];
    let mut others: Vec<&little_exif::iptc::IptcField> = iptc.fields.iter()
        .filter(|f| f.record != 2 || !well_known.contains(&f.dataset))
        .collect();
    others.sort_by_key(|f| (f.record, f.dataset));

    if !others.is_empty()
    {
        println!("\n  Other IPTC fields:");
        for f in others
        {
            println!(
                "    {}:{:<3} = {}",
                f.record, f.dataset,
                String::from_utf8_lossy(&f.data)
            );
        }
    }
}

fn print_iptc_field(iptc: &IptcData, record: u8, dataset: u8, label: &str)
{
    let fields = iptc.get_fields(record, dataset);
    if let Some(f) = fields.first()
    {
        println!("  {:<24}: {}", label, String::from_utf8_lossy(&f.data));
    }
}

fn print_iptc_fields(iptc: &IptcData, record: u8, dataset: u8, label: &str)
{
    let fields = iptc.get_fields(record, dataset);
    if fields.is_empty() { return; }

    let values: Vec<String> = fields.iter()
        .map(|f| String::from_utf8_lossy(&f.data).into_owned())
        .collect();
    println!("  {:<24}: [{}] {}", label, values.len(), values.join(", "));
}

// -----------------------------------------------------------------------
// XMP — print the raw packet length plus extracted dc:/photoshop: fields
// -----------------------------------------------------------------------
fn print_xmp_section(xmp: Option<&XmpData>)
{
    println!("\n--- XMP ---");

    let Some(xmp) = xmp else
    {
        println!("  (no XMP packet)");
        return;
    };

    let packet = String::from_utf8_lossy(xmp.as_bytes());
    println!("  Packet size: {} bytes", xmp.packet.len());

    extract_xmp_simple (&packet, "dc:title",        "Title");
    extract_xmp_simple (&packet, "dc:description",  "Description");
    extract_xmp_bag    (&packet, "dc:subject",      "Keywords (dc:subject)");
    extract_xmp_seq    (&packet, "dc:creator",      "Creator (Author)");
    extract_xmp_simple (&packet, "dc:rights",       "Rights (Copyright)");
    extract_xmp_simple (&packet, "photoshop:Headline",         "Headline");
    extract_xmp_simple (&packet, "photoshop:Credit",           "Credit");
    extract_xmp_simple (&packet, "photoshop:Source",           "Source");
    extract_xmp_simple (&packet, "photoshop:City",             "City");
    extract_xmp_simple (&packet, "photoshop:Country",          "Country");
    extract_xmp_simple (&packet, "photoshop:DateCreated",      "Date Created");
    extract_xmp_simple (&packet, "xmpRights:Marked",           "Rights Marked");
    extract_xmp_simple (&packet, "xmpRights:WebStatement",     "Rights WebStatement");
    extract_xmp_simple (&packet, "Iptc4xmpCore:Location",      "Location");
    extract_xmp_simple (&packet, "Iptc4xmpCore:CountryCode",   "Country Code");
}

/// Reads a simple property like `<dc:title>...</dc:title>` or
/// `<dc:title><rdf:Alt><rdf:li xml:lang="x-default">value</rdf:li></rdf:Alt></dc:title>`.
/// Also handles attribute form: `dc:title="value"`.
fn extract_xmp_simple(packet: &str, name: &str, label: &str)
{
    if let Some(text) = read_alt_or_text(packet, name)
    {
        println!("  {:<28}: {}", label, text.trim());
        return;
    }

    if let Some(value) = read_attribute(packet, name)
    {
        println!("  {:<28}: {}", label, value.trim());
    }
}

/// Reads a `<dc:subject><rdf:Bag><rdf:li>...</rdf:li>...</rdf:Bag></dc:subject>` list.
fn extract_xmp_bag(packet: &str, name: &str, label: &str)
{
    let items = read_list(packet, name, "Bag");
    if !items.is_empty()
    {
        println!("  {:<28}: [{}] {}", label, items.len(), items.join(", "));
    }
}

/// Reads a `<dc:creator><rdf:Seq><rdf:li>...</rdf:li>...</rdf:Seq></dc:creator>` list.
fn extract_xmp_seq(packet: &str, name: &str, label: &str)
{
    let items = read_list(packet, name, "Seq");
    if !items.is_empty()
    {
        println!("  {:<28}: [{}] {}", label, items.len(), items.join(", "));
    }
}

// -----------------------------------------------------------------------
// Tiny XMP scanners — no XML parser, just substring matching. The XMP
// produced by stocks/Adobe is regular enough that this works in practice
// for these inspection helpers.
// -----------------------------------------------------------------------

fn read_alt_or_text(packet: &str, name: &str) -> Option<String>
{
    let open       = format!("<{name}");
    let close      = format!("</{name}>");
    let start      = packet.find(&open)?;
    let after_open = packet[start..].find('>')? + start + 1;
    let end        = packet[after_open..].find(&close)? + after_open;
    let inner      = &packet[after_open..end];

    // Try <rdf:Alt><rdf:li ...>value</rdf:li></rdf:Alt>
    if let Some(li_start) = inner.find("<rdf:li")
    {
        let after_li = inner[li_start..].find('>')? + li_start + 1;
        let li_end   = inner[after_li..].find("</rdf:li>")? + after_li;
        return Some(xml_unescape(inner[after_li..li_end].trim()));
    }

    let trimmed = inner.trim();
    if trimmed.is_empty() { None } else { Some(xml_unescape(trimmed)) }
}

fn read_list(packet: &str, name: &str, container: &str) -> Vec<String>
{
    let open  = format!("<{name}");
    let close = format!("</{name}>");
    let Some(start) = packet.find(&open) else { return Vec::new(); };

    let Some(after_open_rel) = packet[start..].find('>') else { return Vec::new(); };
    let after_open           = start + after_open_rel + 1;

    let Some(end_rel) = packet[after_open..].find(&close) else { return Vec::new(); };
    let end           = after_open + end_rel;
    let inner         = &packet[after_open..end];

    let container_open  = format!("<rdf:{container}");
    let container_close = format!("</rdf:{container}>");
    let Some(c_start) = inner.find(&container_open) else { return Vec::new(); };
    let Some(c_after_rel) = inner[c_start..].find('>') else { return Vec::new(); };
    let c_after = c_start + c_after_rel + 1;
    let Some(c_end_rel) = inner[c_after..].find(&container_close) else { return Vec::new(); };
    let body = &inner[c_after .. c_after + c_end_rel];

    let mut items = Vec::new();
    let mut cursor = 0usize;
    while let Some(li_rel) = body[cursor..].find("<rdf:li")
    {
        let li_start    = cursor + li_rel;
        let Some(after_rel) = body[li_start..].find('>') else { break; };
        let after_open_li = li_start + after_rel + 1;
        let Some(li_end_rel) = body[after_open_li..].find("</rdf:li>") else { break; };
        let li_end = after_open_li + li_end_rel;

        items.push(xml_unescape(body[after_open_li..li_end].trim()));
        cursor = li_end + "</rdf:li>".len();
    }
    items
}

fn read_attribute(packet: &str, name: &str) -> Option<String>
{
    let needle = format!("{name}=\"");
    let start  = packet.find(&needle)? + needle.len();
    let end    = packet[start..].find('"')? + start;
    Some(xml_unescape(&packet[start..end]))
}

fn xml_unescape(s: &str) -> String
{
    s.replace("&lt;",   "<")
     .replace("&gt;",   ">")
     .replace("&quot;", "\"")
     .replace("&apos;", "'")
     .replace("&amp;",  "&")
}
