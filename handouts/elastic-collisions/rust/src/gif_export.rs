use crate::error::Error;
use crate::render::{Frame, PALETTE};
use std::borrow::Borrow;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Frame delay in hundredths of a second (25 fps).
pub const FRAME_DELAY_CS: u16 = 4;

/// Encodes `frames` as an infinitely looping GIF using the shared palette.
pub fn write_gif<W, I>(writer: W, width: u16, height: u16, frames: I) -> Result<(), Error>
where
    W: Write,
    I: IntoIterator,
    I::Item: Borrow<Frame>,
{
    let mut frames = frames.into_iter();
    let first = frames.next().ok_or(Error::NoFrames)?;

    let palette: Vec<u8> = PALETTE.iter().flatten().copied().collect();
    let mut encoder = gif::Encoder::new(writer, width, height, &palette)?;
    let written = encode_frames(
        &mut encoder,
        width,
        height,
        std::iter::once(first).chain(frames),
    );
    // `into_inner` writes the trailer and disarms `Encoder`'s panicking Drop.
    match written {
        Ok(()) => encoder.into_inner()?.flush()?,
        Err(e) => {
            let _ = encoder.into_inner();
            return Err(e);
        }
    }
    Ok(())
}

fn encode_frames<W, I>(
    encoder: &mut gif::Encoder<W>,
    width: u16,
    height: u16,
    frames: I,
) -> Result<(), Error>
where
    W: Write,
    I: Iterator,
    I::Item: Borrow<Frame>,
{
    encoder.set_repeat(gif::Repeat::Infinite)?;
    for frame in frames {
        let frame: &Frame = frame.borrow();
        if frame.width != width
            || frame.height != height
            || frame.pixels.len() != width as usize * height as usize
        {
            return Err(Error::FrameSizeMismatch(format!(
                "expected {width}x{height}, got {}x{} with {} pixels",
                frame.width,
                frame.height,
                frame.pixels.len()
            )));
        }
        let mut out = gif::Frame::from_indexed_pixels(width, height, frame.pixels.as_slice(), None);
        out.delay = FRAME_DELAY_CS;
        encoder.write_frame(&out)?;
    }
    Ok(())
}

/// Writes the GIF to `path`.
pub fn save_gif<P, I>(path: P, width: u16, height: u16, frames: I) -> Result<(), Error>
where
    P: AsRef<Path>,
    I: IntoIterator,
    I::Item: Borrow<Frame>,
{
    let file = std::fs::File::create(path)?;
    write_gif(BufWriter::new(file), width, height, frames)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::PALETTE;

    fn frame(w: u16, h: u16, fill: u8) -> Frame {
        Frame {
            width: w,
            height: h,
            pixels: vec![fill; w as usize * h as usize],
        }
    }

    type DecodedFrames = Vec<(u16, Vec<u8>)>;

    fn decode(bytes: &[u8]) -> (gif::Decoder<&[u8]>, DecodedFrames) {
        let mut opts = gif::DecodeOptions::new();
        opts.set_color_output(gif::ColorOutput::Indexed);
        let mut dec = opts.read_info(bytes).unwrap();
        let mut frames = Vec::new();
        while let Some(f) = dec.read_next_frame().unwrap() {
            frames.push((f.delay, f.buffer.to_vec()));
        }
        (dec, frames)
    }

    #[test]
    fn writes_size_frames_delay_loop_and_palette() {
        let mut out = Vec::new();
        let frames = vec![frame(8, 6, 1), frame(8, 6, 2), frame(8, 6, 0)];
        write_gif(&mut out, 8, 6, &frames).unwrap();

        let (dec, decoded) = decode(&out);
        assert_eq!((dec.width(), dec.height()), (8, 6));
        assert!(matches!(dec.repeat(), gif::Repeat::Infinite));
        let expected: Vec<u8> = PALETTE.iter().flatten().copied().collect();
        assert_eq!(
            dec.global_palette().unwrap()[..expected.len()],
            expected[..]
        );
        assert_eq!(decoded.len(), 3);
        for (i, (delay, buf)) in decoded.iter().enumerate() {
            assert_eq!(*delay, 4);
            assert_eq!(buf, &frames[i].pixels);
        }
    }

    #[test]
    fn accepts_a_lazy_iterator_of_owned_frames() {
        let mut out = Vec::new();
        write_gif(&mut out, 4, 4, (0..5).map(|i| frame(4, 4, i))).unwrap();
        assert_eq!(decode(&out).1.len(), 5);
    }

    #[test]
    fn rejects_no_frames() {
        let mut out = Vec::new();
        let r = write_gif(&mut out, 4, 4, Vec::<Frame>::new());
        assert!(matches!(r, Err(Error::NoFrames)));
    }

    #[test]
    fn rejects_size_mismatch() {
        let mut out = Vec::new();
        let r = write_gif(&mut out, 4, 4, vec![frame(4, 4, 0), frame(5, 4, 0)]);
        assert!(matches!(r, Err(Error::FrameSizeMismatch(_))));
    }

    #[test]
    fn rejects_frame_with_wrong_pixel_count() {
        let mut out = Vec::new();
        let bad = Frame {
            width: 4,
            height: 4,
            pixels: vec![0; 3],
        };
        let r = write_gif(&mut out, 4, 4, vec![bad]);
        assert!(matches!(r, Err(Error::FrameSizeMismatch(_))));
    }

    #[test]
    fn save_gif_reports_io_error_for_unwritable_path() {
        let r = save_gif(
            "/nonexistent-dir-for-test/x.gif",
            4,
            4,
            vec![frame(4, 4, 0)],
        );
        assert!(matches!(r, Err(Error::Io(_))));
    }

    #[test]
    fn save_gif_writes_a_readable_file() {
        let path = std::env::temp_dir().join(format!(
            "elastic-collisions-test-{}.gif",
            std::process::id()
        ));
        save_gif(&path, 4, 4, vec![frame(4, 4, 3)]).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(decode(&bytes).1.len(), 1);
    }
}
