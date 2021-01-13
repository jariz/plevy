# Plevy

A virtual media library.  
**Stream torrent content in realtime from Plex à la Popcorn Time**, thanks to Bevy and dark file system magic.  
This is mostly just a proof of concept, but could be the start of something interesting.

## Requirements:
- UNIX 
  MacOS/Linux.  
  No windows for the foreseeable future, sorry!
- Bevy
- Plex Media Server (duh)
- FUSE  
  - Comes preinstalled with most linux kernels.  
  - MacOS users need to install [OSXFUSE](https://osxfuse.github.io/).
    ```bash
    homebrew install --cask osxfuse
    ```

## TODO

- [x] Add entries w/API
- [x] List entries w/API & filesystem
- [ ] Stream/read files
- [ ] metadata:
   - [ ] fanart.jpg
   - [ ] poster.jpg
   - [ ] FILENAME.nfo
- [ ] Some kind of frontend
  

## Setup/howto

- Copy `config-example.yml` to `config.yml`.  
  Fill in your bevy/plex url's and optionally the path where you'd like your media to appear.
- Start Plevy's backend.
  ```bash
  cargo run
  ```
  Currently, no binaries are provided, so you will need a rust toolchain.
- Add your media folder to Plex
- Start the (modified) Bevy frontend in `frontend/`
- Choose a movie
- Select 'add to plex'
- The movie will now appear in your Plex library