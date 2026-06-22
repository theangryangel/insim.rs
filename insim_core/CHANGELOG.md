# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [4.0.0](https://github.com/theangryangel/insim.rs/compare/insim_core-v3.0.0...insim_core-v4.0.0) - 2026-06-22

### Added

- *(insim_core)* add index function to help identify and provide a unique identifer to objects
- add convenience methods decode_slice, decode_bytes to Decode and encode_bytes to Encode traits
- schemars support & python library backed by insim.rs ([#375](https://github.com/theangryangel/insim.rs/pull/375))
- Allow unknown packets through a feature flag (Fixes #332) ([#336](https://github.com/theangryangel/insim.rs/pull/336))
- *(prefab)* New tools ([#319](https://github.com/theangryangel/insim.rs/pull/319))
- *(prefabs)* curves and stuff
- [**breaking**] bump MSRV, remove if_chain! ([#294](https://github.com/theangryangel/insim.rs/pull/294))
- [**breaking**] insim v10 + Proper Object Handling + Updated PTH and new PIN file handling + other breaking changes ([#266](https://github.com/theangryangel/insim.rs/pull/266))
- *(kitcar)* a series of micro libraries for building mini-games ([#233](https://github.com/theangryangel/insim.rs/pull/233))
- [**breaking**] remove LFSW relay support due to upstream service no longer

### Fixed

- *(insim_core)* re-add into_inner
- fix!(insim,insim_core): Fix incorrectly typed steer, accelf and accelr
- *(insim_core)* Coordinate roundtrip correctness issue
- *(insim_core)* Marshal kind incorrect
- *(insim_core)* InsimCircle incorrect.
- docs.rs deprecated flag fixes #391
- prefabs doesn't honour floating properly
- incorrect default codepage, add spans function to colour module
- *(insim_core)* handle escaped codepages properly!
- letterboards got mangled
- coordinate xyz should be pub

### Other

- *(insim_core)* add aliases
- *(insim,insim_core)* AngVel normalised to human values were a mistake.
- *(insim,insim_core)* Headings normalised to human values were a mistake.
- *(insim,insim_core,outgauge)* Speeds normalised to human values were a mistake.
- use release-plz to automate releases, fixes #135
- [**breaking**] provide the ability to have Option<String> fields - such as Axi lname, Ism hname, and Rip rname
- improvements ([#386](https://github.com/theangryangel/insim.rs/pull/386))
- Kitcar dx experiments ([#358](https://github.com/theangryangel/insim.rs/pull/358))
- Dog fooding: Prefabs and Clockwork Carnage "examples" ([#354](https://github.com/theangryangel/insim.rs/pull/354))
- wiki is wrong: ^8 not ^9 ([#345](https://github.com/theangryangel/insim.rs/pull/345))
- *(insim,insim_core)* record the logic around colours and escaping
- *(insim_core)* reduce LoC with macro_rules
- *(insim_core)* use a lookup table to increase the performance of to_lossy_bytes, fixes #303 ([#325](https://github.com/theangryangel/insim.rs/pull/325))
- criterion
- performance and correctness fixes ([#302](https://github.com/theangryangel/insim.rs/pull/302))
- refactor!
- [**breaking**] clockwork-carnage insim checkpoint edition
- Merge ObjectInfo and ObjectKind ([#272](https://github.com/theangryangel/insim.rs/pull/272))
