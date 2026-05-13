# m3u-creator

A small desktop app for creating M3U playlists from multi-disk game images.

[Download latest release](https://github.com/RogueBurger/m3u-creator/releases/latest)

Drop a set of disk files onto the window and it will:
- Figure out the game name from the filenames
- Create a `GameName.m3u` folder next to the files
- Move the files in
- Write the playlist

## Dev

- [Node.js](https://nodejs.org)
- [Rust](https://rustup.rs)

```
npm install
npm run tauri dev
```
