mod json;

use crate::utils::{realign_legends, Alignment};
use crate::{defaults, NUM_LEGENDS};
use crate::{Color, Key, Legend, Switch};

use itertools::izip;
use smart_default::SmartDefault as Default;

pub(crate) use json::{KleKeyboard, KleLegendsOrProps, KlePropsObject};

#[derive(Debug, Default)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct KleProps {
    // Internal fields
    is_first: bool, // r, rx, & ry can only be set in the first properties object of a line

    // Per-key properties
    x: f64,
    y: f64,
    #[default = 1.]
    w: f64,
    #[default = 1.]
    h: f64,
    x2: f64,
    y2: f64,
    #[default = 1.]
    w2: f64,
    #[default = 1.]
    h2: f64,
    l: bool, // stepped
    n: bool, // homing
    d: bool, // decal

    // Persistent properties
    r: f64,
    rx: f64,
    ry: f64,
    g: bool,    // ghosted
    sm: String, // switch mount
    sb: String, // switch brand
    st: String, // switch type
    #[default(defaults::KEY_COLOR)]
    c: Color, // color
    #[default(defaults::LEGEND_COLOR)]
    t: Color, // fallback legend color
    #[default([defaults::LEGEND_COLOR; NUM_LEGENDS])]
    ta: [Color; NUM_LEGENDS], // legend color array
    #[default(defaults::ALIGNMENT)]
    a: Alignment, // alignment
    p: String,  // profile
    #[default(defaults::FONT_SIZE)]
    f: usize, // fallback font size
    #[default([defaults::FONT_SIZE; NUM_LEGENDS])]
    fa: [usize; NUM_LEGENDS], // font size array
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

        // Internal fields
        self.is_first = false;
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
        self.is_first = true;
        self.x = self.rx; // x resets to rx
        self.y += 1.;
    }

    pub(crate) fn build_key(&self, legends: &str) -> Key {
        let legends = izip!(legends.lines(), self.fa, self.ta).map(|(text, size, color)| {
            (!text.is_empty()).then_some(Legend {
                text: text.into(),
                size,
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
