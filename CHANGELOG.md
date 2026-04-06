# 2.1.0

The headline feature of this release is the ability to locate _all_ detected
Steam installations instead of only returning the first one with the addition of
`steamlocate::locate_all()`

## Features

- Allow returning all detected Steam installations with `locate_all()` [#64]
- Add debian steam installation path [#94]
- Move locate functions to crate root [#107]

## Fix

- Parse shortcut fields regardless of order [#111]

## Dependenciesa

- Update GitHub actions [#100] [#101]
- Update `winreg` 0.55.0 -> 0.56.0 [#108]
- Update `Cargo.lock` deps [#110]

## Internal

- Placate clippy [#95] [#98]
- Depend on `serde_derive` and `serde` separately [#96]
- Separate locate code by platform [#99]
- Replace `home` crate with `std::env::home_dir` [#106]
- Add CI workflow for publishing releases [`08901a8`]
- Group all test assets together [#109]

[#64]: https://github.com/WilliamVenner/steamlocate-rs/pull/64
[#94]: https://github.com/WilliamVenner/steamlocate-rs/pull/94
[#95]: https://github.com/WilliamVenner/steamlocate-rs/pull/95
[#96]: https://github.com/WilliamVenner/steamlocate-rs/pull/96
[#98]: https://github.com/WilliamVenner/steamlocate-rs/pull/98
[#99]: https://github.com/WilliamVenner/steamlocate-rs/pull/99
[#100]: https://github.com/WilliamVenner/steamlocate-rs/pull/100
[#101]: https://github.com/WilliamVenner/steamlocate-rs/pull/101
[#106]: https://github.com/WilliamVenner/steamlocate-rs/pull/106
[#107]: https://github.com/WilliamVenner/steamlocate-rs/pull/107
[#108]: https://github.com/WilliamVenner/steamlocate-rs/pull/108
[#109]: https://github.com/WilliamVenner/steamlocate-rs/pull/109
[#110]: https://github.com/WilliamVenner/steamlocate-rs/pull/110
[#111]: https://github.com/WilliamVenner/steamlocate-rs/pull/111
[`08901a8`]: https://github.com/WilliamVenner/steamlocate-rs/commit/08901a8afe3267faa61b9811db1b1acd3594428b

# 2.0.1

Just a small release to keep things up to date

## Documentation

- Add a changelog #89

## Dependencies

- Update `winreg` from 0.52 -> 0.55 #86

## Internal

- Make tests into into integration tests where possible #87
- Remove publish workflow #88
- Add a release checklist #90

# 2.0.0

Finally after a _very_ long development period we're release version 2.0.0. Living up to the major version bump this release does involve breaking changes to most parts of the API. The majority of these changes fit into three core
themes:

1. `Iterator`ification of the _list all_ flavors of methods
2. Exhaustively parsing `App`s (previously `SteamApp`s) to drop the public dependency on `steamy-vdf`
3. Actually defining an `Error` type instead of returning ambiguous `None`s

Let's dive right in

## `Iterator`ification of the _list all_ methods

Methods that would previously exhaustively collect some set of information and cache it like `SteamDir::libraryfolders()` and `SteamDir::apps()` now return iterators that walk over the set of information and returns values on the fly akin to APIs like `std::fs::read_dir()`. This has a couple of distinct advantages where we can return precise errors for each item ergonomically, and we can be lazier with our computation

## Exhaustive `App`s

We're trying to be a stable library since our major version is >0, but unfortunately there's not a stable [VDF](https://developer.valvesoftware.com/wiki/KeyValues) parser in sight. That's a bit problematic as we'll want to avoid relying on one in our public API, but that also means significant changes to how `App` would hold a `steamy_vdf::Table` representing the parsed appmanifest file. To mitigate this we attempt to exhaustively parse and provide as much data as we can from steam apps, and to top it off we also annotated it with `#[non_exhaustive]`, so that more fields can be added in the future without a breaking change

## An `Error` appears!

This is a _significant_ improvement over the old API. Previously errors would be bubbled up as `None`s that would lead to ambiguity over whether a `None` is from something not existing or simply failing to be parsed. We now religiously return errors to represent failure cases leaving it up to the consumer to decide whether to ignore the error or fail loudly

Where possible we try to include relevant information for the error, but several of the underlying types are intentionally opaque to avoid exposing unstable depdencies in our public API
