use nsh_core::prelude::*;

pub mod spacing {
    use super::*;
    pub fn xs() -> Points { px(4.0) }
    pub fn sm() -> Points { px(8.0) }
    pub fn md() -> Points { px(12.0) }
    pub fn lg() -> Points { px(16.0) }
    pub fn xl() -> Points { px(24.0) }
}

pub mod radius {
    use super::*;
    pub fn sm() -> Points { px(6.0) }
    pub fn md() -> Points { px(8.0) }
    pub fn lg() -> Points { px(12.0) }
    pub fn full() -> Points { px(9999.0) }
}

pub mod status_color {
    use nsh_core::prelude::*;

    pub fn success() -> Rgba {
        rgb(0x10b981) // Emerald 500
    }

    pub fn warning() -> Rgba {
        rgb(0xf59e0b) // Amber 500
    }

    pub fn danger() -> Rgba {
        rgb(0xef4444) // Red 500
    }

    pub fn info() -> Rgba {
        rgb(0x3b82f6) // Blue 500
    }
}
