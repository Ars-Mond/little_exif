// Copyright © 2024-2026 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use little_exif::endian::Endian;
use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;
use little_exif::xmp::XmpData;
use std::fs;

fn main()
{
    let src_path = std::path::Path::new("examples/test_image_2.jpg");
    let out_path = std::path::Path::new("examples/test_image_xmp_out_2.jpg");

    // --- Step 1: read existing metadata ---
    let metadata = Metadata::new_from_path(src_path)
        .expect("Failed to read metadata from source image");

    if let Some(existing) = metadata.get_xmp()
    {
        println!("Existing XMP ({} bytes):", existing.packet.len());
        println!("{}", String::from_utf8_lossy(existing.as_bytes()));
        let exif_tags = existing.get_exif_tags(&Endian::Little);
        println!("  EXIF tags in XMP: {}", exif_tags.len());
        for tag in &exif_tags
        {
            println!("    {:?}", tag);
        }
    }
    else
    {
        println!("No XMP data in source image.");
    }

    // --- Step 2: build new XMP with EXIF tags ---
    let mut xmp = XmpData::new();
    xmp.set_exif_tags(
        &[
            ExifTag::Make("TestCamera".to_string()),
            ExifTag::Model("TestModel XR-9".to_string()),
            ExifTag::Software("little_exif".to_string()),
            ExifTag::Artist("Test Artist".to_string()),
            ExifTag::Orientation(vec![1u16]),
            ExifTag::ISO(vec![400u16]),
        ],
        &Endian::Little,
    )
    .expect("Failed to set EXIF tags in XMP");

    let mut metadata_out = Metadata::new_from_path(src_path)
        .expect("Failed to read metadata for output");
    metadata_out.set_xmp(xmp);

    // --- Step 3: write to output file ---
    fs::copy(src_path, out_path).expect("Failed to copy source image");
    metadata_out
        .write_to_file(out_path)
        .expect("Failed to write metadata");
    println!("Written to {out_path:?}");

    // --- Step 4: read back and verify ---
    let readback = Metadata::new_from_path(out_path)
        .expect("Failed to read back output image");

    let xmp_back = readback
        .get_xmp()
        .expect("No XMP data found in output image");

    println!("Read-back XMP ({} bytes)", xmp_back.packet.len());

    let tags = xmp_back.get_exif_tags(&Endian::Little);

    let make = tags.iter().find(|t| matches!(t, ExifTag::Make(_)));
    assert!(
        matches!(make, Some(ExifTag::Make(s)) if s == "TestCamera"),
        "Make mismatch: {:?}",
        make
    );

    let model = tags.iter().find(|t| matches!(t, ExifTag::Model(_)));
    assert!(
        matches!(model, Some(ExifTag::Model(s)) if s == "TestModel XR-9"),
        "Model mismatch: {:?}",
        model
    );

    let software = tags.iter().find(|t| matches!(t, ExifTag::Software(_)));
    assert!(
        matches!(software, Some(ExifTag::Software(s)) if s == "little_exif"),
        "Software mismatch: {:?}",
        software
    );

    let iso = tags.iter().find(|t| matches!(t, ExifTag::ISO(_)));
    assert!(
        matches!(iso, Some(ExifTag::ISO(v)) if v == &vec![400u16]),
        "ISO mismatch: {:?}",
        iso
    );

    println!("All XMP round-trip checks passed.");
    println!("Tags in output XMP:");
    for tag in &tags
    {
        println!("  {:?}", tag);
    }
}
