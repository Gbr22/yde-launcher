use egui::Color32;

#[macro_export]
macro_rules! rgb {
    ($hex_str:expr) => {{
        let raw = $hex_str;
        let cleaned = raw.strip_prefix('#').unwrap_or(raw);
        rgb(u32::from_str_radix(cleaned, 16).expect("invalid hex string"))
    }};
}

#[inline]
pub const fn rgb(hex: u32) -> Color32 {
    let r = ((hex >> 16) & 0xFF) as u8;
    let g = ((hex >> 8) & 0xFF) as u8;
    let b = (hex & 0xFF) as u8;

    Color32::from_rgb(r, g, b)
}

pub trait ColorUtil {
    fn with_alpha(self, opacity: impl IntoOpacity) -> Color32;
}

pub trait IntoOpacity {
    fn into_opacity(self) -> u8;
}

impl IntoOpacity for f32 {
    fn into_opacity(self) -> u8 {
        (self * 255.0).round() as u8
    }
}
impl IntoOpacity for f64 {
    fn into_opacity(self) -> u8 {
        (self * 255.0).round() as u8
    }
}

impl ColorUtil for Color32 {
    #[inline]
    fn with_alpha(self, alpha: impl IntoOpacity) -> Color32 {
        Color32::from_rgba_unmultiplied(self.r(), self.g(), self.b(), alpha.into_opacity())
    }
}
