# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [5.0.0](https://github.com/theangryangel/insim.rs/compare/insim-v4.1.0...insim-v5.0.0) - 2026-06-22

### Added

- add convenience methods decode_slice, decode_bytes to Decode and encode_bytes to Encode traits
- Semi-typed IS_SET support
- Mci RETIRED flag, fixes #337 ([#387](https://github.com/theangryangel/insim.rs/pull/387))
- schemars support & python library backed by insim.rs ([#375](https://github.com/theangryangel/insim.rs/pull/375))
- *(insim)* add SET packet support
- add useful display outputs
- Allow unknown packets through a feature flag (Fixes #332) ([#336](https://github.com/theangryangel/insim.rs/pull/336))
- surviving PiranDD
- *(prefab)* New tools ([#319](https://github.com/theangryangel/insim.rs/pull/319))
- [**breaking**] insim v10 + Proper Object Handling + Updated PTH and new PIN file handling + other breaking changes ([#266](https://github.com/theangryangel/insim.rs/pull/266))
- *(kitcar)* a series of micro libraries for building mini-games ([#233](https://github.com/theangryangel/insim.rs/pull/233))

### Fixed

- fix!(insim,insim_core): Fix incorrectly typed steer, accelf and accelr
- *(insim)* make tokio read implementation cancel safe
- docs.rs deprecated flag fixes #391
- *(insim)* allow udpport to be explicitly set ([#340](https://github.com/theangryangel/insim.rs/pull/340))
- *(insim)* Review Packet::size_hint, fixes #327 ([#337](https://github.com/theangryangel/insim.rs/pull/337))

### Other

- Merge main into 418-racetracker
- *(insim,insim_core)* AngVel normalised to human values were a mistake.
- *(insim,insim_core)* Headings normalised to human values were a mistake.
- *(insim,insim_core,outgauge)* Speeds normalised to human values were a mistake.
- *(insim_extra)* use multi_index_map
- use release-plz to automate releases, fixes #135
- [**breaking**] provide the ability to have Option<String> fields - such as Axi lname, Ism hname, and Rip rname
- *(insim)* bye-bye Spawned and InsimTask, fixes #398 ([#399](https://github.com/theangryangel/insim.rs/pull/399))
- insim_extras -> insim_extra + kitcar
- improvements ([#386](https://github.com/theangryangel/insim.rs/pull/386))
- lint
- rename kitcar* to insim_extras*
- doc
- Kitcar dx experiments ([#358](https://github.com/theangryangel/insim.rs/pull/358))
- Dog fooding: Prefabs and Clockwork Carnage "examples" ([#354](https://github.com/theangryangel/insim.rs/pull/354))
- *(insim,insim_core)* record the logic around colours and escaping
- *(insim)* remove now redundant traits, fixes #326 ([#329](https://github.com/theangryangel/insim.rs/pull/329))
- *(insim_core)* reduce LoC with macro_rules
- performance and correctness fixes ([#302](https://github.com/theangryangel/insim.rs/pull/302))
- refactor!
- [**breaking**] clockwork-carnage insim checkpoint edition
- Merge ObjectInfo and ObjectKind ([#272](https://github.com/theangryangel/insim.rs/pull/272))
