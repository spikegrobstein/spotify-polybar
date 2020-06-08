# spotify-polybar

Yet another polybar spotify controller.

Unlike other polybar/spotify integration tools, this utility uses the spotify API rather than DBUS to collect
information. This means that if you're listening on your phone or another device, this will still control
playback and report the correct track name.

Currently, after pausing, you will not be able to resume unless you're playing locally because I'm not passing
a player ID to the API to resume playback.

## Building

Before doing anything, you will need to create an application in spotify and add your client/secret IDs into
the sourcecode. I'll probably make this easier in the future.

Then build:

    cargo build --release

Then copy the executable somewhere useful:

    cp target/release/sptify-polybar ~/.local/bin/

## Usage

See `spotify-polybar --help` for usage.

Basic stuff:

 * `spotify-polybar status` -- output the currently playing track
 * `spotify-polybar next` -- next track
 * `spotify-polybar playpause` -- pause (resume doesn't work right, yet)
 * `spotify-polybar previous` -- previous track

## Polybar example

```ini
[module/spotify]
type = custom/script
exec = "~/.local/bin/spotify-polybar status"
interval = 5

label = "%output%"

format-background = #030
format-padding = 1

```

## Licence

This is MIT licensed. See LICENSE file.
