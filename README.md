# medialoop

Medialoop is a looping player for exhibitions, galleries or home automation.
You can schedule and optionally loop media files, executables or browser-based software.

## Dependencies
- ffmpeg
- chromium
- open-jdk ("default-jdk")

## Setup

This software was originally designed for Puppy linux. Puppy linux is unusual for a linux desktop distribution, as it allows the default user to be root. For a Puppy linux installation, all that you would need to do would be to install the dependencies and then manually install `medialoop` and `medialoop_init` to `/usr/sbin/`. Finally, you would need to create a startup job or script that automatically runs `medialoop_init` on startup.

### Other linux distributions

It is possible to run this software on other linux distributions, but be aware that it was not designed to work on devices other than those provided by Considerate Digital and is not tested in other environments.

To set `medialoop` up on most linux distributions you have to give a user `mount` and `umount` permissions. *This is a security risk, as the ability to mount filesystems should be restricted and can have adverse consequences.* We would suggest that you only do this for systems used expressly for the purpose of looping media, executables or browser-based software, and not on your day-to-day or "work" machine.

To give the user permissons:
``` 
sudo visudo /etc/sudoers
```

Then add the following, substituting your username:
```
username    (ALL)=ALL   /usr/bin/mount, /usr/bin/umount
```
These instructions may vary for separate distributions.
