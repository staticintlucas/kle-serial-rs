#![warn(missing_docs, dead_code)]
#![warn(clippy::all, clippy::pedantic, clippy::cargo)]

//! A Rust library for deserialising [Keyboard Layout Editor] files. Designed to be used in
//! conjunction with [`serde_json`] to deserialize JSON files exported from KLE.
//!
//! # Example
//!
//! ![example]
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
//! assert!(keyboard.keys[0].legends[0].is_some());
//! let legend = keyboard.keys[0].legends[0].as_ref().unwrap();
//!
//! assert_eq!(legend.text, "!");
//! assert_eq!(legend.size, 4);
//!
//! assert!(keyboard.keys[0].legends[1].is_none());
//! ```
//!
//! [Keyboard Layout Editor]: http://www.keyboard-layout-editor.com/
//! [`serde_json`]: https://crates.io/crates/serde_json
#![cfg_attr(doc, doc = embed_doc_image::embed_image!("example", "doc/example.png"))]

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

/// A struct representing a single legend.
///
/// **Note**: This is called a `label` in the official TypeScript [`kle-serial`] library and some
/// other deserialisation libraries. It is called `Legend` here to follow the common terminology
/// and match KLE's own UI.
///
/// [`kle-serial`]: https://github.com/ijprest/kle-serial
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

/// A struct representing a key switch.
#[derive(Debug, Clone, Default)]
pub struct Switch {
    /// The switch mount. Typically either `"cherry"` or `"alps"`.
    pub mount: String,
    /// The switch brand. KLE uses lowercase brand names.
    pub brand: String,
    /// The switch type. KLE uses either part number or colour depending on the brand.
    pub typ: String,
}

/// A struct representing a single key.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Key {
    /// The key's legends. This array is indexed in left to right, top to bottom order as shown in
    /// the image below.
    ///
    /// ![alignment]
    ///
    /// Legends that are empty in KLE will be deserialised as [`None`].
    #[cfg_attr(doc, doc = embed_doc_image::embed_image!("alignment", "doc/alignment.png"))]
    pub legends: [Option<Legend>; NUM_LEGENDS],
    /// The colour of the key
    pub color: Color,
    /// The X position of the key in keyboard units (19.05 mm or 0.75 in).
    ///
    /// **Note**: KLE has some strange behaviour when it comes to stepped and L-shaped keys. The
    /// 'true' X position will be less if the key's `x2` field is negative. This behaviour can be
    /// observed by placing an ISO enter in KLE; `x` is 0.25 and `x2` is &minus;0.25.
    pub x: f64,
    /// The Y position of the key in keyboard units (19.05 mm or 0.75 in).
    ///
    /// **Note**: KLE has some strange behaviour when it comes to stepped and L-shaped keys. The
    /// 'true' Y position will be less if the key's `y2` field is negative. This behaviour can be
    /// observed by placing an ISO enter in KLE; `x` is 0.25 and `x2` is &minus;0.25.
    pub y: f64,
    /// The width of the key in keyboard units (19.05 mm or 0.75 in).
    pub width: f64,
    /// The height of the key in keyboard units (19.05 mm or 0.75 in).
    pub height: f64,
    /// The relative X position of a stepped or L-shaped part of the key.
    ///
    /// This is set to 0.0 for regular keys, but is used for stepped caps lock and ISO enter keys,
    /// amongst others.
    pub x2: f64,
    /// The relative Y position of a stepped or L-shaped part of the key.
    ///
    /// This is set to 0.0 for regular keys, but is used for stepped caps lock and ISO enter keys,
    /// amongst others.
    pub y2: f64,
    /// The width of a stepped or L-shaped part of the key.
    ///
    /// This is equal to the width for regular keys, but is used for stepped caps lock and ISO
    /// enter keys, amongst others.
    pub width2: f64,
    /// The height of a stepped or L-shaped part of the key.
    ///
    /// This is equal to the height for regular keys, but is used for stepped caps lock and ISO
    /// enter keys, amongst others.
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

/// The background style of a KLE layout.
#[derive(Debug, Clone, Default)]
pub struct Background {
    /// The name of the background.
    ///
    /// When generated by KLE, this is the same as the name shown in the dropdown menu, for example
    /// `"Carbon fibre 1"`.
    pub name: String,
    /// The CSS style of the background.
    ///
    /// When generated by KLE, this sets the [`background-image`] CSS property to a relative url
    /// where the associated image is located. For example the *Carbon fibre 1* background will set
    /// `style` to `"background-image: url('/bg/carbonfibre/carbon_texture1879.png');"`.
    ///
    /// [`background-image`]: https://developer.mozilla.org/en-US/docs/Web/CSS/background-image
    pub style: String,
}

/// The metadata for the keyboard layout.
#[derive(Debug, Clone)]
pub struct Metadata {
    /// Background colour for the layout.
    pub background_color: Color,
    /// Background style information for the layout.
    pub background: Background,
    /// Corner radii for the background using CSS [`border-radius`] syntax.
    ///
    /// [`border-radius`]: https://developer.mozilla.org/en-US/docs/Web/CSS/border-radius
    pub radii: String,
    /// The name of the layout.
    pub name: String,
    /// The author of the layout.
    pub author: String,
    /// The default switch type used by the layout. This can be set separately for individual keys.
    pub switch: Switch,
    /// Whether the switch is plate mounted.
    pub plate_mount: bool,
    /// Whether the switch is PCB mounted.
    pub pcb_mount: bool,
    /// Notes for the layout. KLE expects GitHub-flavoured Markdown and can render this using the
    /// *preview* button, but any string data is considered valid.
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
    /// Keyboard layout's metadata.
    pub metadata: Metadata,
    /// The layout's keys.
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

/// An iterator of [`Key`]s deserialised from a KLE JSON file.
#[derive(Debug, Clone)]
pub struct KeyIterator(KleLayoutIterator);

impl<'de> Deserialize<'de> for KeyIterator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // TODO don't allocate Vec's here?
        let KleKeyboard { meta: _, layout } = KleKeyboard::deserialize(deserializer)?;

        Ok(Self(KleLayoutIterator::new(layout)))
    }
}

impl Iterator for KeyIterator {
    type Item = Key;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
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

    #[test]
    fn test_key_iterator_deserialize() {
        let keys: Vec<_> = serde_json::from_str::<KeyIterator>(
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
        .unwrap()
        .collect();

        assert_eq!(keys.len(), 4);
        assert_eq!(keys[2].legends[0].as_ref().unwrap().text, "C");

        let keys: Vec<_> = serde_json::from_str::<KeyIterator>(r#"[["A"]]"#)
            .unwrap()
            .collect();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].legends[0].as_ref().unwrap().text, "A");

        let keys: Vec<_> = serde_json::from_str::<KeyIterator>(r#"[{"notes": "'tis a test"}]"#)
            .unwrap()
            .collect();
        assert_eq!(keys.len(), 0);

        assert!(serde_json::from_str::<KeyIterator>("null").is_err());
    }
}
