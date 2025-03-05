# medialoop

Medialoop is a looping player for exhibitions, galleries or home automation.
You can schedule and optionally loop media files, executables or browser-based software.

## Dependencies
- ffmpeg
- chromium
- open-jdk ("default-jdk")

## Setup

This software was originally designed for FunOS (Ubuntu-based). FunOS (Ubuntu-based) is unusual for a linux desktop distribution, as it allows the default user to be root. For a FunOS (Ubuntu-based) installation, all that you would need to do would be to install the dependencies and then manually install `medialoop` and `medialoop_init` to `/usr/sbin/`. Finally, you would need to create a startup job or script that automatically runs `medialoop_init` on startup.

### Other linux distributions

It is possible to run this software on other linux distributions, but be aware that it was not designed to work on devices other than those provided by Considerate Digital and is not tested in other environments.

## Future Plans
- Add automatic addition of systemd service for medialoop_init if none exists
