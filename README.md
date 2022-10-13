# icfs

Internet Computer File System

![](https://img.shields.io/badge/status%EF%B8%8F-experimental-blueviolet)

## Crates

* `icfs` provides implementations of `std::io::{Read, Write, Seek}` backed by stable memory to enable the use of existing Rust code that requires implementations of these traits.
* `icfs-fatfs` uses `icfs` to leverage the `fatfs` crate in providing a FAT file system.

## Develop

`nix develop`

## Build

`nix build` or `nix build '.#package-name'`
