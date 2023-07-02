#![warn(missing_docs, dead_code)]
#![warn(clippy::all, clippy::pedantic, clippy::cargo)]
#![allow(missing_docs, clippy::missing_errors_doc)] // TODO

mod de;
mod utils;

use serde::Deserialize;

use de::{KleKeyboard, KleLayoutIterator};
use utils::FontSize;

pub type Color = rgb::RGBA8;

const NUM_LEGENDS: usize = 12; // Number of legends on a key

pub(crate) mod color {
    use crate::Color;

    pub(crate) const BACKGROUND: Color = Color::new(0xEE, 0xEE, 0xEE, 0xFF); // #EEEEEE
    pub(crate) const KEY: Color = Color::new(0xCC, 0xCC, 0xCC, 0xFF); // #CCCCCC
    pub(crate) const LEGEND: Color = Color::new(0x00, 0x00, 0x00, 0xFF); // #000000
}

#[derive(Debug, Clone)]
pub struct Legend {
    pub text: String,
    pub size: usize,
    pub color: Color,
}

impl Default for Legend {
    fn default() -> Self {
        Self {
            text: String::default(),
            size: usize::from(FontSize::default()),
            color: color::LEGEND,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Switch {
    pub mount: String,
    pub brand: String,
    pub typ: String,
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Key {
    pub legends: [Option<Legend>; NUM_LEGENDS],
    pub color: Color,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub x2: f64,
    pub y2: f64,
    pub w2: f64,
    pub h2: f64,
    pub rotation: f64,
    pub rx: f64,
    pub ry: f64,
    pub profile: String,
    pub switch: Switch,
    pub ghosted: bool,
    pub stepped: bool,
    pub homing: bool,
    pub decal: bool,
}

impl Default for Key {
    fn default() -> Self {
        Self {
            legends: std::array::from_fn(|_| None),
            color: color::KEY,
            x: 0.,
            y: 0.,
            w: 1.,
            h: 1.,
            x2: 0.,
            y2: 0.,
            w2: 1.,
            h2: 1.,
            rotation: 0.,
            rx: 0.,
            ry: 0.,
            profile: String::new(),
            switch: Switch::default(),
            ghosted: false,
            stepped: false,
            homing: false,
            decal: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Background {
    pub name: String,
    pub style: String,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub background_color: Color,
    pub background: Background,
    pub radii: String,
    pub name: String,
    pub author: String,
    pub switch: Switch,
    pub plate_mount: bool,
    pub pcb_mount: bool,
    pub notes: String,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            background_color: color::BACKGROUND,
            background: Background::default(),
            radii: String::new(),
            name: String::new(),
            author: String::new(),
            switch: Switch::default(),
            plate_mount: false,
            pcb_mount: false,
            notes: String::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Keyboard {
    pub metadata: Metadata,
    pub keys: Vec<Key>,
}

impl<'de> Deserialize<'de> for Keyboard {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let kle = KleKeyboard::deserialize(deserializer)?;

        Ok(Self {
            metadata: Metadata::default(), // TODO: parse metadata
            keys: KleLayoutIterator::new(kle.layout).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn it_works() {
        // let result = add(2, 2);
        // assert_eq!(result, 4);
    }
}
