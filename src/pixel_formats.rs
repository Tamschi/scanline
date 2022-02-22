use crate::PixelFormat;

/// Used for garden-variety transparent and, in some cases, solid images.
pub enum RgbaNoPadding<const BIT_DEPTH: usize> {}
impl<const BIT_DEPTH: usize> PixelFormat for RgbaNoPadding<BIT_DEPTH> {
	const PIXEL_STRIDE_BITS: usize = 4 * BIT_DEPTH;
}

pub enum RgbNoPadding<const BIT_DEPTH: usize> {}
impl<const BIT_DEPTH: usize> PixelFormat for RgbNoPadding<BIT_DEPTH> {
	const PIXEL_STRIDE_BITS: usize = 3 * BIT_DEPTH;
}
