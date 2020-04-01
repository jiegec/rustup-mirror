rustup-mirror
=====================================

[![Crates.io version][crate-img]][crate]
[![Changelog][changelog-img]][changelog]
[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2Fjiegec%2Frustup-mirror.svg?type=shield)](https://app.fossa.io/projects/git%2Bgithub.com%2Fjiegec%2Frustup-mirror?ref=badge_shield)

Setup a local rustup mirror. For usage, please run `rustup-mirror -h`.

How to install
=====================================

Run `cargo install rustup-mirror`.

Features
===================================

1. Check if file is already downloaded and check its integrity by comparing sha256 checksum.
2. Download and replace links in the manifest files.

Example usage
=====================================

```shell
$ rustup-mirror # use HTTPS_PROXY for proxy
$ # wait for downloading
$ cd ./mirror # default directory, see rustup-mirror -h
$ python3 -m http.server &
$ RUSTUP_DIST_SERVER=http://127.0.0.1:8000 rustup install stable
```

Note:

1. A full clone of a stable distribution takes 16G disk space (as of Feb 2019).
2. Python3 http.server module does not support Range download. It may fail when a partial downloaded file exists. Do not use this in production.

[crate-img]:     https://img.shields.io/crates/v/rustup-mirror.svg
[crate]:         https://crates.io/crates/rustup-mirror
[changelog-img]: https://img.shields.io/badge/changelog-online-blue.svg
[changelog]:     https://github.com/jiegec/rustup-mirror/blob/master/CHANGELOG.md


## License
[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2Fjiegec%2Frustup-mirror.svg?type=large)](https://app.fossa.io/projects/git%2Bgithub.com%2Fjiegec%2Frustup-mirror?ref=badge_large)