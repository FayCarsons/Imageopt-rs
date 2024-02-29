use serde::{Deserialize, Serialize};

pub type Title = String;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub fn scale(self, factor: u16) -> Self {
        let factor = f64::from(factor) / 100.;
        Self {
            width: (f64::from(self.width) * factor) as u32,
            height: (f64::from(self.height) * factor) as u32,
        }
    }

    pub fn to_image(self, [small, medium, large]: &Scaling) -> Image {
        Image {
            original: self,
            large: self.scale(large.inner()),
            medium: self.scale(medium.inner()),
            small: self.scale(small.inner()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Image {
    pub original: Resolution,
    pub large: Resolution,
    pub medium: Resolution,
    pub small: Resolution,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Scale {
    Large(u16),
    Medium(u16),
    Small(u16),
}

impl Scale {
    pub fn inner(self) -> u16 {
        match self {
            Self::Large(n) | Self::Medium(n) | Self::Small(n) => n,
        }
    }
}

pub type Scaling = [Scale; 3];

pub fn parse_scaling(s: &str) -> Result<Scaling, String> {
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
