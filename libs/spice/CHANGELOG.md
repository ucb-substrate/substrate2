# Changelog

* The following workspace dependencies were updated
  * dependencies
    * substrate bumped from 0.8.0 to 0.8.1

## [0.7.0](https://github.com/ucb-substrate/substrate2/compare/spice-v0.6.0...spice-v0.7.0) (2023-11-25)


### Features

* **simulation:** improve simulation APIs ([#320](https://github.com/ucb-substrate/substrate2/issues/320)) ([4ed59a1](https://github.com/ucb-substrate/substrate2/commit/4ed59a1283f9546e8336cc96015bd87c55682777))
* **stdcells:** add standard cell support to Sky130 PDK ([#323](https://github.com/ucb-substrate/substrate2/issues/323)) ([0b2048e](https://github.com/ucb-substrate/substrate2/commit/0b2048ed44d89c5de87380cac48a4bbff2b4c20a))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * substrate bumped from 0.7.1 to 0.8.0

## [0.6.0](https://github.com/ucb-substrate/substrate2/compare/spice-v0.5.0...spice-v0.6.0) (2023-11-04)


### Features

* **spice:** refactor netlisting and fix voltage source netlist ([#316](https://github.com/ucb-substrate/substrate2/issues/316)) ([7a3df69](https://github.com/ucb-substrate/substrate2/commit/7a3df695cf9b38c837ff86d5a5da2417c4db7aa2))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * scir bumped from 0.6.0 to 0.7.0
    * substrate bumped from 0.7.0 to 0.7.1

## [0.5.0](https://github.com/ucb-substrate/substrate2/compare/spice-v0.4.0...spice-v0.5.0) (2023-11-02)


### Features

* **netlister:** reduce duplicate code between spectre and SPICE netlisters ([#261](https://github.com/ucb-substrate/substrate2/issues/261)) ([5ba3623](https://github.com/ucb-substrate/substrate2/commit/5ba36230e653e4dc77819c5c50b527311768cd83))
* **netlists:** use consistent ordering via indexmap ([#266](https://github.com/ucb-substrate/substrate2/issues/266)) ([f275c19](https://github.com/ucb-substrate/substrate2/commit/f275c19396ed4f7d255836822ff72b808f89cde7)), closes [#265](https://github.com/ucb-substrate/substrate2/issues/265)
* **ngspice:** create ngspice simulator ([#274](https://github.com/ucb-substrate/substrate2/issues/274)) ([0205300](https://github.com/ucb-substrate/substrate2/commit/02053006bc26d0b3d9e1d380def89836d7921857))
* **primitives:** revamp schematic primitives ([#291](https://github.com/ucb-substrate/substrate2/issues/291)) ([e5ba06a](https://github.com/ucb-substrate/substrate2/commit/e5ba06ab10008b72e78397ad70781caa6bc61791))


### Bug Fixes

* **deps:** remove opacity from spice library ([#287](https://github.com/ucb-substrate/substrate2/issues/287)) ([a45b728](https://github.com/ucb-substrate/substrate2/commit/a45b7288e240a9955d91acb437fa251fccb66b75))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * scir bumped from 0.5.0 to 0.6.0
    * substrate bumped from 0.6.1 to 0.7.0

## [0.4.0](https://github.com/substrate-labs/substrate2/compare/spice-v0.3.0...spice-v0.4.0) (2023-08-08)


### Features

* **slices:** use `SliceOne` instead of `Slice` where possible ([#253](https://github.com/substrate-labs/substrate2/issues/253)) ([8eba8ed](https://github.com/substrate-labs/substrate2/commit/8eba8ed5aad0aa4911ae31f4521d297487256087))
* **validation:** create type for unvalidated SCIR library ([#252](https://github.com/substrate-labs/substrate2/issues/252)) ([1550a22](https://github.com/substrate-labs/substrate2/commit/1550a22b9a1c9f7cd9717feaa45d00487cc8848e))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * scir bumped from 0.4.0 to 0.5.0

## [0.3.0](https://github.com/substrate-labs/substrate2/compare/spice-v0.2.0...spice-v0.3.0) (2023-08-05)


### Features

* **terminals:** add support for terminal paths ([#236](https://github.com/substrate-labs/substrate2/issues/236)) ([3fba7f6](https://github.com/substrate-labs/substrate2/commit/3fba7f6227bbf2efcaf79d849c79175e44d783a4))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * scir bumped from 0.3.0 to 0.4.0

## [0.2.0](https://github.com/substrate-labs/substrate2/compare/spice-v0.1.0...spice-v0.2.0) (2023-08-04)


### Features

* **primitives:** support parameters in SCIR primitive devices ([#232](https://github.com/substrate-labs/substrate2/issues/232)) ([a8f5b45](https://github.com/substrate-labs/substrate2/commit/a8f5b45a00b77d050f6a812c469e19da3305e064))
* **repo:** reorganize repo ([#207](https://github.com/substrate-labs/substrate2/issues/207)) ([54a6b43](https://github.com/substrate-labs/substrate2/commit/54a6b43079d283a29bc0aa9e18dc6230b56fa385))
* **schematics:** blackboxes can reference nodes ([#208](https://github.com/substrate-labs/substrate2/issues/208)) ([d998b4a](https://github.com/substrate-labs/substrate2/commit/d998b4a133d47d0123768dfb3c27f8ee32ed9db9))
* **spice-parser:** spice parser follows include directives ([#229](https://github.com/substrate-labs/substrate2/issues/229)) ([5259acf](https://github.com/substrate-labs/substrate2/commit/5259acfa703c3879d44d324279293278c46f1ff5))
* **validation:** SCIR driver analysis and validation ([#239](https://github.com/substrate-labs/substrate2/issues/239)) ([5a91448](https://github.com/substrate-labs/substrate2/commit/5a914489294bed06be1bd34aaa1036e4357d9a52))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * scir bumped from 0.2.0 to 0.3.0

## [0.1.0](https://github.com/substrate-labs/substrate2/compare/spice-v0.0.0...spice-v0.1.0) (2023-07-23)


### Features

* **conversion:** convert parsed SPICE to SCIR ([#178](https://github.com/substrate-labs/substrate2/issues/178)) ([9cb7bc3](https://github.com/substrate-labs/substrate2/commit/9cb7bc3ba549ae12e7a59465241c848800c39363))
* **organization:** move `spice` from netlist/ to libs/ ([#174](https://github.com/substrate-labs/substrate2/issues/174)) ([bd31a44](https://github.com/substrate-labs/substrate2/commit/bd31a4481aef357daeb2c217dd7f403f6f882f78))
* **proc-macros:** codegen for schematic hard macros ([#191](https://github.com/substrate-labs/substrate2/issues/191)) ([50240b1](https://github.com/substrate-labs/substrate2/commit/50240b167876873c4133315d35298b44e8eeac51))
* **scir-instances:** allow Substrate users to instantiate raw SCIR instances ([#184](https://github.com/substrate-labs/substrate2/issues/184)) ([8fd5192](https://github.com/substrate-labs/substrate2/commit/8fd5192fd2017ab04e9e3220612d0a132702bb2e))
* **spice-to-scir:** do not convert blackboxed subcircuits ([#179](https://github.com/substrate-labs/substrate2/issues/179)) ([c501313](https://github.com/substrate-labs/substrate2/commit/c501313334279b636f1d8b581357dd805177f1ca))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * scir bumped from 0.1.0 to 0.2.0
