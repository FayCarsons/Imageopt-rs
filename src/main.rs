#[cfg(test)]
mod tests;

mod process;
mod utils;

use process::convert_dir;
use utils::{parse_scaling, Scaling};

use clap::Parser;
use serde_json::to_string_pretty;
use std::fs;
use std::path::PathBuf;

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
       
export const ";

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(short, long)]
    output: PathBuf,

    #[arg(short = 'y', long = "yes")]
    delete_dir: bool,

    #[arg(short, long)]
    js: PathBuf,

    #[arg(short, long, default_value = "15 30 60", value_parser = parse_scaling)]
    scale: Scaling,
}

fn main() -> std::io::Result<()> {
    let Args {
        input,
        output,
        delete_dir,
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
        if delete_dir {
            fs::remove_dir_all(output)?;
            fs::create_dir(output)?;
        } else {
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

    let object = format!(
        "{TS_SIZEMAP_DECLARATION} {output_title}: SizeMap = {};",
        to_string_pretty(&size_map)?
    );
    let output_title = &format!("{output_title}.ts");
    fs::write(js.join(output_title), object)
}
