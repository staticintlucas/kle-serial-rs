mod json;

use std::vec;

use crate::{
    color,
    utils::{realign_legends, Alignment, FontSize},
    Background, Color, Key, Legend, Metadata, Switch, NUM_LEGENDS,
};
use json::{KleBackground, KleLegendsOrProps, KleMetadata, KlePropsObject};

pub(crate) use json::KleKeyboard;
use num_traits::real::Real;

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

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
struct KleProps<T = f64>
where
    T: Real,
{
    // Per-key properties
    x: T,
    y: T,
    w: T,
    h: T,
    x2: T,
    y2: T,
    w2: T,
    h2: T,
    l: bool, // stepped
    n: bool, // homing
    d: bool, // decal

    // Persistent properties
    r: T,
    rx: T,
    ry: T,
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

impl<T> KleProps<T>
where
    T: Real,
{
    fn update(&mut self, props: KlePropsObject<T>) {
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
        self.x = x + props.x.unwrap_or(T::zero());
        self.y = y + props.y.unwrap_or(T::zero());
        self.w = props.w.unwrap_or(T::one());
        self.h = props.h.unwrap_or(T::one());
        self.x2 = props.x2.unwrap_or(T::zero());
        self.y2 = props.y2.unwrap_or(T::zero());
        self.w2 = props.w2.or(props.w).unwrap_or(T::one());
        self.h2 = props.h2.or(props.h).unwrap_or(T::one());
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
        self.x = self.x + self.w;
        // Reset per-key properties
        self.w = T::one();
        self.h = T::one();
        self.x2 = T::zero();
        self.y2 = T::zero();
        self.w2 = T::one();
        self.h2 = T::one();
        self.l = false;
        self.n = false;
        self.d = false;
    }

    #[inline]
    fn next_line(&mut self) {
        self.next_key();
        self.x = self.rx; // x resets to rx
        self.y = self.y + T::one();
    }

    fn build_key(&self, legends: &str) -> Key<T> {
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

impl<T> Default for KleProps<T>
where
    T: Real,
{
    fn default() -> Self {
        Self {
            x: T::zero(),
            y: T::zero(),
            w: T::one(),
            h: T::one(),
            x2: T::zero(),
            y2: T::zero(),
            w2: T::one(),
            h2: T::one(),
            l: false,
            n: false,
            d: false,
            r: T::zero(),
            rx: T::zero(),
            ry: T::zero(),
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

#[derive(Debug, Clone)]
pub(crate) struct KleLayoutIterator<T = f64>
where
    T: Real,
{
    state: KleProps<T>,
    row_iter: vec::IntoIter<Vec<KleLegendsOrProps<T>>>,
    key_iter: vec::IntoIter<KleLegendsOrProps<T>>,
}

impl<T> KleLayoutIterator<T>
where
    T: Real,
{
    pub(crate) fn new(kle: Vec<Vec<KleLegendsOrProps<T>>>) -> Self {
        let state = KleProps::default();
        let mut row_iter = kle.into_iter();
        let key_iter = row_iter.next().unwrap_or_default().into_iter();
        KleLayoutIterator {
            state,
            row_iter,
            key_iter,
        }
    }
}

impl<T> Iterator for KleLayoutIterator<T>
where
    T: Real,
{
    type Item = Key<T>;

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
    use isclose::assert_is_close;

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
    #[allow(clippy::too_many_lines)]
    fn test_kle_props_update() {
        let props_obj = KlePropsObject::<f64> {
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

        assert_is_close!(props.x, 0.0);
        assert_is_close!(props.y, 0.0);
        assert_is_close!(props.w, 1.0);
        assert_is_close!(props.h, 1.0);
        assert_is_close!(props.x2, 0.0);
        assert_is_close!(props.y2, 0.0);
        assert_is_close!(props.w2, 1.0);
        assert_is_close!(props.h2, 1.0);
        assert!(!props.l);
        assert!(!props.n);
        assert!(!props.d);
        assert_is_close!(props.r, 0.0);
        assert_is_close!(props.rx, 0.0);
        assert_is_close!(props.ry, 0.0);
        assert!(!props.g);
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
            x: Some(1.0),
            y: Some(1.0),
            w: Some(2.0),
            h: Some(2.0),
            x2: Some(1.5),
            y2: Some(1.5),
            w2: Some(2.5),
            h2: Some(2.5),
            l: Some(true),
            n: Some(true),
            d: Some(true),
            r: Some(15.0),
            rx: Some(1.0),
            ry: Some(1.0),
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

        assert_is_close!(props.x, 2.0); // rx adds for whatever reason
        assert_is_close!(props.y, 2.0);
        assert_is_close!(props.w, 2.0);
        assert_is_close!(props.h, 2.0);
        assert_is_close!(props.x2, 1.5);
        assert_is_close!(props.y2, 1.5);
        assert_is_close!(props.w2, 2.5);
        assert_is_close!(props.h2, 2.5);
        assert!(props.l);
        assert!(props.n);
        assert!(props.d);
        assert_is_close!(props.r, 15.0);
        assert_is_close!(props.rx, 1.0);
        assert_is_close!(props.ry, 1.0);
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

        assert_is_close!(props.x, 5.0);
        assert_is_close!(props.y, 0.0);
        assert_is_close!(props.w, 1.0);
        assert_is_close!(props.h, 1.0);
        assert_is_close!(props.x2, 0.0);
        assert_is_close!(props.y2, 0.0);
        assert_is_close!(props.w2, 1.0);
        assert_is_close!(props.h2, 1.0);
        assert!(!props.l);
        assert!(!props.n);
        assert!(!props.d);
    }

    #[test]
    fn test_kle_props_next_line() {
        let mut props = KleProps {
            x: 2.0,
            ..KleProps::default()
        };
        props.next_line();

        assert_is_close!(props.x, 0.0);
        assert_is_close!(props.y, 1.0);
        assert_is_close!(props.w, 1.0);
        assert_is_close!(props.h, 1.0);
        assert_is_close!(props.x2, 0.0);
        assert_is_close!(props.y2, 0.0);
        assert_is_close!(props.w2, 1.0);
        assert_is_close!(props.h2, 1.0);
        assert!(!props.l);
        assert!(!props.n);
        assert!(!props.d);
    }

    #[test]
    fn test_kle_props_build_key() {
        let legends = "A\nB\nC\nD\nE\nF\nG\nH\nI\nJ\nK\nL";
        let expected = ["A", "I", "C", "G", "J", "H", "B", "K", "D", "F", "E", "L"];

        let props = <KleProps>::default();
        let key = props.build_key(legends);

        for (res, exp) in key.legends.iter().zip(expected) {
            assert_eq!(res.as_ref().unwrap().text, exp);
            assert_eq!(res.as_ref().unwrap().size, usize::from(FontSize::default()));
            assert_eq!(res.as_ref().unwrap().color, color::LEGEND);
        }
        assert_eq!(key.color, color::KEY);
        assert_is_close!(key.x, 0.0);
        assert_is_close!(key.y, 0.0);
        assert_is_close!(key.width, 1.0);
        assert_is_close!(key.height, 1.0);
        assert_is_close!(key.x2, 0.0);
        assert_is_close!(key.y2, 0.0);
        assert_is_close!(key.width2, 1.0);
        assert_is_close!(key.height2, 1.0);
        assert_is_close!(key.rotation, 0.0);
        assert_is_close!(key.rx, 0.0);
        assert_is_close!(key.ry, 0.0);
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
        assert_is_close!(keys[0].x, 0.0);
        assert_is_close!(keys[1].x, 1.0);
        assert_is_close!(keys[2].x, 1.5);
        assert_is_close!(keys[3].x, 0.0);
    }
}
