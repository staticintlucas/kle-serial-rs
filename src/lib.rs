#![warn(missing_docs, dead_code)]
#![warn(clippy::all, clippy::pedantic, clippy::cargo)]
#![allow(missing_docs, clippy::missing_errors_doc)] // TODO

mod de;
mod error;
mod utils;

use smart_default::SmartDefault as Default;

pub use error::{Error, Result};

use de::{KleKeyboard, KleLegendsOrProps, KleProps};

pub type Color = rgb::RGBA8;

const NUM_LEGENDS: usize = 12; // Number of legends on a key

pub(crate) mod defaults {
    use crate::Color;

    pub const FONT_SIZE: usize = 3; // The default font size
    pub const ALIGNMENT: usize = 4; // The default alignment

    pub const BACKGROUND_COLOR: Color = Color::new(0xEE, 0xEE, 0xEE, 0xFF); // #EEEEEE
    pub const KEY_COLOR: Color = Color::new(0xCC, 0xCC, 0xCC, 0xFF); // #CCCCCC
    pub const LEGEND_COLOR: Color = Color::new(0x00, 0x00, 0x00, 0xFF); // #000000
}

#[derive(Debug, Clone, Default)]
pub struct Legend {
    pub text: String,
    #[default = 4]
    pub size: usize,
    #[default(defaults::LEGEND_COLOR)]
    pub color: Color,
}

#[derive(Debug, Clone, Default)]
pub struct Switch {
    pub mount: String,
    pub brand: String,
    pub typ: String,
}

#[derive(Debug, Clone, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct Key {
    pub legends: [Option<Legend>; NUM_LEGENDS],
    #[default(defaults::KEY_COLOR)]
    pub color: Color,
    pub x: f64,
    pub y: f64,
    #[default = 1.]
    pub w: f64,
    #[default = 1.]
    pub h: f64,
    pub x2: f64,
    pub y2: f64,
    #[default = 1.]
    pub w2: f64,
    #[default = 1.]
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

#[derive(Debug, Clone, Default)]
pub struct Background {
    pub name: String,
    pub style: String,
}

#[derive(Debug, Clone, Default)]
pub struct Metadata {
    #[default(defaults::BACKGROUND_COLOR)]
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

#[derive(Debug, Clone, Default)]
pub struct Keyboard {
    pub metadata: Metadata,
    pub keys: Vec<Key>,
}

impl Keyboard {
    pub fn from_json(json: &str) -> Result<Self> {
        let kle: KleKeyboard = serde_json::from_str(json)?;

        let mut state = KleProps::default();
        let mut keys = Vec::with_capacity(kle.rows.iter().flatten().count());

        for row in kle.rows {
            for key_or_props in row {
                match key_or_props {
                    KleLegendsOrProps::Props(props) => {
                        state.update(*props);
                    }
                    KleLegendsOrProps::Legend(text) => {
                        keys.push(state.build_key(&text)?);
                        state.next_key();
                    }
                }
            }
            state.next_line();
        }

        Ok(Self {
            metadata: Metadata::default(), // TODO: parse metadata
            keys,
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
