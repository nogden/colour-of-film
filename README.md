# Colour of Film

Create colour profiles of any film.

![Example Profile](example.png)

## How it works

A colour profile is constructed by taking each frame in the film,
determining the average colour of that frame and painting a 1 pixel
wide stripe into the output image.  The result is an image equal in
width to the number of frames in the video* that shows a profile of
the average colour of the video over time.

*Due to the large number of frames in most video, colour-of-film
defaults to processing key frames only, this can be overridden with
the `--all-frames` flag.
