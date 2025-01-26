## 2.0.0

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
