use gpui::{Hsla, Rgba};

pub trait WithAlpha {
    /// Set the alpha (opacity) of this color.
    fn with_alpha(&self, alpha: f32) -> Self;
}

impl WithAlpha for Hsla {
    fn with_alpha(&self, alpha: f32) -> Self {
        Hsla {
            h: self.h,
            s: self.s,
            l: self.l,
            a: alpha,
        }
    }
}

impl WithAlpha for Rgba {
    fn with_alpha(&self, alpha: f32) -> Self {
        Rgba {
            r: self.r,
            g: self.g,
            b: self.b,
            a: alpha,
        }
    }
}
