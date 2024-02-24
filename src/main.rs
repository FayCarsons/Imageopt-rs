use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::collections::HashMap;

use std::fs;
use std::path::PathBuf;

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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct Resolution {
    width: u32,
    height: u32,
}

impl Resolution {
    fn scale(&self, factor: u16) -> Self {
        let factor = factor as f32 / 100.;
        Self {
            width: (self.width as f32 * factor) as u32,
            height: (self.height as f32 * factor) as u32,
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
    fn inner(&self) -> u16 {
        match self {
            Self::Large(n) => *n,
            Self::Medium(n) => *n,
            Self::Small(n) => *n,
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

    #[arg(long)]
    js: PathBuf,

    #[arg(short, long, value_parser = parse_scaling)]
    scale: Option<Scaling>,
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

fn main() -> std::io::Result<()> {
    let Args {
        input,
        output,
        js,
        scale,
    } = Args::parse();

    let scale = scale.unwrap_or([Scale::Small(15), Scale::Medium(30), Scale::Large(60)]);

    let mut images = HashMap::<Title, Image>::new();

    if !input.exists() || !input.is_dir() {
        panic!(
            "Input directory {} does not exist!",
            input.to_str().unwrap()
        );
    }

    if !output.exists() {
        fs::create_dir(output.clone()).unwrap();
    } else {
        let mut buffer = String::new();
        println!("Delete directory {:?}? [y/n]", output.clone());
        std::io::stdin()
            .read_line(&mut buffer)
            .expect("User input failed\n");

        if buffer.trim() == "y" {
            println!("Removing directory . . .");
            fs::remove_dir_all(output.clone())?;
            fs::create_dir(output.clone())?;
        }
    }

    if !js.exists() {
        fs::create_dir(js.clone())?;
    }

    for file in input.read_dir().unwrap() {
        let path = file?.path();
        let path_str = path.to_str().unwrap();
        println!("Path: {:?}", path);

        let process = std::process::Command::new("magick")
            .args([
                "identify",
                "-ping",
                "-format",
                r#"{ "width": %w, "height": %h }"#,
                path_str,
            ])
            .output()?;

        let original = if process.status.success() {
            let stdout = String::from_utf8_lossy(&process.stdout);
            serde_json::from_str::<Resolution>(&stdout)
        } else {
            panic!("Image: {path_str} cannot be read");
        }
        .expect("RESOLUTION DESERIALIZATION FAILED");

        let scaled_images = scale
            .into_iter()
            .map(|scl| {
                let (tag, factor) = match scl {
                    Scale::Large(n) => ("_large", n),
                    Scale::Medium(n) => ("_medium", n),
                    Scale::Small(n) => ("_small", n),
                };

                let output_filename = path.file_stem().and_then(|s| s.to_str()).unwrap();
                let output_filename = format!("{output_filename}{tag}.avif");
                let output_path = output.join(output_filename.clone());
                let output_path = output_path.to_str().expect("Cannot convert `output_path` to str");

                println!("Path Str: {path_str}\n Output filename: {output_filename}\n Output path: {output_path}");

                std::process::Command::new("magick")
                    .args([
                        "convert",
                        "-resize",
                        &format!("{factor}%"),
                        path_str,
                        output_path,
                    ])
                    .output()
            })
            .collect::<std::io::Result<Vec<std::process::Output>>>()?;

        if let Some(e) = scaled_images.into_iter().find(|p| !p.status.success()) {
            eprintln!("Error creating scaled images: {e:?}");
        };

        let sizes = original.to_image(&scale);

        images.insert(
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap()
                .to_string(),
            sizes,
        );
    }

    let output_title = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(output.file_stem().and_then(|s| s.to_str()).unwrap());

    if !js.clone().join("image-types.ts").exists() {
        fs::write(js.join("image-types.ts"), TS_TYPE)?;
    }

    let object = format!(
        "const {output_title} = {};",
        to_string_pretty(&images).unwrap()
    );
    let output_title = &format!("{output_title}.ts");
    fs::write(js.join(output_title), object)
}
