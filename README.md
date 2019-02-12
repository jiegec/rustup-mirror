rustup-mirror
=====================================

[![Crates.io version][crate-img]][crate]
[![Changelog][changelog-img]][changelog]

Setup a local rustup mirror. For usage, please run `rustup-mirror -h`.

How to install
=====================================

Run `cargo install rustup-mirror`.

Example usage
=====================================

```shell
$ rustup-mirror
$ # wait for downloading
$ cd ./mirror # default directory, see rustup-mirror -h
$ python3 -m http.server &
$ RUSTUP_DIST_SERVER=http://127.0.0.1:8000 rustup install stable
```

[crate-img]:     https://img.shields.io/crates/v/rustup-mirror.svg
[crate]:         https://crates.io/crates/rustup-mirror
[changelog-img]: https://img.shields.io/badge/changelog-online-blue.svg
[changelog]:     https://github.com/jiegec/rustup-mirror/blob/master/CHANGELOG.md
