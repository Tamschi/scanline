//! Pixel formats.
//!
//! > This is quite sparse, as I only add to it as needed.
//! >
//! > Feel free to [file an issue](https://github.com/Tamschi/scanline/issues) if you need a specific one.

use crate::PixelFormat;

/// Used for garden-variety transparent and, in some cases, solid images.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RgbaNoPadding<const BIT_DEPTH: usize> {}
impl<const BIT_DEPTH: usize> PixelFormat for RgbaNoPadding<BIT_DEPTH> {
	const PIXEL_STRIDE_BITS: usize = 4 * BIT_DEPTH;
}

/// Used for garden-variety solid images.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RgbNoPadding<const BIT_DEPTH: usize> {}
impl<const BIT_DEPTH: usize> PixelFormat for RgbNoPadding<BIT_DEPTH> {
	const PIXEL_STRIDE_BITS: usize = 3 * BIT_DEPTH;
}
