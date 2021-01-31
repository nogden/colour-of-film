use std::{error::Error, env};

use colour_of_film::{stripe_image, Mode};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os();
    let _ = args.next(); // Skip executable name
    if let Some(path) = args.next() {
        let image = stripe_image(path, 100, Mode::KeyFramesOnly)?;
        image.save("output.png")?;
        Ok(())
    } else {
        Err("A film file must be provided".into())
    }
}
