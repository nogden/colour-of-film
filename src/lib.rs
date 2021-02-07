use ffav::{
    format::{self, Pixel},
    media,
    software::{self, scaling::flag::Flags},
    util::frame::video::Video,
    Packet, Stream,
};
use image::{imageops, RgbImage};
use std::{error::Error, path::Path, convert::TryInto, io::Write, time::Instant};

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    AllFrames,
    KeyFramesOnly,
}

pub fn stripe_image<P: AsRef<Path>>(
    path: P, image_height: u32, mode: Mode
) -> Result<RgbImage, Box<dyn Error>> {
    ffav::init()?;

    let mut video_file = format::input(&path)?;
    let video_stream = video_file.streams()
        .best(media::Type::Video)
        .ok_or("Couldn't find video stream".to_owned())?;
    let video_stream_index = video_stream.index();
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

    let frame_count: u32;
    let mut all_frames_iter;
    let mut key_frames_iter;
    let packets: &mut dyn Iterator<Item=(Stream, Packet)>;
    packets = if mode == Mode::KeyFramesOnly {
        frame_count = video_file.packets().filter(|(stream, packet)| {
            stream.index() == video_stream_index  && packet.is_key()
        }).count() as u32;
        video_file.seek(0, 0..0)?;  // Move back to the start of the input
        key_frames_iter = video_file.packets().filter(|(stream, packet)| {
            stream.index() == video_stream_index && packet.is_key()
        });
        &mut key_frames_iter
    } else {
        frame_count = video_stream.frames().try_into()?;
        all_frames_iter = video_file.packets().filter(|(stream, _)| {
            stream.index() == video_stream_index
        });
        &mut all_frames_iter
    };

    let mut image_buffer = Vec::with_capacity(
        (frame_count * image_height * 3) as usize
    );

    let mut frame_index = 0;
    let mut frame = Video::empty();
    let start_time = Instant::now();
    for (_, packet) in packets {
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
            for _ in 0..image_height {
                image_buffer.extend_from_slice(&[*r, *g, *b]);
            }
            frame_index += 1;
            if frame_index % 200 == 0 {
                let rate = frame_index as f32 / start_time.elapsed().as_secs_f32();
                let eta = (frame_count - frame_index) as f32 / rate;
                print!(
                    "\rProgress: {:>3}%    {:.2}fps    ETA: {}m {}s    ",
                    ((frame_index as f32 / frame_count as f32) * 100.) as u32,
                    rate,
                    (eta / 60.0) as u32,
                    (eta % 60.0) as u32
                );
                std::io::stdout().flush()?;
            }
        }
    }
    println!();
    let image = RgbImage::from_vec(image_height, frame_index, image_buffer)
        .expect("Image buffer wasn't big enough");

    Ok(imageops::rotate90(&image))
}
