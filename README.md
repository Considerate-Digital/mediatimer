# mediatimer

Media Timer is a looping player for exhibitions, galleries or home automation.
You can schedule and optionally loop media files, executables or browser-based software.

## Dependencies
- ffmpeg
- chromium
- open-jdk ("default-jdk")
- feh

## Setup
*This software is not designed to be used outwith devices provided with the  AdaptableOS operating system*

For a standard installation:
- Install the dependencies listed above. 
- Compile and manually install `mediatimer` and `mediatimer_init`.
- Create a startup job or script that automatically runs `mediatimer_init` on startup using the init system of your choice.

### Other platforms

It is possible to run this software on other platforms, but be aware that it was not designed to work on devices other than those provided by Considerate Digital and is not tested for other contexts.

