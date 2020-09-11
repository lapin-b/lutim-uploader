# "Lutim uploader"

(Don't judge me, naming things is hard !)

This little program uploads an image to Lutim whenever a new file has appeared in a configured directory, thanks to a
 Linux kernel feature: inotify.
 
## Context

As an user of `xfce4-screenshoter` (insert your favorite one here), I felt like that something was missing:

- You *can* upload to Imgur, but I didn't wan to;
- You can't copy the fresh image to your clipboard unless you open the image with GIMP and copy it from there

With those frustrations, I've hacked together a Python script which works wonders but with hardcoded values. Then, I
 wrote this little program in Rust to practice the language.

## Compatibility

Since this program uses inotify, you don't have high flying chances to be able to execute it on Windows. Unless you
 manage to make it work in WSL, but that's another story. 