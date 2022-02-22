use crate::{pixel_formats::RgbaNoPadding, Effect, PixelFormat, Sprite};
use std::{convert::TryInto, iter, marker::PhantomData, ops::Range};
use tap::{Conv, Pipe, TryConv};

/// An integer-zoomed bitmap sprite.
pub struct ZoomedBitmap<'a, P: PixelFormat> {
	width: usize,
	data: &'a [u8],
	horizontal_zoom_factor: usize,
	vertical_zoom_factor: usize,
	_phantom: PhantomData<P>,
}
impl<'a> ZoomedBitmap<'a, RgbaNoPadding<8>> {
	/// Creates a new instance of [`ZoomedBitmap`].
	///
	/// # Panics
	///
	/// Iff `data` doesn't represent a whole number of lines of width `width`.
	#[must_use]
	pub fn new(
		width: usize,
		data: &'a [u8],
		horizontal_zoom_factor: usize,
		vertical_zoom_factor: usize,
	) -> Self {
		assert_eq!(
			data.len() % (width * RgbaNoPadding::<8>::PIXEL_STRIDE_BITS * 8),
			0
		);
		Self {
			width,
			data,
			horizontal_zoom_factor,
			vertical_zoom_factor,
			_phantom: PhantomData,
		}
	}
}
impl Sprite<RgbaNoPadding<8>> for ZoomedBitmap<'_, RgbaNoPadding<8>> {
	fn lines(&self, _all_lines_range: Option<Range<isize>>) -> Range<isize> {
		0..(self.data.len() / 4 / self.width * self.vertical_zoom_factor)
			.try_into()
			.expect("`isize` too small to represent sprite height")
	}

	fn line_segment(
		&self,
		_all_lines_range: Option<Range<isize>>,
		_line: usize,
		_line_span: Range<isize>,
	) -> Range<isize> {
		0..(self.width * self.horizontal_zoom_factor)
			.try_into()
			.expect("`isize` too small to represent sprite width")
	}

	fn render(
		&self,
		_all_lines_range: Option<Range<isize>>,
		line: isize,
		_line_span: Range<isize>,
		segment: Range<isize>,
		offset_bits: usize,
		data: &mut [u8],
	) {
		const PIXEL_BYTES: usize = RgbaNoPadding::<8>::PIXEL_STRIDE_BITS / 8;

		assert!(line >= 0);
		let line: usize = line.try_into().expect("infallible");
		assert!(line < self.data.len() / PIXEL_BYTES / self.width * self.vertical_zoom_factor);
		assert_eq!(offset_bits % 8, 0);
		assert!(segment.start >= 0);
		assert!(segment.start <= segment.end);
		let segment: Range<usize> = segment.start.try_into().expect("infallible")
			..segment.end.try_into().expect("infallible");
		assert!(
			segment.end.try_conv::<usize>().expect("infallible")
				<= self.width * self.horizontal_zoom_factor
		);
		assert_eq!(segment.len() * PIXEL_BYTES, data.len());

		for (src, dest) in self
			.data
			.chunks_exact(self.width * PIXEL_BYTES)
			.pipe(|lines| repeat_each(lines, self.vertical_zoom_factor))
			.skip(line)
			.flat_map(|line| line.chunks_exact(PIXEL_BYTES))
			.pipe(|pixels| repeat_each(pixels, self.horizontal_zoom_factor))
			.skip(segment.start)
			.take(segment.len())
			.zip(data.chunks_exact_mut(PIXEL_BYTES))
		{
			let dest_alpha = dest[3];

			for (src, dest) in src.iter().zip(dest) {
				*dest += ((*src).conv::<u16>() * (u8::MAX - dest_alpha).conv::<u16>()
					/ u8::MAX.conv::<u16>())
				.try_conv::<u8>()
				.expect("infallible");
			}
		}
	}
}

impl Effect<RgbaNoPadding<8>> for ZoomedBitmap<'_, RgbaNoPadding<8>> {
	fn lines(&self, _all_lines_range: Option<Range<isize>>) -> Range<isize> {
		0..(self.data.len() / 4 / self.width)
			.try_into()
			.expect("`isize` too small to represent sprite height")
	}

	fn line_segment(
		&self,
		_all_lines_range: Option<Range<isize>>,
		_line: usize,
		_line_span: Range<isize>,
	) -> Range<isize> {
		0..self
			.width
			.try_into()
			.expect("`isize` too small to represent sprite width")
	}

	fn render(
		&self,
		_all_lines_range: Option<Range<isize>>,
		line: isize,
		_line_span: Range<isize>,
		segment: Range<isize>,
		offset_bits: usize,
		data: &mut [u8],
	) {
		const PIXEL_BYTES: usize = RgbaNoPadding::<8>::PIXEL_STRIDE_BITS / 8;

		assert!(line >= 0);
		let line: usize = line.try_into().expect("infallible");
		assert!(line < self.data.len() / PIXEL_BYTES / self.width * self.vertical_zoom_factor);
		assert_eq!(offset_bits % 8, 0);
		assert!(segment.start >= 0);
		assert!(segment.start <= segment.end);
		let segment: Range<usize> = segment.start.try_into().expect("infallible")
			..segment.end.try_into().expect("infallible");
		assert!(
			segment.end.try_conv::<usize>().expect("infallible")
				<= self.width * self.horizontal_zoom_factor
		);
		assert_eq!(segment.len() * PIXEL_BYTES, data.len());

		for (src, dest) in self
			.data
			.chunks_exact(self.width * PIXEL_BYTES)
			.pipe(|lines| repeat_each(lines, self.vertical_zoom_factor))
			.skip(line)
			.flat_map(|line| line.chunks_exact(PIXEL_BYTES))
			.pipe(|pixels| repeat_each(pixels, self.horizontal_zoom_factor))
			.skip(segment.start)
			.take(segment.len())
			.zip(data.chunks_exact_mut(PIXEL_BYTES))
		{
			let src_alpha = src[3];

			for (src, dest) in src.iter().zip(dest) {
				*dest = src
					+ ((*dest).conv::<u16>() * (u8::MAX - src_alpha).conv::<u16>()
						/ u8::MAX.conv::<u16>())
					.try_conv::<u8>()
					.expect("infallible");
			}
		}
	}
}

fn repeat_each<T: Clone>(items: impl IntoIterator<Item = T>, n: usize) -> impl Iterator<Item = T> {
	items
		.into_iter()
		.flat_map(move |item| iter::repeat(item).take(n))
}
