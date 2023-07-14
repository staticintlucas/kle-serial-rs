//! A Rust library for deserialising [Keyboard Layout Editor] files. Designed to be used in
//! conjunction with [`serde_json`] to deserialize JSON files exported from KLE.
//!
//! # Example
//!
//!
//! ```
//! use kle_serial::Keyboard;
//!
//! let keyboard: Keyboard = serde_json::from_str(
//!     r#"[
//!         {"name": "example"},
//!         [{"f": 4}, "!\n1\n¹\n¡"]
//!     ]"#
//! ).unwrap();
//!
//! assert_eq!(keyboard.metadata.name, "example");
//! assert_eq!(keyboard.keys.len(), 1);
//!
//! let legend = keyboard.keys[0].legends[0].as_ref().unwrap();
//!
//! assert_eq!(legend.text, "!");
//! assert_eq!(legend.size, 4);
//! ```
//!
//! [Keyboard Layout Editor]: http://www.keyboard-layout-editor.com/
//! [`serde_json`]: https://crates.io/crates/serde_json

#![warn(missing_docs, dead_code)]
#![warn(clippy::all, clippy::pedantic, clippy::cargo)]

mod de;
mod utils;

use serde::Deserialize;

use de::{KleKeyboard, KleLayoutIterator};
use utils::FontSize;

/// Colour type used for deserialising. Type alias of [`rgb::RGBA8`].
pub type Color = rgb::RGBA8;

const NUM_LEGENDS: usize = 12; // Number of legends on a key

pub(crate) mod color {
    use crate::Color;

    pub(crate) const BACKGROUND: Color = Color::new(0xEE, 0xEE, 0xEE, 0xFF); // #EEEEEE
    pub(crate) const KEY: Color = Color::new(0xCC, 0xCC, 0xCC, 0xFF); // #CCCCCC
    pub(crate) const LEGEND: Color = Color::new(0x00, 0x00, 0x00, 0xFF); // #000000
}

/// Struct containing data for a single legend.
#[derive(Debug, Clone)]
pub struct Legend {
    /// The legend's text.
    pub text: String,
    /// The legend size (in KLE's font size unit).
    pub size: usize,
    /// The legend colour.
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

/// Struct containing data on a key switch.
#[derive(Debug, Clone, Default)]
pub struct Switch {
    /// The switch mount. Typically either "cherry" or "alps".
    pub mount: String,
    /// The switch brand. KLE uses lowercase brand names.
    pub brand: String,
    /// The switch type. KLE uses either part number or colour depending on the brand.
    pub typ: String,
}

/// Struct containing data on a single key
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Key {
    /// Array of legends on the key. This array is indexed in left to right, top to bottom order
    /// as shown in the image below.
    ///
    /// ![image](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAADgAAAA4CAIAAAAn5KxJAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAAJcEhZcwAADsMAAA7DAcdvqGQAAAL1SURBVGhD7Zo9cvIwEIahy2kylHTcwceAjpuQGRouQUdaDpCCOoegpUmRd63Vz9qyVpYdx2TyVLH9evfRWvDNN2ZxfxK86Ol0Wq/Xy+VyMQOgARkosZwT3e12q9XqeDx+zAbIQAlixpBEIY5TfH1mQMzMlUQx5PYsr9fr++SgKbe3QAx6LIoNwactuOF2uz0ej68JQTs0bbtCj0WxefmcBYub2NKApmjNEhbopUT51skZInrZvtRsDp98RuPzsHnZXvigA8owYbRY1PfM6U7wutJRhOy6qa6fQbEoKtqWGaa15OZw0ZJBVRAelYpCzq82GEOa3OEbRNlSUVHkJ0Tlg5/rRFGwWbFUlGrZlvlzyklSJhIqFqV6ZtH5nnq0wxIUiwJ6QETeYyc00dpT4NJDRCflX3RsBojaLar9qxiif/DkNvXZUtGgo97cYiTS2eBrTzBgoo5MU8Q2260SpVD0S2QE0TxPY6Bm/X4CYXKYKPUF+hepHZQmStddNXtTzQgTrceQdPUdNVFJuF9HEe38BBjITpJclmcMUTFEbaIebaKiEsI+Wz7RYE6ZlkB/9F1ly0Un5l90bIaIuu2k7NFg2zGpberTIlUu6r878j/0QAn7yyQcJItFvWcftDXBzl8XLUpF64qHnv8XoSEpqxt9olTGNhVjSKCNk0EssvxBE7WF8gxyUuEYSXj4o0eZnqL9Q+KgWJTK9Hn0efvjByYKqFKN8gGpEeNJ4arK/BDRSfm7os/xsmFWr29SogA3YHET07YEiqjlvH/FdeJ1f+Zzcd6qRfXGf9NBx00i1jqMgCq6KGnaOqmS7BVEjaH7w9COhYcdIJEl6ho1mgac93v0QpZbBnfhJufRiDXv6iJLtC6E60RVVR2iBiFqm7c9GmfagSZozaLtl7ZRUDJfNDZRQ29R/9I2+hqc8Y26nzwTtHTZyE39RMVr8PQPC9DLoCxctrR3tdfWT1T8sAA8x081DBDHkLEhzCh+F2hAxszS4EVnzf3+DR1Awz3+fptYAAAAAElFTkSuQmCC)
    ///
    /// Legends that are empty in KLE will be [`None`] when deserialised.
    pub legends: [Option<Legend>; NUM_LEGENDS],
    /// The colour of the key
    pub color: Color,
    /// The X position of the key in keyboard units (19.05 mm or 0.75 in).
    pub x: f64,
    /// The Y position of the key in keyboard units (19.05 mm or 0.75 in).
    pub y: f64,
    /// The width of the key in keyboard units (19.05 mm or 0.75 in).
    pub width: f64,
    /// The height of the key in keyboard units (19.05 mm or 0.75 in).
    pub height: f64,
    /// The relative X position of a stepped or L-shaped part of the key. This is set to 0.0 for
    /// regular keys, but is used for stepped caps lock and ISO enter keys, for example.
    pub x2: f64,
    /// The relative Y position of a stepped or L-shaped part of the key. This is set to 0.0 for
    /// regular keys, but is used for stepped caps lock and ISO enter keys, for example.
    pub y2: f64,
    /// The width of a stepped or L-shaped part of the key. This is equal to the width for
    /// regular keys, but is used for stepped caps lock and ISO enter keys, for example.
    pub width2: f64,
    /// The height of a stepped or L-shaped part of the key. This is equal to the height for
    /// regular keys, but is used for stepped caps lock and ISO enter keys, for example.
    pub height2: f64,
    /// The rotation of the key in degrees.
    pub rotation: f64,
    /// The X coordinate for the centre of rotation of the key.
    pub rx: f64,
    /// The Y coordinate for the centre of rotation of the key.
    pub ry: f64,
    /// The keycap profile of the key.
    pub profile: String,
    /// The key switch.
    pub switch: Switch,
    /// Whether the key is ghosted.
    pub ghosted: bool,
    /// Whether the key is stepped.
    pub stepped: bool,
    /// Whether this is a homing key.
    pub homing: bool,
    /// Whether this is a decal.
    pub decal: bool,
}

impl Default for Key {
    fn default() -> Self {
        Self {
            legends: std::array::from_fn(|_| None),
            color: color::KEY,
            x: 0.,
            y: 0.,
            width: 1.,
            height: 1.,
            x2: 0.,
            y2: 0.,
            width2: 1.,
            height2: 1.,
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

/// Struct containing the background style of a KLE layout.
#[derive(Debug, Clone, Default)]
pub struct Background {
    /// The name of the background.
    pub name: String,
    /// The CSS style of the background.
    pub style: String,
}

/// Metadata struct for the keyboard layout
#[derive(Debug, Clone)]
pub struct Metadata {
    /// Background colour for the layout.
    pub background_color: Color,
    /// Background style information for the layout.
    pub background: Background,
    /// Corner radii for the background using CSS `border-radius` syntax.
    pub radii: String,
    /// The name of the layout.
    pub name: String,
    /// The author of the layout.
    pub author: String,
    /// The default switch type used by the layout. This can also be set for individual keys.
    pub switch: Switch,
    /// Whether the switch is plate mounted.
    pub plate_mount: bool,
    /// Whether the switch is PCB mounted.
    pub pcb_mount: bool,
    /// Notes for the layout. KLE expects Markdown syntax.
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

/// A keyboard deserialised from a KLE JSON file.
#[derive(Debug, Clone, Default)]
pub struct Keyboard {
    /// Keyboard layout metadata.
    pub metadata: Metadata,
    /// The keys in the layout.
    pub keys: Vec<Key>,
}

impl<'de> Deserialize<'de> for Keyboard {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let KleKeyboard { meta, layout } = KleKeyboard::deserialize(deserializer)?;

        Ok(Self {
            metadata: meta.into(),
            keys: KleLayoutIterator::new(layout).collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legend_default() {
        let legend = Legend::default();

        assert_eq!(legend.text, "");
        assert_eq!(usize::from(legend.size), 3);
        assert_eq!(legend.color, Color::new(0, 0, 0, 255));
    }

    #[test]
    fn test_key_default() {
        let key = Key::default();

        for leg in key.legends {
            assert!(leg.is_none());
        }
        assert_eq!(key.color, Color::new(204, 204, 204, 255));
        assert_eq!(key.x, 0.);
        assert_eq!(key.y, 0.);
        assert_eq!(key.width, 1.);
        assert_eq!(key.height, 1.);
        assert_eq!(key.x2, 0.);
        assert_eq!(key.y2, 0.);
        assert_eq!(key.width2, 1.);
        assert_eq!(key.height2, 1.);
        assert_eq!(key.rotation, 0.);
        assert_eq!(key.rx, 0.);
        assert_eq!(key.ry, 0.);
        assert_eq!(key.profile, "");
        assert_eq!(key.switch.mount, "");
        assert_eq!(key.switch.brand, "");
        assert_eq!(key.switch.typ, "");
        assert!(!key.ghosted);
        assert!(!key.stepped);
        assert!(!key.homing);
        assert!(!key.decal);
    }

    #[test]
    fn test_metadata_default() {
        let meta = Metadata::default();

        assert_eq!(meta.background_color, Color::new(238, 238, 238, 255));
        assert_eq!(meta.background.name, "");
        assert_eq!(meta.background.style, "");
        assert_eq!(meta.radii, "");
        assert_eq!(meta.name, "");
        assert_eq!(meta.author, "");
        assert_eq!(meta.switch.mount, "");
        assert_eq!(meta.switch.brand, "");
        assert_eq!(meta.switch.typ, "");
        assert!(!meta.plate_mount);
        assert!(!meta.pcb_mount);
        assert_eq!(meta.notes, "");
    }

    #[test]
    fn test_keyboard_deserialize() {
        let kb: Keyboard = serde_json::from_str(
            r#"[
                {
                    "name": "test",
                    "unknown": "key"
                },
                [
                    {
                        "a": 4,
                        "unknown2": "key"
                    },
                    "A",
                    "B",
                    "C"
                ],
                [
                    "D"
                ]
            ]"#,
        )
        .unwrap();
        assert_eq!(kb.metadata.name, "test");
        assert_eq!(kb.keys.len(), 4);

        let kb: Keyboard = serde_json::from_str(r#"[["A"]]"#).unwrap();
        assert_eq!(kb.metadata.name, "");
        assert_eq!(kb.keys.len(), 1);

        let kb: Keyboard = serde_json::from_str(r#"[{"notes": "'tis a test"}]"#).unwrap();
        assert_eq!(kb.metadata.notes, "'tis a test");
        assert_eq!(kb.keys.len(), 0);

        assert!(serde_json::from_str::<Keyboard>("null").is_err());
    }
}
