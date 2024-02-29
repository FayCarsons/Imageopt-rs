use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self, DirEntry};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

type Title = String;

const TS_TYPE: &str = r"
export type Resolution = {
    width: number; 
    height: number;
}

export type ImageSizing = {
    original: Resolution;
    large: Resolution; 
    medium: Resolution;
    small: Resolution;
}

export type SizeMap = Record<string, ImageSizing>;";

const TS_SIZEMAP_DECLARATION: &str = r"
import type { SizeMap } from './image-types.ts'
       
export const {output_title}: SizeMap = ";

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct Resolution {
    width: u32,
    height: u32,
}

impl Resolution {
    fn scale(self, factor: u16) -> Self {
        let factor = f64::from(factor) / 100.;
        Self {
            width: (f64::from(self.width) * factor) as u32,
            height: (f64::from(self.height) * factor) as u32,
        }
    }

    fn to_image(self, [small, medium, large]: &Scaling) -> Image {
        Image {
            original: self,
            large: self.scale(large.inner()),
            medium: self.scale(medium.inner()),
            small: self.scale(small.inner()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct Image {
    original: Resolution,
    large: Resolution,
    medium: Resolution,
    small: Resolution,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum Scale {
    Large(u16),
    Medium(u16),
    Small(u16),
}

impl Scale {
    fn inner(self) -> u16 {
        match self {
            Self::Large(n) | Self::Medium(n) | Self::Small(n) => n,
        }
    }
}

type Scaling = [Scale; 3];

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(short, long)]
    output: PathBuf,

    #[arg(short, long)]
    js: PathBuf,

    #[arg(short, long, default_value = "15 30 60", value_parser = parse_scaling)]
    scale: Scaling,
}

fn parse_scaling(s: &str) -> Result<Scaling, String> {
    let vals = s
        .split(&[',', ' '][..])
        .filter_map(|s| str::parse::<u16>(s).ok())
        .collect::<Vec<u16>>();

    let [small, medium, large] = &vals[..] else {
        return Err("Scaling arg should be three unsigned integers separated by space or commas, I.E. --scale 10, 50, 75".to_string());
    };

    Ok([
        Scale::Small(*small),
        Scale::Medium(*medium),
        Scale::Large(*large),
    ])
}

fn is_image(s: &OsStr) -> bool {
    matches!(s.as_bytes(), b"png" | b"jpg" | b"jpeg" | b"tiff" | b"raw")
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
            .args(["convert", path_str, &output_path])
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

            let output_filename = path.file_stem().and_then(|s| s.to_str()).unwrap();
            let output_filename = format!("{output_filename}{tag}.avif");
            let output_path = output.join(output_filename.clone());
            let output_path = output_path
                .to_str()
                .expect("Cannot convert `output_path` to str");

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

fn convert_dir(
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

fn main() -> std::io::Result<()> {
    let Args {
        input,
        output,
        js,
        scale,
    } = Args::parse();

    let (input, output, js) = (&input, &output, &js);

    assert!(
        (input.exists() && input.is_dir()),
        "input directory {} does not exist!",
        input.to_str().unwrap()
    );

    if output.exists() {
        let mut buffer = String::new();
        println!("Delete directory {:?}? [y/n]", output.clone());
        std::io::stdin()
            .read_line(&mut buffer)
            .expect("User input failed\n");

        if buffer.trim() == "y" {
            println!("Removing directory . . .");
            fs::remove_dir_all(output)?;
            fs::create_dir(output)?;
        }
    } else {
        fs::create_dir(output)?;
    }

    if !js.exists() {
        fs::create_dir(js)?;
    }

    let size_map = convert_dir(input, &scale, output)?;

    let output_title = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(output.file_stem().and_then(|s| s.to_str()).unwrap());

    if !js.join("image-types.ts").exists() {
        fs::write(js.join("image-types.ts"), TS_TYPE)?;
    }

    let object = format!("{TS_SIZEMAP_DECLARATION}{};", to_string_pretty(&size_map)?);
    let output_title = &format!("{output_title}.ts");
    fs::write(js.join(output_title), object)
}
