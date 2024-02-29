use super::utils::{Image, Resolution, Scale, Scaling, Title};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::DirEntry;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

pub fn is_image(s: &OsStr) -> bool {
    matches!(
        &s.as_bytes().to_ascii_lowercase()[..],
        b"png" | b"jpg" | b"jpeg" | b"tiff" | b"raw"
    )
}

fn convert_image(
    file: &DirEntry,
    scale: &Scaling,
    output: &Path,
) -> std::io::Result<(String, Image)> {
    let path = file.path();
    let path_str = path.to_str().expect("Error parsing file path: {path:?}");

    // Get resolution
    let get_resolution = std::process::Command::new("magick")
        .args([
            "identify",
            "-ping",
            "-format",
            r#"{ "width": %w, "height": %h }"#,
            path_str,
        ])
        .output()?;

    let original = if get_resolution.status.success() {
        let stdout = String::from_utf8_lossy(&get_resolution.stdout);
        serde_json::from_str::<Resolution>(&stdout)
    } else {
        panic!("Image {path_str} cannot be read");
    }?;

    // Convert original to AVIF
    if path
        .extension()
        .is_some_and(|ext| ext.as_bytes().to_ascii_lowercase() != b"avif")
        && path.file_stem().is_some()
    {
        let output_path = path
            .file_stem()
            .and_then(|os| os.to_str())
            .map(|s| s.to_string() + ".avif")
            .unwrap();
        let convert_avif = std::process::Command::new("magick")
            .args([
                "convert",
                path_str,
                &output.join(output_path).to_string_lossy(),
            ])
            .output()?;

        assert!(convert_avif.status.success());
    }

    let scale_processes = scale
        .iter()
        .map(|scl| {
            let (tag, factor) = match scl {
                Scale::Large(n) => ("_large", n),
                Scale::Medium(n) => ("_medium", n),
                Scale::Small(n) => ("_small", n),
            };

            let output_filename = path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string() + tag + ".avif")
                .unwrap();
            let output_path = output.join(output_filename.clone());
            let output_path = output_path
                .to_str()
                .unwrap_or_else(|| panic!("Cannot convert dirname {output:?} to string"));

            std::process::Command::new("magick")
                .args([
                    "convert",
                    "-resize",
                    &format!("{factor}%"),
                    path_str,
                    output_path,
                ])
                .spawn()
        })
        .collect::<std::io::Result<Vec<std::process::Child>>>()?;

    path.file_stem()
        .iter()
        .for_each(|title| println!("Reszing image {title:?} and converting to AVIF"));

    let scaled_results = scale_processes
        .into_iter()
        .map(std::process::Child::wait_with_output)
        .collect::<Result<Vec<std::process::Output>, _>>()?;

    if let Some(e) = scaled_results.into_iter().find(|p| !p.status.success()) {
        eprintln!("Error creating scaled images: {e:?}");
    };

    let sizes = original.to_image(scale);

    Ok((
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap()
            .to_string(),
        sizes,
    ))
}

pub fn convert_dir(
    dir: &Path,
    scale: &Scaling,
    output: &Path,
) -> std::io::Result<HashMap<Title, Image>> {
    dir.read_dir()?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().map_or(false, is_image))
        .map(|ref entry| convert_image(entry, scale, output))
        .collect::<std::io::Result<HashMap<Title, Image>>>()
}
