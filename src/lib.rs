//! A simple software scanline renderer.
//! This is helpful for example for streaming [PNG](https://crates.io/crates/png) output line by line.
//!
//! [![Zulip Chat](https://img.shields.io/endpoint?label=chat&url=https%3A%2F%2Fiteration-square-automation.schichler.dev%2F.netlify%2Ffunctions%2Fstream_subscribers_shield%3Fstream%3Dproject%252Fscanline)](https://iteration-square.schichler.dev/#narrow/stream/project.2Fscanline)
//!
//!

#![doc(html_root_url = "https://docs.rs/scanline/0.0.1")]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::semicolon_if_nothing_returned)]

use std::ops::Range;

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

pub fn render_line<'a, P: PixelFormat, S: 'a + Sprite<P>, E: 'a + Effect<P>>(
	all_lines_range: Option<Range<isize>>,
	line_index: isize,
	buffer: &mut [u8],
	sprites: impl IntoIterator<Item = &'a S>,
	post_effects: impl IntoIterator<Item = &'a E>,
) {
	render_segment(
		all_lines_range,
		line_index,
		0,
		buffer,
		sprites,
		post_effects,
	)
}

pub fn render_segment<'a, P: PixelFormat, S: 'a + Sprite<P>, E: 'a + Effect<P>>(
	all_lines_range: Option<Range<isize>>,
	line_index: isize,
	segment_offset: isize,
	buffer: &mut [u8],
	sprites: impl IntoIterator<Item = &'a S>,
	post_effects: impl IntoIterator<Item = &'a E>,
) {
	todo!()
}
