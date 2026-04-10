// Copyright © 2024-2026 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use little_exif::iptc::IptcData;
use little_exif::metadata::Metadata;
use std::fs;

fn main()
{
    let src_path = std::path::Path::new("examples/test_image.jpg");
    let out_path = std::path::Path::new("examples/test_image_iptc_out.jpg");

    // --- Step 1: read source image ---
    let mut metadata = Metadata::new_from_path(src_path)
        .expect("Failed to read metadata from source image");

    if let Some(existing) = metadata.get_iptc()
    {
        println!("Existing IPTC fields: {}", existing.fields.len());
        for f in &existing.fields
        {
            println!(
                "  Record {} Dataset {}: {}",
                f.record, f.dataset,
                String::from_utf8_lossy(&f.data)
            );
        }
    }
    else
    {
        println!("No IPTC data found in source image.");
    }

    // --- Step 2: build new IPTC data ---
    let mut iptc = IptcData::new();

    // 2:05  Object Name (title)
    iptc.set_field(2, 5, b"Test Title".to_vec());

    // 2:25  Keywords (multi-value)
    iptc.add_field(2, 25, b"rust".to_vec());
    iptc.add_field(2, 25, b"exif".to_vec());
    iptc.add_field(2, 25, b"iptc".to_vec());

    // 2:80  By-line (author)
    iptc.set_field(2, 80, b"Test Author".to_vec());

    // 2:116 Copyright Notice
    iptc.set_field(2, 116, b"(c) 2026 Test".to_vec());

    metadata.set_iptc(iptc);

    // --- Step 3: write to output file ---
    fs::copy(src_path, out_path).expect("Failed to copy source image");
    metadata.write_to_file(out_path).expect("Failed to write metadata");
    println!("Written to {out_path:?}");

    // --- Step 4: read back and verify ---
    let readback = Metadata::new_from_path(out_path)
        .expect("Failed to read back output image");

    let iptc_back = readback.get_iptc().expect("No IPTC data found in output image");

    // Verify title
    let title_fields = iptc_back.get_fields(2, 5);
    assert!(!title_fields.is_empty(), "Title field missing");
    assert_eq!(title_fields[0].data, b"Test Title", "Title mismatch");

    // Verify keywords
    let keyword_fields = iptc_back.get_fields(2, 25);
    assert_eq!(keyword_fields.len(), 3, "Expected 3 keyword fields");
    let keywords: Vec<_> = keyword_fields.iter()
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .collect();
    assert!(keywords.contains(&"rust".to_string()),  "Missing keyword 'rust'");
    assert!(keywords.contains(&"exif".to_string()),  "Missing keyword 'exif'");
    assert!(keywords.contains(&"iptc".to_string()),  "Missing keyword 'iptc'");

    // Verify author
    let author_fields = iptc_back.get_fields(2, 80);
    assert!(!author_fields.is_empty(), "Author field missing");
    assert_eq!(author_fields[0].data, b"Test Author", "Author mismatch");

    println!("All IPTC round-trip checks passed.");
    println!("Fields in output:");
    for f in &iptc_back.fields
    {
        println!(
            "  Record {} Dataset {}: {}",
            f.record, f.dataset,
            String::from_utf8_lossy(&f.data)
        );
    }
}
