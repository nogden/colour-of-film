use std::{error::Error, ffi::OsString};

use clap::Clap;
use colour_of_film::{stripe_image, Mode};

/// Create a colour profile of a video by taking the average colour of
/// each frame
#[derive(Clap)]
#[clap(version = "1.0", author = "Nick Ogden <nick@nickogden.org>")]
struct Options {
    /// Path to the input video file
    #[clap(index(1))]
    input_file: OsString,

    /// Path to write the output image to (must be either .png or .jpeg)
    #[clap(short, long, default_value = "output.png")]
    output_file: OsString,

    /// Use every frame of the input video instead of just key frames
    /// (very slow for long videos)
    #[clap(short, long)]
    all_frames: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = Options::parse();
    let mode = if options.all_frames {
        Mode::AllFrames
    } else {
        Mode::KeyFramesOnly
    };
    let image = stripe_image(options.input_file, 100, mode)?;
    image.save(options.output_file)?;
    Ok(())
}
