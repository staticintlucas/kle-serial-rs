mod json;

use std::vec;

use crate::{
    color,
    utils::{realign_legends, Alignment, FontSize},
    Color, Key, Legend, Switch, NUM_LEGENDS,
};
use json::{KleLegendsOrProps, KlePropsObject};

pub(crate) use json::KleKeyboard;

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
            w: self.w,
            h: self.h,
            x2: self.x2,
            y2: self.y2,
            w2: self.w2,
            h2: self.h2,
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
