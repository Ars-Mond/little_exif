// Test XPTitle, XPKeywords, XPSubject and UnknownUTF16 tags

use std::fs;
use std::path::Path;

extern crate little_exif;
use little_exif::exif_tag::ExifTag;
use little_exif::exif_tag_format::Utf16String;
use little_exif::ifd::ExifTagGroup;
use little_exif::metadata::Metadata;

fn main() -> Result<(), std::io::Error>
{
    let src = Path::new("examples/test_image.jpg");

    // -----------------------------------------------------------------------
    // 1. Read existing XP tags from the supplied image
    // -----------------------------------------------------------------------
    println!("=== Reading XP tags from examples/test_image.jpg ===");
    let metadata = Metadata::new_from_path(src)?;

    let xp_tags: Vec<&ExifTag> = (&metadata)
        .into_iter()
        .filter(|t| matches!(
            t,
            ExifTag::XPTitle(_) | ExifTag::XPKeywords(_) | ExifTag::XPSubject(_)
        ))
        .collect();

    if xp_tags.is_empty()
    {
        println!("  (no XPTitle / XPKeywords / XPSubject found in the image)");
    }
    else
    {
        for tag in &xp_tags
        {
            match tag
            {
                ExifTag::XPTitle(v)    => println!("  XPTitle    : {:?}", v.0),
                ExifTag::XPKeywords(v) => println!("  XPKeywords : {:?}", v.0),
                ExifTag::XPSubject(v)  => println!("  XPSubject  : {:?}", v.0),
                _ => {}
            }
        }
    }

    // -----------------------------------------------------------------------
    // 2. Write round-trip: copy image, set new XP tags, re-read
    // -----------------------------------------------------------------------
    let dst = Path::new("examples/test_image_xp_out.jpg");
    fs::copy(src, dst)?;

    let mut meta_write = Metadata::new_from_path(dst)?;

    meta_write.set_tag(ExifTag::XPTitle(   Utf16String::from("Sample title for testing")));
    meta_write.set_tag(ExifTag::XPKeywords(Utf16String::from("rust; exif; utf-16")));
    meta_write.set_tag(ExifTag::XPSubject( Utf16String::from("Subject: library test")));

    // UnknownUTF16 for XPAuthor (0x9c9d) — not yet a named tag
    meta_write.set_tag(ExifTag::UnknownUTF16(
        Utf16String::from("Test Author"),
        0x9c9d,
        ExifTagGroup::GENERIC,
    ));

    meta_write.write_to_file(dst)?;

    println!("\n=== Re-reading examples/test_image_xp_out.jpg ===");
    let meta_read = Metadata::new_from_path(dst)?;
    for tag in &meta_read
    {
        match tag
        {
            ExifTag::XPTitle(v)    => println!("  XPTitle    : {:?}", v.0),
            ExifTag::XPKeywords(v) => println!("  XPKeywords : {:?}", v.0),
            ExifTag::XPSubject(v)  => println!("  XPSubject  : {:?}", v.0),
            ExifTag::UnknownUTF16(v, hex, grp) =>
                println!("  UnknownUTF16 0x{:04x} ({:?}): {:?}", hex, grp, v.0),
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // 3. Assert round-trip correctness
    // -----------------------------------------------------------------------
    let title = meta_read.get_tag(&ExifTag::XPTitle(Utf16String::new())).next();
    match title
    {
        Some(ExifTag::XPTitle(v)) => {
            assert_eq!(v.0, "Sample title for testing", "XPTitle mismatch!");
            println!("\n[OK] XPTitle round-trip correct");
        }
        _ => println!("\n[FAIL] XPTitle not found after write"),
    }

    let keywords = meta_read.get_tag(&ExifTag::XPKeywords(Utf16String::new())).next();
    match keywords
    {
        Some(ExifTag::XPKeywords(v)) => {
            assert_eq!(v.0, "rust; exif; utf-16", "XPKeywords mismatch!");
            println!("[OK] XPKeywords round-trip correct");
        }
        _ => println!("[FAIL] XPKeywords not found after write"),
    }

    let subject = meta_read.get_tag(&ExifTag::XPSubject(Utf16String::new())).next();
    match subject
    {
        Some(ExifTag::XPSubject(v)) => {
            assert_eq!(v.0, "Subject: library test", "XPSubject mismatch!");
            println!("[OK] XPSubject round-trip correct");
        }
        _ => println!("[FAIL] XPSubject not found after write"),
    }

    println!("\nOutput written to: examples/test_image_xp_out.jpg");
    Ok(())
}
