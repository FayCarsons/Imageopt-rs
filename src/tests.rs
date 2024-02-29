use clap::builder::OsStr;

use crate::{
    process::is_image,
    utils::{parse_scaling, Scale},
};

#[test]
fn test_parse_scaling() {
    let input = "30, 50, 99";
    let res = parse_scaling(input);
    assert_eq!(
        res.unwrap(),
        [Scale::Small(30), Scale::Medium(50), Scale::Large(99)]
    )
}

#[test]
fn test_image_extension() {
    use std::ffi::OsStr;
    let not_images = vec![
        OsStr::new("DS_Store"),
        OsStr::new("txt"),
        OsStr::new("pdf"),
        OsStr::new("json"),
    ];
    let mut res = not_images.into_iter().map(is_image);
    assert!(res.all(|b| !b));

    let images = vec![
        OsStr::new("png"),
        OsStr::new("tiff"),
        OsStr::new("jpg"),
        OsStr::new("jpeg"),
        OsStr::new("raw"),
    ];
    let mut res = images.into_iter().map(is_image);
    assert!(res.all(|b| b))
}
