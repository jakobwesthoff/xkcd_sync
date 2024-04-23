# xkcd sync

This repository contains a little rust program to always retrieve the newest
[xkcd](http://xkcd.com) comics and stores them in conjunction with their
metadata.Each time you execute the `xkcd_sync` command only the new or not yet
downloaded images and informations will be downloaded.

## Building

Simply use `cargo build --release` to create a release build.

## Running

Execute the built application at `target/release/xkcd_sync` or use `cargo run --release`

## Storage paths

At the moment the sync state will always be read and stored within the current
working directory as `xkcd_sync_state.json`.

Comics will always be put into a subfolder called `comics` in the current
working directory as well.
