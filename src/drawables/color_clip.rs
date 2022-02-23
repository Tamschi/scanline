use std::{marker::PhantomData, ops::Range};

use tap::{Conv, TryConv};

use crate::{pixel_formats::RgbaNoPadding, Effect, PixelFormat, Sprite};

/// A flat-coloured dynamically masked sprite.
pub struct ColorClip<
	P: PixelFormat,
	L: Fn(Option<Range<isize>>) -> Range<isize>,
	S: Fn(Option<Range<isize>>, isize, Range<isize>) -> Range<isize>,
	C,
> {
	lines: L,
	segments: S,
	color: C,
	_phantom: PhantomData<P>,
}

impl<
		P: PixelFormat,
		L: Fn(Option<Range<isize>>) -> Range<isize>,
		S: Fn(Option<Range<isize>>, isize, Range<isize>) -> Range<isize>,
		C,
	> ColorClip<P, L, S, C>
{
	/// Creates a new [`ColorMask`] instance.
	pub fn new(lines: L, segments: S, color: C) -> Self {
		Self {
			lines,
			segments,
			color,
			_phantom: PhantomData,
		}
	}
}

impl<
		L: Fn(Option<Range<isize>>) -> Range<isize>,
		S: Fn(Option<Range<isize>>, isize, Range<isize>) -> Range<isize>,
	> Sprite<RgbaNoPadding<8>> for ColorClip<RgbaNoPadding<8>, L, S, [u8; 4]>
{
	fn lines(&self, all_lines_range: Option<Range<isize>>) -> Range<isize> {
		(self.lines)(all_lines_range)
	}

	fn line_segment(
		&self,
		all_lines_range: Option<Range<isize>>,
		line: isize,
		line_span: Range<isize>,
	) -> Range<isize> {
		(self.segments)(all_lines_range, line, line_span)
	}

	fn render(
		&self,
		_all_lines_range: Option<Range<isize>>,
		_line: isize,
		_line_span: Range<isize>,
		_segment: Range<isize>,
		offset_bits: usize,
		data: &mut [u8],
	) {
		assert_eq!(offset_bits, 0);

		for dest in data.chunks_exact_mut(4) {
			let dest_alpha = dest[3];

			for (src, dest) in self.color.iter().zip(dest) {
				*dest = (*dest).saturating_add(
					((*src).conv::<u16>() * (u8::MAX - dest_alpha).conv::<u16>()
						/ u8::MAX.conv::<u16>())
					.try_conv::<u8>()
					.expect("infallible"),
				);
			}
		}
	}
}

impl<
		L: Fn(Option<Range<isize>>) -> Range<isize>,
		S: Fn(Option<Range<isize>>, isize, Range<isize>) -> Range<isize>,
	> Effect<RgbaNoPadding<8>> for ColorClip<RgbaNoPadding<8>, L, S, [u8; 4]>
{
	fn lines(&self, all_lines_range: Option<Range<isize>>) -> Range<isize> {
		(self.lines)(all_lines_range)
	}

	fn line_segment(
		&self,
		all_lines_range: Option<Range<isize>>,
		line: isize,
		line_span: Range<isize>,
	) -> Range<isize> {
		(self.segments)(all_lines_range, line, line_span)
	}

	fn render(
		&self,
		_all_lines_range: Option<Range<isize>>,
		_line: isize,
		_line_span: Range<isize>,
		_segment: Range<isize>,
		offset_bits: usize,
		data: &mut [u8],
	) {
		assert_eq!(offset_bits, 0);

		for dest in data.chunks_exact_mut(4) {
			let src_alpha = self.color[3];

			for (src, dest) in self.color.iter().zip(dest) {
				*dest = src.saturating_add(
					((*dest).conv::<u16>() * (u8::MAX - src_alpha).conv::<u16>()
						/ u8::MAX.conv::<u16>())
					.try_conv::<u8>()
					.expect("infallible"),
				);
			}
		}
	}
}
