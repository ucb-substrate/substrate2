# Changelog

## [0.7.2](https://github.com/ucb-substrate/substrate2/compare/cache-v0.7.1...cache-v0.7.2) (2025-06-20)


### Bug Fixes

* **deps:** update tonic monorepo to 0.13 ([#642](https://github.com/ucb-substrate/substrate2/issues/642)) ([2b34382](https://github.com/ucb-substrate/substrate2/commit/2b34382d065141ef05f0c8998cd8de39bc4c6154))

## [0.7.1](https://github.com/ucb-substrate/substrate2/compare/cache-v0.7.0...cache-v0.7.1) (2025-01-24)


### Dependencies

* update dependencies ([0b87032](https://github.com/ucb-substrate/substrate2/commit/0b8703276631fbb19a958453394c981d6b092441))
* update dependencies ([#538](https://github.com/ucb-substrate/substrate2/issues/538)) ([19438d6](https://github.com/ucb-substrate/substrate2/commit/19438d65ac7078a2a971b4147420364ca0717763))
* update deps, GH actions ([#551](https://github.com/ucb-substrate/substrate2/issues/551)) ([357e82a](https://github.com/ucb-substrate/substrate2/commit/357e82ae0a01317d3ad5afb33b5290d3ac10cd7a))

## [0.7.0](https://github.com/ucb-substrate/substrate2/compare/cache-v0.6.0...cache-v0.7.0) (2025-01-23)


### Features

* **ac:** implement Save for Substrate types in Spectre AC sim, fix lints ([#471](https://github.com/ucb-substrate/substrate2/issues/471)) ([9825520](https://github.com/ucb-substrate/substrate2/commit/98255207569cc00bd9ddc35419c2df1e48f1999c))
* **cache-config:** allow configuration of cache via config files ([#192](https://github.com/ucb-substrate/substrate2/issues/192)) ([0461402](https://github.com/ucb-substrate/substrate2/commit/0461402edfc1ec0886bbb25cf5471ee8480754fc))
* **cache:** add initial implementation of in-memory caching ([#150](https://github.com/ucb-substrate/substrate2/issues/150)) ([2b26077](https://github.com/ucb-substrate/substrate2/commit/2b26077d5d9726c2689d489ac428c67c039dbb1d))
* **cache:** add local cache implementation ([#168](https://github.com/ucb-substrate/substrate2/issues/168)) ([676b585](https://github.com/ucb-substrate/substrate2/commit/676b5851488594824c4cd31c310e4b7d7bdb0a59))
* **cache:** bump dependencies ([#325](https://github.com/ucb-substrate/substrate2/issues/325)) ([7506a8a](https://github.com/ucb-substrate/substrate2/commit/7506a8ad84d0101b8a8b654bd98face751beae81))
* **cache:** implement persistent caching ([#171](https://github.com/ucb-substrate/substrate2/issues/171)) ([1f8ea24](https://github.com/ucb-substrate/substrate2/commit/1f8ea24f805085392bfd1a2067bb8774d0fa4ae4))
* **grid:** add ATOLL LCM routing grid and layer stack definition ([#338](https://github.com/ucb-substrate/substrate2/issues/338)) ([1e1ad90](https://github.com/ucb-substrate/substrate2/commit/1e1ad90d02b50dd0dd15516c306971241bf30b7c))
* **magic:** support magic for pex and lvs extraction ([#465](https://github.com/ucb-substrate/substrate2/issues/465)) ([c759341](https://github.com/ucb-substrate/substrate2/commit/c759341f065cf1e8aca8c4552a214391a7149cbf))
* **namespacing:** enforce namespace format ([#194](https://github.com/ucb-substrate/substrate2/issues/194)) ([90b1ebd](https://github.com/ucb-substrate/substrate2/commit/90b1ebdee52dc934cdde2996520e1acecf323c81))
* **primitives:** revamp schematic primitives ([#291](https://github.com/ucb-substrate/substrate2/issues/291)) ([e5ba06a](https://github.com/ucb-substrate/substrate2/commit/e5ba06ab10008b72e78397ad70781caa6bc61791))
* **remote-cache:** add initial implementation of remote-cache ([#166](https://github.com/ucb-substrate/substrate2/issues/166)) ([7d90aab](https://github.com/ucb-substrate/substrate2/commit/7d90aab47c282cf90e814ffce357a1e694c0c357))
* **terminals:** add support for terminal paths ([#236](https://github.com/ucb-substrate/substrate2/issues/236)) ([3fba7f6](https://github.com/ucb-substrate/substrate2/commit/3fba7f6227bbf2efcaf79d849c79175e44d783a4))
* **testing:** clean up wording/naming and add new tests ([#190](https://github.com/ucb-substrate/substrate2/issues/190)) ([d60076b](https://github.com/ucb-substrate/substrate2/commit/d60076b49a7f03663cddb5abe59ec047dcab8462))
* **tests:** show server errors and fix port picking ([#195](https://github.com/ucb-substrate/substrate2/issues/195)) ([2e477e3](https://github.com/ucb-substrate/substrate2/commit/2e477e3a733e6668ea1222c8a6796798e7dca9dd))
* **windows:** fix issues for windows ([#197](https://github.com/ucb-substrate/substrate2/issues/197)) ([008b607](https://github.com/ucb-substrate/substrate2/commit/008b607b2c21c14ac3106dca6eb74d806131ef8f))


### Bug Fixes

* **build:** fix build script for publishing ([#202](https://github.com/ucb-substrate/substrate2/issues/202)) ([de11a28](https://github.com/ucb-substrate/substrate2/commit/de11a28e79fea1b7a611f5f7a7815ff5433adaf9))
* **deps:** update rust crate prost-types to 0.12 ([#300](https://github.com/ucb-substrate/substrate2/issues/300)) ([06ca94e](https://github.com/ucb-substrate/substrate2/commit/06ca94e903b6996876585f162f82ff8615025710))
* **tests:** fix hanging test ([#246](https://github.com/ucb-substrate/substrate2/issues/246)) ([b60c7f2](https://github.com/ucb-substrate/substrate2/commit/b60c7f26db1993069d542d8333e173293f4c217b))
* **tests:** increase cache server wait time ([#167](https://github.com/ucb-substrate/substrate2/issues/167)) ([b0db3aa](https://github.com/ucb-substrate/substrate2/commit/b0db3aa6285367de1650e972c9cf7e2185a68250))
* **tests:** use `portpicker` to pick available ports in tests ([#170](https://github.com/ucb-substrate/substrate2/issues/170)) ([072998c](https://github.com/ucb-substrate/substrate2/commit/072998c32a97988494d2312b2676479ed4cb28fe))

## [0.6.0](https://github.com/ucb-substrate/substrate2/compare/cache-v0.5.0...cache-v0.6.0) (2025-01-22)


### Features

* **ac:** implement Save for Substrate types in Spectre AC sim, fix lints ([#471](https://github.com/ucb-substrate/substrate2/issues/471)) ([9825520](https://github.com/ucb-substrate/substrate2/commit/98255207569cc00bd9ddc35419c2df1e48f1999c))
* **grid:** add ATOLL LCM routing grid and layer stack definition ([#338](https://github.com/ucb-substrate/substrate2/issues/338)) ([1e1ad90](https://github.com/ucb-substrate/substrate2/commit/1e1ad90d02b50dd0dd15516c306971241bf30b7c))
* **magic:** support magic for pex and lvs extraction ([#465](https://github.com/ucb-substrate/substrate2/issues/465)) ([c759341](https://github.com/ucb-substrate/substrate2/commit/c759341f065cf1e8aca8c4552a214391a7149cbf))

## [0.5.0](https://github.com/ucb-substrate/substrate2/compare/cache-v0.4.0...cache-v0.5.0) (2023-11-25)


### Features

* **cache:** bump dependencies ([#325](https://github.com/ucb-substrate/substrate2/issues/325)) ([7506a8a](https://github.com/ucb-substrate/substrate2/commit/7506a8ad84d0101b8a8b654bd98face751beae81))

## [0.4.0](https://github.com/ucb-substrate/substrate2/compare/cache-v0.3.1...cache-v0.4.0) (2023-11-02)


### Features

* **primitives:** revamp schematic primitives ([#291](https://github.com/ucb-substrate/substrate2/issues/291)) ([e5ba06a](https://github.com/ucb-substrate/substrate2/commit/e5ba06ab10008b72e78397ad70781caa6bc61791))


### Bug Fixes

* **deps:** update rust crate prost-types to 0.12 ([#300](https://github.com/ucb-substrate/substrate2/issues/300)) ([06ca94e](https://github.com/ucb-substrate/substrate2/commit/06ca94e903b6996876585f162f82ff8615025710))

## [0.3.1](https://github.com/substrate-labs/substrate2/compare/cache-v0.3.0...cache-v0.3.1) (2023-08-08)


### Bug Fixes

* **tests:** fix hanging test ([#246](https://github.com/substrate-labs/substrate2/issues/246)) ([b60c7f2](https://github.com/substrate-labs/substrate2/commit/b60c7f26db1993069d542d8333e173293f4c217b))

## [0.3.0](https://github.com/substrate-labs/substrate2/compare/cache-v0.2.1...cache-v0.3.0) (2023-08-05)


### Features

* **terminals:** add support for terminal paths ([#236](https://github.com/substrate-labs/substrate2/issues/236)) ([3fba7f6](https://github.com/substrate-labs/substrate2/commit/3fba7f6227bbf2efcaf79d849c79175e44d783a4))

## [0.2.1](https://github.com/substrate-labs/substrate2/compare/cache-v0.2.0...cache-v0.2.1) (2023-08-04)


### Bug Fixes

* **build:** fix build script for publishing ([#202](https://github.com/substrate-labs/substrate2/issues/202)) ([de11a28](https://github.com/substrate-labs/substrate2/commit/de11a28e79fea1b7a611f5f7a7815ff5433adaf9))

## [0.2.0](https://github.com/substrate-labs/substrate2/compare/cache-v0.1.0...cache-v0.2.0) (2023-07-23)


### Features

* **cache-config:** allow configuration of cache via config files ([#192](https://github.com/substrate-labs/substrate2/issues/192)) ([0461402](https://github.com/substrate-labs/substrate2/commit/0461402edfc1ec0886bbb25cf5471ee8480754fc))
* **cache:** add local cache implementation ([#168](https://github.com/substrate-labs/substrate2/issues/168)) ([676b585](https://github.com/substrate-labs/substrate2/commit/676b5851488594824c4cd31c310e4b7d7bdb0a59))
* **cache:** implement persistent caching ([#171](https://github.com/substrate-labs/substrate2/issues/171)) ([1f8ea24](https://github.com/substrate-labs/substrate2/commit/1f8ea24f805085392bfd1a2067bb8774d0fa4ae4))
* **namespacing:** enforce namespace format ([#194](https://github.com/substrate-labs/substrate2/issues/194)) ([90b1ebd](https://github.com/substrate-labs/substrate2/commit/90b1ebdee52dc934cdde2996520e1acecf323c81))
* **remote-cache:** add initial implementation of remote-cache ([#166](https://github.com/substrate-labs/substrate2/issues/166)) ([7d90aab](https://github.com/substrate-labs/substrate2/commit/7d90aab47c282cf90e814ffce357a1e694c0c357))
* **testing:** clean up wording/naming and add new tests ([#190](https://github.com/substrate-labs/substrate2/issues/190)) ([d60076b](https://github.com/substrate-labs/substrate2/commit/d60076b49a7f03663cddb5abe59ec047dcab8462))
* **tests:** show server errors and fix port picking ([#195](https://github.com/substrate-labs/substrate2/issues/195)) ([2e477e3](https://github.com/substrate-labs/substrate2/commit/2e477e3a733e6668ea1222c8a6796798e7dca9dd))
* **windows:** fix issues for windows ([#197](https://github.com/substrate-labs/substrate2/issues/197)) ([008b607](https://github.com/substrate-labs/substrate2/commit/008b607b2c21c14ac3106dca6eb74d806131ef8f))


### Bug Fixes

* **tests:** increase cache server wait time ([#167](https://github.com/substrate-labs/substrate2/issues/167)) ([b0db3aa](https://github.com/substrate-labs/substrate2/commit/b0db3aa6285367de1650e972c9cf7e2185a68250))
* **tests:** use `portpicker` to pick available ports in tests ([#170](https://github.com/substrate-labs/substrate2/issues/170)) ([072998c](https://github.com/substrate-labs/substrate2/commit/072998c32a97988494d2312b2676479ed4cb28fe))

## 0.1.0 (2023-07-07)


### Features

* **cache:** add initial implementation of in-memory caching ([#150](https://github.com/substrate-labs/substrate2/issues/150)) ([2b26077](https://github.com/substrate-labs/substrate2/commit/2b26077d5d9726c2689d489ac428c67c039dbb1d))
