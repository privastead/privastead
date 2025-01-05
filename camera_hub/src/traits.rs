//! Privastead camera traits.
//!
//! Copyright (C) 2024  Ardalan Amiri Sani
//!
//! This program is free software: you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation, either version 3 of the License, or
//! (at your option) any later version.
//!
//! This program is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//! GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License
//! along with this program.  If not, see <https://www.gnu.org/licenses/>.

use anyhow::Error;
use bytes::BytesMut;

pub trait CodecParameters {
    fn write_codec_box(&self, buf: &mut BytesMut);
    fn get_clock_rate(&self) -> u32;
    fn get_dimensions(&self) -> (u32, u32);
}

pub trait Mp4 {
    async fn video(
        &mut self,
        frame: &[u8],
        frame_timestamp: u64,
        is_random_access_point: bool,
    ) -> Result<(), Error>;
    async fn audio(&mut self, frame: &[u8], frame_timestamp: u64) -> Result<(), Error>;
    async fn finish_fragment(&mut self) -> Result<(), Error>;
}