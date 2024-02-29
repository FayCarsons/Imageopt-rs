# Image optimization CLI tool
Batch image conversion (only to AVIF at the moment, plans to add WEBP) and scaling, 
with image resolutions and utility types written to .ts files for CLS mitigation. 

# Basic Use
Options are as follows:
`-i / --input`: image input directory, <br>
`-o / --output`: image output directory, <br>
`-j / --js`: Typescript output directory, <br>
`-s / --scale`: (optional) scaling for small, medium, large versions of images. <br>
Must be three comma or whitespace separated integers in ascending order. <br>
`-y / --yes`: automatically deletes output directory and contents if already exists. <br>

Can be run with `cargo`, <br> 
for example: `cargo run --release -- -i test -o test-out -j js-test -s 15 30 60 --yes`
Alternatively, build with `cargo build --release` and embed the executable in `target/release` into your project.

# TS Output
The typescript files written to your output dir will contain a Record of all converted filenames mapped to 
original and resized resolutions as well as `image-types.ts` which contains declarations for that type and <br>
the types that it contains. <br>

For example, converting a directory "home" will also yield `home.ts` which contains something like this: <br>
```typescript
export const home: SizeMap = {
  "title": {
    "original": {
      "width": 1200,
      "height": 1200,
    },
    "large": {
      "width": 900,
      "height": 900
    },
    "medium": {
      "width": 600,
      "height": 600
    },
    "small": {
      "width": 300,
      "height": 300
    }
  }
}
```
