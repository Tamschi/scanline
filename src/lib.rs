//! A simple software scanline renderer.
//! This is helpful for example for streaming [PNG](https://crates.io/crates/png) output line by line.
//!
//! [![Zulip Chat](https://img.shields.io/endpoint?label=chat&url=https%3A%2F%2Fiteration-square-automation.schichler.dev%2F.netlify%2Ffunctions%2Fstream_subscribers_shield%3Fstream%3Dproject%252Fscanline)](https://iteration-square.schichler.dev/#narrow/stream/project.2Fscanline)
//!
//! Coordinates in this crate grow rightwards and downwards, and are in pixels unless otherwise noted.

#![doc(html_root_url = "https://docs.rs/scanline/0.0.1")]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::semicolon_if_nothing_returned)]

use std::{convert::TryInto, ops::Range};
use tap::TryConv;

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

pub mod drawables;
pub mod pixel_formats;

/// Defines a pixel format for the output buffer.
pub trait PixelFormat {
	/// Bits used for each pixel, *including padding*.
	const PIXEL_STRIDE_BITS: usize;
}

/// All coordinates are effect-relative and in pixels.
///
/// [`Effect`]s are drawn back-to-front after sprites into a buffer, with premultiplied alpha (if applicable).
pub trait Effect<P: PixelFormat> {
	/// Gets the applicable line range.
	fn lines(&self, all_lines_range: Option<Range<isize>>) -> Range<isize>;

	/// Gets the line segment affected by a particular line in this effect.
	fn line_segment(
		&self,
		all_lines_range: Option<Range<isize>>,
		line: usize,
		line_span: Range<isize>,
	) -> Range<isize>;

	/// Renders the given segment of the given line. The relevant data is offset `offset_bits` into `data`.
	///
	/// All coordinates are sprite-relative.
	///
	/// `offset_bits` can be relied on to be a multiple of `P::BITS_PER_PIXEL` modulo 8.
	fn render(
		&self,
		all_lines_range: Option<Range<isize>>,
		line: isize,
		line_span: Range<isize>,
		segment: Range<isize>,
		offset_bits: usize,
		data: &mut [u8],
	);
}
impl<T, P: PixelFormat> Effect<P> for &T
where
	T: Effect<P>,
{
	fn lines(&self, all_lines_range: Option<Range<isize>>) -> Range<isize> {
		T::lines(self, all_lines_range)
	}

	fn line_segment(
		&self,
		all_lines_range: Option<Range<isize>>,
		line: usize,
		line_span: Range<isize>,
	) -> Range<isize> {
		T::line_segment(self, all_lines_range, line, line_span)
	}

	fn render(
		&self,
		all_lines_range: Option<Range<isize>>,
		line: isize,
		line_span: Range<isize>,
		segment: Range<isize>,
		offset_bits: usize,
		data: &mut [u8],
	) {
		T::render(
			self,
			all_lines_range,
			line,
			line_span,
			segment,
			offset_bits,
			data,
		)
	}
}

/// All coordinates are sprite-relative and in pixels.
///
/// [`Sprite`]s are drawn front to back into a buffer, with premultiplied alpha (if applicable).
pub trait Sprite<P: PixelFormat> {
	/// Gets the applicable line range.
	fn lines(&self, all_lines_range: Option<Range<isize>>) -> Range<isize>;

	/// Gets the line segment affected by a particular line in this sprite.
	fn line_segment(
		&self,
		all_lines_range: Option<Range<isize>>,
		line: usize,
		line_span: Range<isize>,
	) -> Range<isize>;

	/// Renders the given segment of the given line. The relevant data is offset `offset_bits` into `data`.
	///
	/// `offset_bits` can be relied on to be a multiple of `P::BITS_PER_PIXEL` modulo 8.
	fn render(
		&self,
		all_lines_range: Option<Range<isize>>,
		line: isize,
		line_span: Range<isize>,
		segment: Range<isize>,
		offset_bits: usize,
		data: &mut [u8],
	);
}
impl<T, P: PixelFormat> Sprite<P> for &T
where
	T: Sprite<P>,
{
	fn lines(&self, all_lines_range: Option<Range<isize>>) -> Range<isize> {
		T::lines(self, all_lines_range)
	}

	fn line_segment(
		&self,
		all_lines_range: Option<Range<isize>>,
		line: usize,
		line_span: Range<isize>,
	) -> Range<isize> {
		T::line_segment(self, all_lines_range, line, line_span)
	}

	fn render(
		&self,
		all_lines_range: Option<Range<isize>>,
		line: isize,
		line_span: Range<isize>,
		segment: Range<isize>,
		offset_bits: usize,
		data: &mut [u8],
	) {
		T::render(
			self,
			all_lines_range,
			line,
			line_span,
			segment,
			offset_bits,
			data,
		)
	}
}

/// An offset of a renderable.
pub struct Position {
	/// Rightwards offset, in pixels.
	pub x: isize,
	/// Downwards offset, in pixels.
	pub y: isize,
}

/// Renders an entire line.
///
/// # Panics
///
/// - Iff [`P::PIXEL_STRIDE_BITS`](`PixelFormat::PIXEL_STRIDE_BITS`) isn't a multiple of 8,
/// - and also in cases where [`render_segment`] would panic.
pub fn render_line<P: PixelFormat, S: Sprite<P>, E: Effect<P>>(
	all_lines_range: Option<Range<isize>>,
	line_index: isize,
	buffer: &mut [u8],
	sprites: impl IntoIterator<Item = S>,
	post_effects: impl IntoIterator<Item = E>,
) {
	assert_eq!(P::PIXEL_STRIDE_BITS % 8, 0);

	render_segment(
		all_lines_range,
		line_index,
		0..(buffer.len() / (P::PIXEL_STRIDE_BITS / 8))
			.try_into()
			.expect("`buffer.len() / P::PIXEL_STRIDE_BITS` too large"),
		buffer,
		sprites,
		post_effects,
	)
}

/// Renders a segment of a line.
///
/// # Panics
///
/// Iff coordinates and/or sizes are extreme enough to go out of range.
pub fn render_segment<P: PixelFormat, S: Sprite<P>, E: Effect<P>>(
	all_lines_range: Option<Range<isize>>,
	line_index: isize,
	segment_span: Range<isize>,
	buffer: &mut [u8],
	sprites: impl IntoIterator<Item = S>,
	post_effects: impl IntoIterator<Item = E>,
) {
	let segment_offset_bits = segment_span
		.start
		.try_conv::<i64>()
		.expect("`segment_offset` too extreme")
		.checked_mul(
			P::PIXEL_STRIDE_BITS
				.try_conv::<i64>()
				.expect("`PIXEL_STRIDE_BITS` too large"),
		)
		.expect("segment offset in bits too extreme")
		% 8;

	for sprite in sprites {
		if !sprite.lines(all_lines_range.clone()).contains(&line_index) {
			continue;
		}
	}
}
