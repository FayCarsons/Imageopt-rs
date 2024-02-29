use crate::{
    process::is_image,
    utils::{parse_scaling, Image, Resolution, Scale},
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

#[test]
fn test_scale_res() {
    let res = Resolution {
        width: 1000,
        height: 1000,
    };

    let scaled = res.scale(50);

    assert_eq!(
        scaled,
        Resolution {
            width: 500,
            height: 500
        }
    );
}

#[test]
fn test_res_to_image() {
    let res = Resolution {
        width: 1000,
        height: 1000,
    };

    let scaling = [Scale::Small(10), Scale::Medium(40), Scale::Large(80)];

    let image = res.to_image(&scaling);

    assert_eq!(
        image,
        Image {
            original: res,
            small: Resolution {
                width: 100,
                height: 100
            },
            medium: Resolution {
                width: 400,
                height: 400
            },
            large: Resolution {
                width: 800,
                height: 800
            }
        }
    );
}
