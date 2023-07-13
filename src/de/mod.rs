mod json;

use std::vec;

use crate::{
    color,
    utils::{realign_legends, Alignment, FontSize},
    Background, Color, Key, Legend, Metadata, Switch, NUM_LEGENDS,
};
use json::{KleBackground, KleLegendsOrProps, KleMetadata, KlePropsObject};

pub(crate) use json::KleKeyboard;

impl From<KleBackground> for Background {
    fn from(value: KleBackground) -> Self {
        let default = Self::default();
        Self {
            name: value.name.unwrap_or(default.name),
            style: value.style.unwrap_or(default.style),
        }
    }
}

impl From<KleMetadata> for Metadata {
    fn from(value: KleMetadata) -> Self {
        let default = Self::default();

        Self {
            background_color: value.backcolor.unwrap_or(default.background_color),
            background: value
                .background
                .map_or(default.background, Background::from),
            radii: value.radii.unwrap_or(default.radii),
            name: value.name.unwrap_or(default.name),
            author: value.author.unwrap_or(default.author),
            switch: Switch {
                mount: value.switch_mount.unwrap_or(default.switch.mount),
                brand: value.switch_brand.unwrap_or(default.switch.brand),
                typ: value.switch_type.unwrap_or(default.switch.typ),
            },
            plate_mount: value.plate.unwrap_or(default.plate_mount),
            pcb_mount: value.pcb.unwrap_or(default.pcb_mount),
            notes: value.notes.unwrap_or(default.notes),
        }
    }
}

#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
struct KleProps {
    // Per-key properties
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    x2: f64,
    y2: f64,
    w2: f64,
    h2: f64,
    l: bool, // stepped
    n: bool, // homing
    d: bool, // decal

    // Persistent properties
    r: f64,
    rx: f64,
    ry: f64,
    g: bool,                     // ghosted
    sm: String,                  // switch mount
    sb: String,                  // switch brand
    st: String,                  // switch type
    c: Color,                    // color
    t: Color,                    // fallback legend color
    ta: [Color; NUM_LEGENDS],    // legend color array
    a: Alignment,                // alignment
    p: String,                   // profile
    f: FontSize,                 // fallback font size
    fa: [FontSize; NUM_LEGENDS], // font size array
}

impl KleProps {
    fn update(&mut self, props: KlePropsObject) {
        let f = props.f.unwrap_or(self.f);
        let fa = if let Some(fa) = props.fa {
            std::array::from_fn(|i| match fa.get(i).copied() {
                Some(fa) if usize::from(fa) > 0 => fa,
                _ => f,
            })
        } else if let Some(f2) = props.f2 {
            std::array::from_fn(|i| if i == 0 { f } else { f2 })
        } else if let Some(f) = props.f {
            [f; NUM_LEGENDS]
        } else {
            self.fa
        };

        let t = (props.t.as_ref())
            .and_then(|v| v.first().copied().flatten())
            .unwrap_or(self.t);
        let ta = props.t.map_or(self.ta, |ta| {
            std::array::from_fn(|i| ta.get(i).copied().flatten().unwrap_or(t))
        });

        // KLE has some weird rotation behaviour, with rx and ry (if present) resetting x and y
        // Also note: KLE only allows r, rx, ry at the start of the line. We don't enforce this,
        // but expect some weird behaviour if you try to use it
        let (x, y, rx, ry) = if props.rx.is_some() || props.ry.is_some() {
            let rx = props.rx.unwrap_or(self.rx);
            let ry = props.ry.unwrap_or(self.ry);
            (rx, ry, rx, ry)
        } else {
            (self.x, self.y, self.rx, self.ry)
        };

        // Per-key properties
        self.x = x + props.x.unwrap_or(0.0);
        self.y = y + props.y.unwrap_or(0.0);
        self.w = props.w.unwrap_or(1.);
        self.h = props.h.unwrap_or(1.);
        self.x2 = props.x2.unwrap_or(0.);
        self.y2 = props.y2.unwrap_or(0.);
        self.w2 = props.w2.or(props.w).unwrap_or(1.);
        self.h2 = props.h2.or(props.h).unwrap_or(1.);
        self.l = props.l.unwrap_or(false);
        self.n = props.n.unwrap_or(false);
        self.d = props.d.unwrap_or(false);
        // Persistent properties
        self.r = props.r.unwrap_or(self.r);
        self.rx = rx;
        self.ry = ry;
        self.g = props.g.unwrap_or(self.g);
        self.sm = props.sm.unwrap_or(self.sm.clone());
        self.sb = props.sb.unwrap_or(self.sb.clone());
        self.st = props.st.unwrap_or(self.st.clone());
        self.c = props.c.unwrap_or(self.c);
        self.t = t;
        self.ta = ta;
        self.a = props.a.unwrap_or(self.a);
        self.p = props.p.unwrap_or(self.p.clone());
        self.f = f;
        self.fa = fa;
    }

    #[inline]
    fn next_key(&mut self) {
        // Increment x
        self.x += self.w.max(self.x2 + self.w2);
        // Reset per-key properties
        self.w = 1.;
        self.h = 1.;
        self.x2 = 0.;
        self.y2 = 0.;
        self.w2 = 1.;
        self.h2 = 1.;
        self.l = false;
        self.n = false;
        self.d = false;
    }

    #[inline]
    fn next_line(&mut self) {
        self.next_key();
        self.x = self.rx; // x resets to rx
        self.y += 1.;
    }

    fn build_key(&self, legends: &str) -> Key {
        let legends =
            legends
                .lines()
                .zip(self.fa.into_iter().zip(self.ta))
                .map(|(text, (size, color))| {
                    (!text.is_empty()).then_some(Legend {
                        text: text.into(),
                        size: usize::from(size),
                        color,
                    })
                });
        let legends = realign_legends(legends, self.a);

        Key {
            legends,
            color: self.c,
            x: self.x,
            y: self.y,
            width: self.w,
            height: self.h,
            x2: self.x2,
            y2: self.y2,
            width2: self.w2,
            height2: self.h2,
            rotation: self.r,
            rx: self.rx,
            ry: self.ry,
            profile: self.p.clone(),
            ghosted: self.g,
            switch: Switch {
                mount: self.sm.clone(),
                brand: self.sb.clone(),
                typ: self.st.clone(),
            },
            stepped: self.l,
            homing: self.n,
            decal: self.d,
        }
    }
}

impl Default for KleProps {
    fn default() -> Self {
        Self {
            x: 0.,
            y: 0.,
            w: 1.,
            h: 1.,
            x2: 0.,
            y2: 0.,
            w2: 1.,
            h2: 1.,
            l: false,
            n: false,
            d: false,
            r: 0.,
            rx: 0.,
            ry: 0.,
            g: false,
            sm: String::new(),
            sb: String::new(),
            st: String::new(),
            c: color::KEY,
            t: color::LEGEND,
            ta: [color::LEGEND; NUM_LEGENDS],
            a: Alignment::default(),
            p: String::new(),
            f: FontSize::default(),
            fa: [FontSize::default(); NUM_LEGENDS],
        }
    }
}

pub(crate) struct KleLayoutIterator {
    state: KleProps,
    row_iter: vec::IntoIter<Vec<KleLegendsOrProps>>,
    key_iter: vec::IntoIter<KleLegendsOrProps>,
}

impl KleLayoutIterator {
    pub(crate) fn new(kle: Vec<Vec<KleLegendsOrProps>>) -> Self {
        let state = KleProps::default();
        let mut row_iter = kle.into_iter();
        let key_iter = row_iter.next().unwrap_or(Vec::new()).into_iter();
        KleLayoutIterator {
            state,
            row_iter,
            key_iter,
        }
    }
}

impl Iterator for KleLayoutIterator {
    type Item = Key;

    fn next(&mut self) -> Option<Self::Item> {
        let legends = loop {
            let key = self.key_iter.next().or_else(|| {
                self.key_iter = self.row_iter.next()?.into_iter();
                self.state.next_line();
                self.key_iter.next()
            })?;

            match key {
                KleLegendsOrProps::Props(props) => self.state.update(*props),
                KleLegendsOrProps::Legend(str) => break str,
            }
        };

        let key = self.state.build_key(&legends);
        self.state.next_key();

        Some(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_background_from() {
        let bg = Background::from(KleBackground::default());

        assert_eq!(bg.name, Background::default().name);
        assert_eq!(bg.style, Background::default().style);

        let bg = Background::from(KleBackground {
            name: Some("name".into()),
            style: Some("style".into()),
        });

        assert_eq!(bg.name, "name");
        assert_eq!(bg.style, "style");
    }

    #[test]
    fn test_metadata_from() {
        let md = Metadata::from(KleMetadata::default());

        assert_eq!(md.background_color, Metadata::default().background_color);
        assert_eq!(md.background.name, Metadata::default().background.name);
        assert_eq!(md.background.style, Metadata::default().background.style);
        assert_eq!(md.radii, Metadata::default().radii);
        assert_eq!(md.name, Metadata::default().name);
        assert_eq!(md.author, Metadata::default().author);
        assert_eq!(md.switch.mount, Metadata::default().switch.mount);
        assert_eq!(md.switch.brand, Metadata::default().switch.brand);
        assert_eq!(md.switch.typ, Metadata::default().switch.typ);
        assert_eq!(md.plate_mount, Metadata::default().plate_mount);
        assert_eq!(md.pcb_mount, Metadata::default().pcb_mount);
        assert_eq!(md.notes, Metadata::default().notes);

        let md: Metadata = Metadata::from(KleMetadata {
            author: Some("author".into()),
            backcolor: Some(Color::new(204, 34, 34, 255)),
            background: Some(KleBackground {
                name: Some("name".into()),
                style: Some("style".into()),
            }),
            name: Some("name".into()),
            notes: Some("notes".into()),
            radii: Some("radii".into()),
            switch_mount: Some("switch_mount".into()),
            switch_brand: Some("switch_brand".into()),
            switch_type: Some("switch_type".into()),
            css: Some("css".into()),
            pcb: Some(true),
            plate: Some(true),
        });

        assert_eq!(md.background_color, Color::new(204, 34, 34, 255));
        assert_eq!(md.background.name, "name");
        assert_eq!(md.background.style, "style");
        assert_eq!(md.radii, "radii");
        assert_eq!(md.name, "name");
        assert_eq!(md.author, "author");
        assert_eq!(md.switch.mount, "switch_mount");
        assert_eq!(md.switch.brand, "switch_brand");
        assert_eq!(md.switch.typ, "switch_type");
        assert!(md.plate_mount);
        assert!(md.pcb_mount);
        assert_eq!(md.notes, "notes");
    }

    #[test]
    fn test_kle_props_update() {
        let props_obj = KlePropsObject {
            x: None,
            y: None,
            w: None,
            h: None,
            x2: None,
            y2: None,
            w2: None,
            h2: None,
            l: None,
            n: None,
            d: None,
            r: None,
            rx: None,
            ry: None,
            g: None,
            sm: None,
            sb: None,
            st: None,
            c: None,
            t: None,
            a: None,
            p: None,
            f: None,
            f2: None,
            fa: None,
        };
        let mut props = KleProps::default();
        props.update(props_obj);

        assert_eq!(props.x, 0.);
        assert_eq!(props.y, 0.);
        assert_eq!(props.w, 1.);
        assert_eq!(props.h, 1.);
        assert_eq!(props.x2, 0.);
        assert_eq!(props.y2, 0.);
        assert_eq!(props.w2, 1.);
        assert_eq!(props.h2, 1.);
        assert_eq!(props.l, false);
        assert_eq!(props.n, false);
        assert_eq!(props.d, false);
        assert_eq!(props.r, 0.);
        assert_eq!(props.rx, 0.);
        assert_eq!(props.ry, 0.);
        assert_eq!(props.g, false);
        assert_eq!(props.sm, "");
        assert_eq!(props.sb, "");
        assert_eq!(props.st, "");
        assert_eq!(props.c, color::KEY);
        assert_eq!(props.t, color::LEGEND);
        assert_eq!(props.ta, [color::LEGEND; NUM_LEGENDS]);
        assert_eq!(props.a, Alignment::default());
        assert_eq!(props.p, "");
        assert_eq!(props.f, FontSize::default());
        assert_eq!(props.fa, [FontSize::default(); NUM_LEGENDS]);

        let props_obj = KlePropsObject {
            x: Some(1.),
            y: Some(1.),
            w: Some(2.),
            h: Some(2.),
            x2: Some(1.5),
            y2: Some(1.5),
            w2: Some(2.5),
            h2: Some(2.5),
            l: Some(true),
            n: Some(true),
            d: Some(true),
            r: Some(15.),
            rx: Some(1.),
            ry: Some(1.),
            g: Some(true),
            sm: Some("cherry".into()),
            sb: Some("cherry".into()),
            st: Some("MX1A-31xx".into()),
            c: Some(Color::new(127, 51, 76, 255)),
            t: Some(vec![
                Some(Color::new(25, 25, 25, 255)),
                None,
                Some(Color::new(76, 38, 51, 255)),
            ]),
            a: Some(Alignment::new(5).unwrap()),
            p: Some("DSA".into()),
            f: Some(FontSize::new(4).unwrap()),
            f2: Some(FontSize::new(4).unwrap()),
            fa: Some(vec![FontSize::new(4).unwrap(); 3]),
        };
        props.update(props_obj);

        assert_eq!(props.x, 2.); // rx adds for whatever reason
        assert_eq!(props.y, 2.);
        assert_eq!(props.w, 2.);
        assert_eq!(props.h, 2.);
        assert_eq!(props.x2, 1.5);
        assert_eq!(props.y2, 1.5);
        assert_eq!(props.w2, 2.5);
        assert_eq!(props.h2, 2.5);
        assert_eq!(props.l, true);
        assert_eq!(props.n, true);
        assert_eq!(props.d, true);
        assert_eq!(props.r, 15.);
        assert_eq!(props.rx, 1.);
        assert_eq!(props.ry, 1.);
        assert!(props.g);
        assert_eq!(props.sm, "cherry");
        assert_eq!(props.sb, "cherry");
        assert_eq!(props.st, "MX1A-31xx");
        assert_eq!(props.c, Color::new(127, 51, 76, 255));
        assert_eq!(props.t, Color::new(25, 25, 25, 255));
        assert_eq!(
            props.ta,
            [
                Color::new(25, 25, 25, 255),
                Color::new(25, 25, 25, 255),
                Color::new(76, 38, 51, 255),
                Color::new(25, 25, 25, 255),
                Color::new(25, 25, 25, 255),
                Color::new(25, 25, 25, 255),
                Color::new(25, 25, 25, 255),
                Color::new(25, 25, 25, 255),
                Color::new(25, 25, 25, 255),
                Color::new(25, 25, 25, 255),
                Color::new(25, 25, 25, 255),
                Color::new(25, 25, 25, 255),
            ]
        );
        assert_eq!(usize::from(props.a), 5);
        assert_eq!(props.p, "DSA");
        assert_eq!(usize::from(props.f), 4);
        assert_eq!(props.fa.map(usize::from), [4; NUM_LEGENDS]);

        let props_obj = KlePropsObject {
            f: Some(FontSize::new(2).unwrap()),
            f2: Some(FontSize::new(4).unwrap()),
            ..KlePropsObject::default()
        };
        props.update(props_obj);
        assert_eq!(
            props.fa.map(usize::from),
            [2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4]
        );

        let rawprops4 = KlePropsObject {
            f: Some(FontSize::new(5).unwrap()),
            ..KlePropsObject::default()
        };
        props.update(rawprops4);
        assert_eq!(props.fa.map(usize::from), [5; NUM_LEGENDS]);
    }

    #[test]
    fn test_kle_props_next_key() {
        let mut props = KleProps {
            x: 2.0,
            w: 3.0,
            h: 1.5,
            ..KleProps::default()
        };
        props.next_key();

        assert_eq!(props.x, 5.);
        assert_eq!(props.y, 0.);
        assert_eq!(props.w, 1.);
        assert_eq!(props.h, 1.);
        assert_eq!(props.x2, 0.);
        assert_eq!(props.y2, 0.);
        assert_eq!(props.w2, 1.);
        assert_eq!(props.h2, 1.);
        assert_eq!(props.l, false);
        assert_eq!(props.n, false);
        assert_eq!(props.d, false);
    }

    #[test]
    fn test_kle_props_next_line() {
        let mut props = KleProps {
            x: 2.0,
            ..KleProps::default()
        };
        props.next_line();

        assert_eq!(props.x, 0.);
        assert_eq!(props.y, 1.);
        assert_eq!(props.w, 1.);
        assert_eq!(props.h, 1.);
        assert_eq!(props.x2, 0.);
        assert_eq!(props.y2, 0.);
        assert_eq!(props.w2, 1.);
        assert_eq!(props.h2, 1.);
        assert_eq!(props.l, false);
        assert_eq!(props.n, false);
        assert_eq!(props.d, false);
    }

    #[test]
    fn test_kle_props_build_key() {
        let legends = "A\nB\nC\nD\nE\nF\nG\nH\nI\nJ\nK\nL";
        let expected = ["A", "I", "C", "G", "J", "H", "B", "K", "D", "F", "E", "L"];

        let props = KleProps::default();
        let key = props.build_key(legends);

        for (res, exp) in key.legends.iter().zip(expected) {
            assert_eq!(res.as_ref().unwrap().text, exp);
            assert_eq!(res.as_ref().unwrap().size, usize::from(FontSize::default()));
            assert_eq!(res.as_ref().unwrap().color, color::LEGEND);
        }
        assert_eq!(key.color, color::KEY);
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

        let props = KleProps { d: true, ..props };
        let key = props.build_key(legends);
        assert!(key.decal);

        let props = KleProps { n: true, ..props };
        let key = props.build_key(legends);
        assert!(key.homing);

        let props = KleProps {
            p: "DSA".into(),
            ..props
        };
        let key = props.build_key(legends);
        assert_eq!(key.profile, "DSA");
    }

    #[test]
    fn test_kle_layout_iterator() {
        let kle: KleKeyboard = serde_json::from_str(
            r#"[
                {
                    "meta": "data"
                },
                [
                    {
                        "a": 4,
                        "unknown": "key"
                    },
                    "A",
                    "B",
                    {
                        "x": -0.5,
                        "y": 0.25
                    },
                    "C"
                ],
                [
                    "D"
                ]
            ]"#,
        )
        .unwrap();

        let iterator = KleLayoutIterator::new(kle.layout);
        let keys: Vec<_> = iterator.collect();

        assert_eq!(keys.len(), 4);
        assert_eq!(keys[0].x, 0.0);
        assert_eq!(keys[1].x, 1.0);
        assert_eq!(keys[2].x, 1.5);
        assert_eq!(keys[3].x, 0.0);
    }
}
