#[cfg(test)]
mod tests;

mod process;
mod utils;

use process::convert_dir;
use utils::{parse_scaling, Scaling};

use clap::Parser;
use std::fs;
use std::ops::Not;
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

const ELM_TYPES: &str = r"
import dict 

type alias Resolution = 
{
    width : Int
,   height : Int
}

type alias ImageSizing = 
{
    original : Resolution 
,   large : Resolution 
,   medium : Resolution 
,   small : Resolution
}

type alias SizeMap = Dict String ImageSizing

imageSizes : SizeMap 
imageSizes = 
";

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(short, long)]
    output: PathBuf,

    #[arg(short = 'y', long = "yes")]
    delete_dir: bool,

    #[arg(short, long)]
    js: Option<PathBuf>,

    #[arg(short, long)]
    elm: Option<PathBuf>,

    #[arg(short, long, default_value = "15 30 60", value_parser = parse_scaling)]
    scale: Scaling,
}

fn main() -> std::io::Result<()> {
    let Args {
        input,
        output,
        delete_dir,
        js,
        elm,
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

    if js.as_ref().is_some_and(|path| path.exists().not()) {
        fs::create_dir(js.clone().unwrap())?;
    } else if elm.as_ref().is_some_and(|path| path.exists().not()) {
        fs::create_dir(elm.clone().unwrap())?;
    }

    let size_map = convert_dir(input, &scale, output)?;

    let output_title = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(output.file_stem().and_then(|s| s.to_str()).unwrap());

    if js
        .as_ref()
        .is_some_and(|path| path.join("image-types.ts").exists().not())
    {
        fs::write(js.clone().unwrap().join("image-types.ts"), TS_TYPE)?;
    }

    if let Some(dir) = js {
        let object = format!(
            "{TS_SIZEMAP_DECLARATION} {output_title}: SizeMap = {};",
            serde_json::to_string_pretty(&size_map)?
        );
        let output_title = &format!("{output_title}.ts");
        fs::write(dir.join(output_title), object)
    } else if let Some(dir) = elm {
        let assoc_list = size_map.iter().fold(String::new(), |acc, (k, v)| {
            acc + &format!("(\"{k}\", {})", v.serialize_elm())
        });
        let dict =
            format!("module ImageSizes exposing (..)\n{ELM_TYPES} Dict.fromList [{assoc_list}]");
        fs::write(dir.join("ImageSizes.elm"), dict)
    } else {
        println!("No output dir was given, converted images, now exiting . . .");
        Ok(())
    }
}
