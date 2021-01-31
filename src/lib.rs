use ffav::{
    format::{self, Pixel},
    media,
    software::{self, scaling::flag::Flags},
    util::frame::video::Video,
};
use image::{RgbImage, Rgb};
use std::{error::Error, path::Path, convert::TryInto, io::Write};

pub fn stripe_image<P: AsRef<Path>>(
    path: P, image_height: u32,
) -> Result<RgbImage, Box<dyn Error>> {
    ffav::init()?;

    let mut video_file = format::input(&path)?;
    let video_stream = video_file.streams()
        .best(media::Type::Video)
        .ok_or("Couldn't find video stream".to_owned())?;
    let video_stream_index = video_stream.index();
    let frame_count: u32 = video_stream.frames().try_into()?;
    let mut decoder = video_stream.codec().decoder().video()?;
    let mut scale_and_convert = software::scaling::Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::RGB24,
        1,
        1,
        Flags::BILINEAR,
    )?;
    let mut frame_index = 0;
    let mut frame = Video::empty();
    let mut image = RgbImage::new(frame_count, image_height);

    print!("0%");
    std::io::stdout().flush()?;
    for (stream, packet) in video_file.packets() {
        if stream.index() != video_stream_index {
            continue;
        }
        // Shove in the packet
        if decoder.decode(&packet, &mut frame)? {
            // A frame popped out; scale it
            let mut scaled_rgb_frame = Video::empty();
            scale_and_convert.run(&frame, &mut scaled_rgb_frame)?;
            let bytes = scaled_rgb_frame.data(0);
            let (r, g, b) = unsafe {(
                bytes.get_unchecked(0),
                bytes.get_unchecked(1),
                bytes.get_unchecked(2),
            )};
            for line in 0..image_height {
                image.put_pixel(frame_index, line, Rgb([*r, *g, *b]));
            }
            frame_index += 1;
            if frame_index % 500 == 0 {
                print!(
                    " => {}%",
                    ((frame_index as f32 / frame_count as f32) * 100.) as u32
                );
                std::io::stdout().flush()?;
                // if frame_index == 2000 {
                //     break
                // }
            }
        }
    }

    Ok(image)
}
