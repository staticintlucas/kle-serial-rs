mod json;

use crate::utils::realign_legends;
use crate::{defaults, NUM_LEGENDS};
use crate::{Color, Key, Legend, Result, Switch};

use itertools::izip;
pub(crate) use json::{KleKeyboard, KleLegendsOrProps, KlePropsObject};

#[derive(Debug)]
pub(crate) struct KleProps {
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
    c: Color,                 // color
    t: Color,                 // fallback legend color
    ta: [Color; NUM_LEGENDS], // legend color array
    a: usize,                 // alignment
    p: String,                // profile
    f: usize,                 // fallback font size
    fa: [usize; NUM_LEGENDS], // font size array
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
            c: defaults::KEY_COLOR,
            t: defaults::LEGEND_COLOR,
            ta: [defaults::LEGEND_COLOR; NUM_LEGENDS],
            a: defaults::ALIGNMENT,
            p: String::new(),
            f: defaults::FONT_SIZE,
            fa: [defaults::FONT_SIZE; NUM_LEGENDS],
        }
    }
}

impl KleProps {
    pub(crate) fn update(&mut self, props: KlePropsObject) {
        let f = props.f.unwrap_or(self.f);
        let fa = if let Some(fa) = props.fa {
            std::array::from_fn(|i| match fa.get(i).copied() {
                Some(fa) if fa > 0 => fa,
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

        // Per-key properties
        self.x += props.x.unwrap_or(0.0);
        self.y += props.y.unwrap_or(0.0);
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
        self.c = props.c.unwrap_or(self.c);
        self.t = t;
        self.ta = ta;
        self.a = props.a.unwrap_or(self.a);
        self.p = props.p.unwrap_or(self.p.clone());
        self.f = f;
        self.fa = fa;
    }

    #[inline]
    pub(crate) fn next_key(&mut self) {
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
    pub(crate) fn next_line(&mut self) {
        self.next_key();
        self.x = 0.;
        self.y += 1.;
    }

    pub(crate) fn build_key(&self, legends: &str) -> Result<Key> {
        let legends = izip!(legends.lines(), self.fa, self.ta).map(|(text, size, color)| {
            (!text.is_empty()).then_some(Legend {
                text: text.into(),
                size,
                color,
            })
        });
        let legends = realign_legends(legends, self.a)?;

        Ok(Key {
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
            rotation: 0., // TODO: self.r
            rx: 0.,       // TODO: self.rx
            ry: 0.,       // TODO: self.ry
            profile: self.p.clone(),
            ghosted: false,            // TODO: self.g
            switch: Switch::default(), // TODO: self.sm
            stepped: self.l,
            homing: self.n,
            decal: self.d,
        })
    }
}
