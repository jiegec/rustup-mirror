v0.11.0 2026-08-20
========================

1. `--keep` now also prunes unkept compression formats from historical nightly date directories (not just the currently-synced date), so e.g. keep-xz mode cleans up old `.tar.gz` files too.
2. Update the supported `TARGETS` list from `rustc --print target-list` (rustc 1.97.1, 322 targets).

v0.10.0 2026-08-20
========================

> **Breaking change**: on a default run, `.gz` archives that were
> mirrored previously are removed and the served manifest no longer lists them.
> Pass `--keep gz,xz` to restore the old behavior.

1. Add `--keep` option to choose which archive compression format(s) to mirror (e.g. `--keep xz` or `--keep gz`; default is `xz` only). Unkept formats are removed from the served manifest and their already-mirrored files are deleted by url, saving disk space. rustup already prefers `zst > xz > gz`, so functionality is unchanged.

v0.9.0 2025-03-17
========================

1. Migrate from clap v3 to v4

v0.8.1 2024-03-12
========================

1. Do not download "*" target for rustup

v0.8.0 2024-03-12
========================

1. Download latest version of rustup-init binary for bootstrapping

v0.7.2 2024-01-06
========================

1. Fix rustup compatibility: do not remove excluded target, but set available to false

v0.7.1 2024-01-06
========================

1. Sync target list from rustc 1.75.0

v0.7.0 2023-12-02
========================

1. Fix empty `rust-src` in channel.toml
2. Fix toml pretty output

v0.6.2 2023-11-11
========================

1. Bump dependencies and fix deprecations.

v0.6.1 2022-06-04
========================

1. Bump dependencies and fix deprecations.

v0.6.0 2021-09-16
========================

1. Add option to specify target architectures, thanks @johnlepikhin

v0.5.0 2021-08-18
========================

1. Add option to specify release channels, thanks @jessebraham

v0.4.4 2021-02-18
========================

1. Fix garbage collect regression.

v0.4.3 2021-01-24
========================

1. Upgrade dependencies.
2. Improve garbage collect logic.

v0.4.2 2020-06-08
========================

1. Upgrade dependencies.

v0.4.1 2020-02-21
========================

1. Correctly skip non-nightly files for --gc.

v0.4.0 2020-02-20
========================

1. Add --gc option to remove old nightly builds.

v0.3.3 2020-02-18
========================

1. HTTPS_PROXY env is respected now.

v0.3.2 2019-10-04
========================

1. SHA256 checksums of local files are saved.

v0.3.1 2019-02-16
========================

1. Beta channel is added.
2. Now support stable and beta toolchain installation with certain date.

v0.3.0 2019-02-15
========================

1. Now support nightly toolchain installation with certain date.

v0.2.2 2019-02-15
========================

1. Fix wrong use of clap library.

v0.2.1 2019-02-14
========================

1. Enable nightly channel mirroring.

v0.1.1 2019-02-12
=========================

1. Add progress bar indicator for downloading.

v0.1.0 2019-02-12
=========================

1. Add initial support for mirroring stable channel.
