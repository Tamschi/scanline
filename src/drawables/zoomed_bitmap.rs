use crate::{pixel_formats::RgbaNoPadding, PixelFormat, PostEffect, Sprite};
use std::{convert::TryInto, iter, marker::PhantomData, ops::Range};
use tap::{Conv, Pipe, TryConv};

/// An integer-zoomed bitmap sprite.
pub struct ZoomedBitmap<'a, P: PixelFormat> {
	width: usize,
	data: &'a [u8],
	zoom_factor: usize,
	_phantom: PhantomData<P>,
}
impl Sprite<RgbaNoPadding<8>> for ZoomedBitmap<'_, RgbaNoPadding<8>> {
	fn lines(&self) -> Range<isize> {
		0..(self.data.len() / 4 / self.width * self.zoom_factor)
			.try_into()
			.expect("`isize` too small to represent sprite height")
	}

	fn line_segment(&self, _line: usize, _line_span: Range<isize>) -> Range<isize> {
		0..(self.width * self.zoom_factor)
			.try_into()
			.expect("`isize` too small to represent sprite width")
	}

	fn render(
		&self,
		line: isize,
		_line_span: Range<isize>,
		segment: Range<isize>,
		offset_bits: usize,
		data: &mut [u8],
	) {
		const PIXEL_BYTES: usize = RgbaNoPadding::<8>::PIXEL_STRIDE_BITS / 8;

		assert!(line >= 0);
		let line: usize = line.try_into().expect("infallible");
		assert!(line < self.data.len() / PIXEL_BYTES / self.width * self.zoom_factor);
		assert!(offset_bits % 8 == 0);
		assert!(segment.start >= 0);
		assert!(segment.start <= segment.end);
		let segment: Range<usize> = segment.start.try_into().expect("infallible")
			..segment.end.try_into().expect("infallible");
		assert!(
			segment.end.try_conv::<usize>().expect("infallible") <= self.width * self.zoom_factor
		);
		assert_eq!(segment.len() * PIXEL_BYTES, data.len());

		for (src, dest) in self
			.data
			.chunks_exact(self.width * PIXEL_BYTES)
			.pipe(|lines| repeat_each(lines, self.zoom_factor))
			.skip(line)
			.flat_map(|line| line.chunks_exact(PIXEL_BYTES))
			.pipe(|pixels| repeat_each(pixels, self.zoom_factor))
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

impl PostEffect<RgbaNoPadding<8>> for ZoomedBitmap<'_, RgbaNoPadding<8>> {
	fn lines(&self) -> Range<isize> {
		0..(self.data.len() / 4 / self.width)
			.try_into()
			.expect("`isize` too small to represent sprite height")
	}

	fn line_segment(&self, _line: usize, _line_span: Range<isize>) -> Range<isize> {
		0..self
			.width
			.try_into()
			.expect("`isize` too small to represent sprite width")
	}

	fn render(
		&self,
		line: isize,
		_line_span: Range<isize>,
		segment: Range<isize>,
		offset_bits: usize,
		data: &mut [u8],
	) {
		const PIXEL_BYTES: usize = RgbaNoPadding::<8>::PIXEL_STRIDE_BITS / 8;

		assert!(line >= 0);
		let line: usize = line.try_into().expect("infallible");
		assert!(line < self.data.len() / PIXEL_BYTES / self.width);
		assert!(offset_bits % 8 == 0);
		assert!(segment.start >= 0);
		assert!(segment.start <= segment.end);
		let segment: Range<usize> = segment.start.try_into().expect("infallible")
			..segment.end.try_into().expect("infallible");
		assert!(segment.end.try_conv::<usize>().expect("infallible") <= self.width);
		assert_eq!(segment.len() * PIXEL_BYTES, data.len());

		for (src, dest) in self
			.data
			.chunks_exact(self.width * PIXEL_BYTES)
			.pipe(|lines| repeat_each(lines, self.zoom_factor))
			.skip(line)
			.flat_map(|line| line.chunks_exact(PIXEL_BYTES))
			.pipe(|pixels| repeat_each(pixels, self.zoom_factor))
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
