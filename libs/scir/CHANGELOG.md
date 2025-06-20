# Changelog

## [0.9.2](https://github.com/ucb-substrate/substrate2/compare/scir-v0.9.1...scir-v0.9.2) (2025-06-20)


### Features

* **scir:** driver analysis handles opaque SCIR primitives correctly ([#636](https://github.com/ucb-substrate/substrate2/issues/636)) ([7dccba4](https://github.com/ucb-substrate/substrate2/commit/7dccba4da427b4b07b93b92d13ad9ecb1aef8512))
* **scir:** remove unused primitives when pruning SCIR libraries ([#658](https://github.com/ucb-substrate/substrate2/issues/658)) ([7536255](https://github.com/ucb-substrate/substrate2/commit/753625570bb6edce9e7637ca30d2d7f0225a7d21))
* **scir:** support removing cells from `LibraryBuilder` ([#657](https://github.com/ucb-substrate/substrate2/issues/657)) ([eb57369](https://github.com/ucb-substrate/substrate2/commit/eb573691672c6109c41942811a9cdd1ffe75087b))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * uniquify bumped from 0.4.0 to 0.4.1
    * enumify bumped from 0.2.1 to 0.2.2

## [0.9.1](https://github.com/ucb-substrate/substrate2/compare/scir-v0.9.0...scir-v0.9.1) (2025-01-24)


### Dependencies

* bump thiserror to v2 ([206a09e](https://github.com/ucb-substrate/substrate2/commit/206a09e01540b4e4a2b0d8b4c6e20619f35a8452))
* update dependencies ([0b87032](https://github.com/ucb-substrate/substrate2/commit/0b8703276631fbb19a958453394c981d6b092441))
* The following workspace dependencies were updated
  * dependencies
    * enumify bumped from 0.2.0 to 0.2.1

## [0.9.0](https://github.com/ucb-substrate/substrate2/compare/scir-v0.8.0...scir-v0.9.0) (2025-01-23)


### Features

* **cdl2spice:** add CDL to SPICE conversion command line tool ([#420](https://github.com/ucb-substrate/substrate2/issues/420)) ([1edb23a](https://github.com/ucb-substrate/substrate2/commit/1edb23a7bbd45d96bbb1c11418eb0d0843b7138b))
* **cdl:** add CDL parser ([#419](https://github.com/ucb-substrate/substrate2/issues/419)) ([23f0dab](https://github.com/ucb-substrate/substrate2/commit/23f0dab7b7a94cbe8960371b89d15211bddf51da))
* **io:** composable port directions and runtime connection checking ([#231](https://github.com/ucb-substrate/substrate2/issues/231)) ([e1e367a](https://github.com/ucb-substrate/substrate2/commit/e1e367a2b8940319cb4f804888746a094f06e161))
* **netlister:** reduce duplicate code between spectre and SPICE netlisters ([#261](https://github.com/ucb-substrate/substrate2/issues/261)) ([5ba3623](https://github.com/ucb-substrate/substrate2/commit/5ba36230e653e4dc77819c5c50b527311768cd83))
* **netlists:** consistent Spectre/Spice netlist API ([#349](https://github.com/ucb-substrate/substrate2/issues/349)) ([2f9fabf](https://github.com/ucb-substrate/substrate2/commit/2f9fabf336fa1048d759e78834979ef892fc0bcf))
* **netlists:** support ideal 2-terminal capacitors ([#269](https://github.com/ucb-substrate/substrate2/issues/269)) ([7de9843](https://github.com/ucb-substrate/substrate2/commit/7de9843c9b629ea06518448fe26d384de4a66cdc))
* **netlists:** use consistent ordering via indexmap ([#266](https://github.com/ucb-substrate/substrate2/issues/266)) ([f275c19](https://github.com/ucb-substrate/substrate2/commit/f275c19396ed4f7d255836822ff72b808f89cde7)), closes [#265](https://github.com/ucb-substrate/substrate2/issues/265)
* **ngspice:** create ngspice simulator ([#274](https://github.com/ucb-substrate/substrate2/issues/274)) ([0205300](https://github.com/ucb-substrate/substrate2/commit/02053006bc26d0b3d9e1d380def89836d7921857))
* **ports:** add name map for ports ([#237](https://github.com/ucb-substrate/substrate2/issues/237)) ([118b484](https://github.com/ucb-substrate/substrate2/commit/118b4849e4408aa93d9fa39ef387dd051b2f5044))
* **primitives:** add 2-terminal capacitor primitive ([#262](https://github.com/ucb-substrate/substrate2/issues/262)) ([bc622b9](https://github.com/ucb-substrate/substrate2/commit/bc622b936a77719dbf92f76fdc3cbfbae61e9021))
* **primitives:** add built-in resistor and capacitor schematic blocks ([bc622b9](https://github.com/ucb-substrate/substrate2/commit/bc622b936a77719dbf92f76fdc3cbfbae61e9021))
* **primitives:** revamp schematic primitives ([#291](https://github.com/ucb-substrate/substrate2/issues/291)) ([e5ba06a](https://github.com/ucb-substrate/substrate2/commit/e5ba06ab10008b72e78397ad70781caa6bc61791))
* **primitives:** support parameters in SCIR primitive devices ([#232](https://github.com/ucb-substrate/substrate2/issues/232)) ([a8f5b45](https://github.com/ucb-substrate/substrate2/commit/a8f5b45a00b77d050f6a812c469e19da3305e064))
* **repo:** reorganize repo ([#207](https://github.com/ucb-substrate/substrate2/issues/207)) ([54a6b43](https://github.com/ucb-substrate/substrate2/commit/54a6b43079d283a29bc0aa9e18dc6230b56fa385))
* **save-api:** add typed API for saving arbitrary signals ([#228](https://github.com/ucb-substrate/substrate2/issues/228)) ([046be02](https://github.com/ucb-substrate/substrate2/commit/046be02acbedc7fa2bb4896b92ec17babd80eee5))
* **schematics:** blackboxes can reference nodes ([#208](https://github.com/ucb-substrate/substrate2/issues/208)) ([d998b4a](https://github.com/ucb-substrate/substrate2/commit/d998b4a133d47d0123768dfb3c27f8ee32ed9db9))
* **schematics:** user-specified schematic hierarchy flattening ([#222](https://github.com/ucb-substrate/substrate2/issues/222)) ([251f377](https://github.com/ucb-substrate/substrate2/commit/251f37778526d2f1c08a2b3c66f72ffe273021fa))
* **scir:** expose port directions, update docs ([#426](https://github.com/ucb-substrate/substrate2/issues/426)) ([fd883b7](https://github.com/ucb-substrate/substrate2/commit/fd883b7ca803f7b45d4736a7b4b460e602b84704))
* **scir:** SCIR lib imports merge only the instantiated cell ([#437](https://github.com/ucb-substrate/substrate2/issues/437)) ([7a0b285](https://github.com/ucb-substrate/substrate2/commit/7a0b285446b224569d430a2764e3a4e6d30ee031))
* **slices:** use `SliceOne` instead of `Slice` where possible ([#253](https://github.com/ucb-substrate/substrate2/issues/253)) ([8eba8ed](https://github.com/ucb-substrate/substrate2/commit/8eba8ed5aad0aa4911ae31f4521d297487256087))
* **spice-parser:** spice parser follows include directives ([#229](https://github.com/ucb-substrate/substrate2/issues/229)) ([5259acf](https://github.com/ucb-substrate/substrate2/commit/5259acfa703c3879d44d324279293278c46f1ff5))
* **spice:** refactor netlisting and fix voltage source netlist ([#316](https://github.com/ucb-substrate/substrate2/issues/316)) ([7a3df69](https://github.com/ucb-substrate/substrate2/commit/7a3df695cf9b38c837ff86d5a5da2417c4db7aa2))
* **terminals:** add support for terminal paths ([#236](https://github.com/ucb-substrate/substrate2/issues/236)) ([3fba7f6](https://github.com/ucb-substrate/substrate2/commit/3fba7f6227bbf2efcaf79d849c79175e44d783a4))
* **tests:** reorganize tests and documentation ([#464](https://github.com/ucb-substrate/substrate2/issues/464)) ([928b9b7](https://github.com/ucb-substrate/substrate2/commit/928b9b7c45dc334ca11d86e4564edc58bf6db6f2))
* **validation:** create type for unvalidated SCIR library ([#252](https://github.com/ucb-substrate/substrate2/issues/252)) ([1550a22](https://github.com/ucb-substrate/substrate2/commit/1550a22b9a1c9f7cd9717feaa45d00487cc8848e))
* **validation:** SCIR driver analysis and validation ([#239](https://github.com/ucb-substrate/substrate2/issues/239)) ([5a91448](https://github.com/ucb-substrate/substrate2/commit/5a914489294bed06be1bd34aaa1036e4357d9a52))


### Bug Fixes

* **deps:** update rust crate rust_decimal to 1.31 ([#219](https://github.com/ucb-substrate/substrate2/issues/219)) ([6f596d5](https://github.com/ucb-substrate/substrate2/commit/6f596d5c46dc1bf045a1b8a5ef727adbc3b147cf))
* **deps:** update rust crate rust_decimal to 1.32 ([#296](https://github.com/ucb-substrate/substrate2/issues/296)) ([a2fe877](https://github.com/ucb-substrate/substrate2/commit/a2fe877d03d3f907f348d7711a2132194ae91034))
* **deps:** update rust crate rust_decimal_macros to 1.31 ([#220](https://github.com/ucb-substrate/substrate2/issues/220)) ([72147d3](https://github.com/ucb-substrate/substrate2/commit/72147d385368e2bd302821c981dd75209aa87dcb))
* **deps:** update rust crate rust_decimal_macros to 1.32 ([#297](https://github.com/ucb-substrate/substrate2/issues/297)) ([5474cc8](https://github.com/ucb-substrate/substrate2/commit/5474cc8778b81c30b34fc7d146eec6e5e2532a26))
* **schematic:** correctly deduplicate SCIR cell names during export ([#435](https://github.com/ucb-substrate/substrate2/issues/435)) ([48af6fc](https://github.com/ucb-substrate/substrate2/commit/48af6fcd360fe9f2e8246ed0198945bfbae72724))
* **scir:** add additional functionality for SCIR and SPICE libraries ([#337](https://github.com/ucb-substrate/substrate2/issues/337)) ([e49f075](https://github.com/ucb-substrate/substrate2/commit/e49f07529273c38cc8ec9ae1a5020ae48fb2a202))
* **scir:** avoid panic when converting inst paths ([#400](https://github.com/ucb-substrate/substrate2/issues/400)) ([34a86da](https://github.com/ucb-substrate/substrate2/commit/34a86da36679628f44dce366d9168420179d9379))
* **scir:** remove use of opacity from SCIR ([#286](https://github.com/ucb-substrate/substrate2/issues/286)) ([5e38b28](https://github.com/ucb-substrate/substrate2/commit/5e38b288629b5f2d6d3ca372418a331b6bd98e5e))
* **sim:** add `Sky130CommercialSchema` and simplify trait bounds ([#351](https://github.com/ucb-substrate/substrate2/issues/351)) ([c95e5c0](https://github.com/ucb-substrate/substrate2/commit/c95e5c08e5fc3bf6e34e00731ab4e38e9e586c01))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * diagnostics bumped from 0.3.0 to 0.4.0
    * uniquify bumped from 0.3.0 to 0.4.0
    * enumify bumped from 0.1.1 to 0.2.0

## [0.8.0](https://github.com/ucb-substrate/substrate2/compare/scir-v0.7.0...scir-v0.8.0) (2025-01-22)


### Features

* **cdl2spice:** add CDL to SPICE conversion command line tool ([#420](https://github.com/ucb-substrate/substrate2/issues/420)) ([1edb23a](https://github.com/ucb-substrate/substrate2/commit/1edb23a7bbd45d96bbb1c11418eb0d0843b7138b))
* **cdl:** add CDL parser ([#419](https://github.com/ucb-substrate/substrate2/issues/419)) ([23f0dab](https://github.com/ucb-substrate/substrate2/commit/23f0dab7b7a94cbe8960371b89d15211bddf51da))
* **netlists:** consistent Spectre/Spice netlist API ([#349](https://github.com/ucb-substrate/substrate2/issues/349)) ([2f9fabf](https://github.com/ucb-substrate/substrate2/commit/2f9fabf336fa1048d759e78834979ef892fc0bcf))
* **scir:** expose port directions, update docs ([#426](https://github.com/ucb-substrate/substrate2/issues/426)) ([fd883b7](https://github.com/ucb-substrate/substrate2/commit/fd883b7ca803f7b45d4736a7b4b460e602b84704))
* **scir:** SCIR lib imports merge only the instantiated cell ([#437](https://github.com/ucb-substrate/substrate2/issues/437)) ([7a0b285](https://github.com/ucb-substrate/substrate2/commit/7a0b285446b224569d430a2764e3a4e6d30ee031))
* **tests:** reorganize tests and documentation ([#464](https://github.com/ucb-substrate/substrate2/issues/464)) ([928b9b7](https://github.com/ucb-substrate/substrate2/commit/928b9b7c45dc334ca11d86e4564edc58bf6db6f2))


### Bug Fixes

* **schematic:** correctly deduplicate SCIR cell names during export ([#435](https://github.com/ucb-substrate/substrate2/issues/435)) ([48af6fc](https://github.com/ucb-substrate/substrate2/commit/48af6fcd360fe9f2e8246ed0198945bfbae72724))
* **scir:** add additional functionality for SCIR and SPICE libraries ([#337](https://github.com/ucb-substrate/substrate2/issues/337)) ([e49f075](https://github.com/ucb-substrate/substrate2/commit/e49f07529273c38cc8ec9ae1a5020ae48fb2a202))
* **scir:** avoid panic when converting inst paths ([#400](https://github.com/ucb-substrate/substrate2/issues/400)) ([34a86da](https://github.com/ucb-substrate/substrate2/commit/34a86da36679628f44dce366d9168420179d9379))
* **sim:** add `Sky130CommercialSchema` and simplify trait bounds ([#351](https://github.com/ucb-substrate/substrate2/issues/351)) ([c95e5c0](https://github.com/ucb-substrate/substrate2/commit/c95e5c08e5fc3bf6e34e00731ab4e38e9e586c01))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * uniquify bumped from 0.2.0 to 0.3.0
    * enumify bumped from 0.1.0 to 0.1.1

## [0.7.0](https://github.com/ucb-substrate/substrate2/compare/scir-v0.6.0...scir-v0.7.0) (2023-11-04)


### Features

* **spice:** refactor netlisting and fix voltage source netlist ([#316](https://github.com/ucb-substrate/substrate2/issues/316)) ([7a3df69](https://github.com/ucb-substrate/substrate2/commit/7a3df695cf9b38c837ff86d5a5da2417c4db7aa2))

## [0.6.0](https://github.com/ucb-substrate/substrate2/compare/scir-v0.5.0...scir-v0.6.0) (2023-11-02)


### Features

* **netlister:** reduce duplicate code between spectre and SPICE netlisters ([#261](https://github.com/ucb-substrate/substrate2/issues/261)) ([5ba3623](https://github.com/ucb-substrate/substrate2/commit/5ba36230e653e4dc77819c5c50b527311768cd83))
* **netlists:** support ideal 2-terminal capacitors ([#269](https://github.com/ucb-substrate/substrate2/issues/269)) ([7de9843](https://github.com/ucb-substrate/substrate2/commit/7de9843c9b629ea06518448fe26d384de4a66cdc))
* **netlists:** use consistent ordering via indexmap ([#266](https://github.com/ucb-substrate/substrate2/issues/266)) ([f275c19](https://github.com/ucb-substrate/substrate2/commit/f275c19396ed4f7d255836822ff72b808f89cde7)), closes [#265](https://github.com/ucb-substrate/substrate2/issues/265)
* **ngspice:** create ngspice simulator ([#274](https://github.com/ucb-substrate/substrate2/issues/274)) ([0205300](https://github.com/ucb-substrate/substrate2/commit/02053006bc26d0b3d9e1d380def89836d7921857))
* **primitives:** add 2-terminal capacitor primitive ([#262](https://github.com/ucb-substrate/substrate2/issues/262)) ([bc622b9](https://github.com/ucb-substrate/substrate2/commit/bc622b936a77719dbf92f76fdc3cbfbae61e9021))
* **primitives:** add built-in resistor and capacitor schematic blocks ([bc622b9](https://github.com/ucb-substrate/substrate2/commit/bc622b936a77719dbf92f76fdc3cbfbae61e9021))
* **primitives:** revamp schematic primitives ([#291](https://github.com/ucb-substrate/substrate2/issues/291)) ([e5ba06a](https://github.com/ucb-substrate/substrate2/commit/e5ba06ab10008b72e78397ad70781caa6bc61791))


### Bug Fixes

* **deps:** update rust crate rust_decimal to 1.32 ([#296](https://github.com/ucb-substrate/substrate2/issues/296)) ([a2fe877](https://github.com/ucb-substrate/substrate2/commit/a2fe877d03d3f907f348d7711a2132194ae91034))
* **deps:** update rust crate rust_decimal_macros to 1.32 ([#297](https://github.com/ucb-substrate/substrate2/issues/297)) ([5474cc8](https://github.com/ucb-substrate/substrate2/commit/5474cc8778b81c30b34fc7d146eec6e5e2532a26))
* **scir:** remove use of opacity from SCIR ([#286](https://github.com/ucb-substrate/substrate2/issues/286)) ([5e38b28](https://github.com/ucb-substrate/substrate2/commit/5e38b288629b5f2d6d3ca372418a331b6bd98e5e))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * enumify bumped from 0.0.0 to 0.1.0

## [0.5.0](https://github.com/substrate-labs/substrate2/compare/scir-v0.4.0...scir-v0.5.0) (2023-08-08)


### Features

* **slices:** use `SliceOne` instead of `Slice` where possible ([#253](https://github.com/substrate-labs/substrate2/issues/253)) ([8eba8ed](https://github.com/substrate-labs/substrate2/commit/8eba8ed5aad0aa4911ae31f4521d297487256087))
* **validation:** create type for unvalidated SCIR library ([#252](https://github.com/substrate-labs/substrate2/issues/252)) ([1550a22](https://github.com/substrate-labs/substrate2/commit/1550a22b9a1c9f7cd9717feaa45d00487cc8848e))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * diagnostics bumped from 0.2.0 to 0.3.0

## [0.4.0](https://github.com/substrate-labs/substrate2/compare/scir-v0.3.0...scir-v0.4.0) (2023-08-05)


### Features

* **terminals:** add support for terminal paths ([#236](https://github.com/substrate-labs/substrate2/issues/236)) ([3fba7f6](https://github.com/substrate-labs/substrate2/commit/3fba7f6227bbf2efcaf79d849c79175e44d783a4))

## [0.3.0](https://github.com/substrate-labs/substrate2/compare/scir-v0.2.0...scir-v0.3.0) (2023-08-04)


### Features

* **io:** composable port directions and runtime connection checking ([#231](https://github.com/substrate-labs/substrate2/issues/231)) ([e1e367a](https://github.com/substrate-labs/substrate2/commit/e1e367a2b8940319cb4f804888746a094f06e161))
* **ports:** add name map for ports ([#237](https://github.com/substrate-labs/substrate2/issues/237)) ([118b484](https://github.com/substrate-labs/substrate2/commit/118b4849e4408aa93d9fa39ef387dd051b2f5044))
* **primitives:** support parameters in SCIR primitive devices ([#232](https://github.com/substrate-labs/substrate2/issues/232)) ([a8f5b45](https://github.com/substrate-labs/substrate2/commit/a8f5b45a00b77d050f6a812c469e19da3305e064))
* **repo:** reorganize repo ([#207](https://github.com/substrate-labs/substrate2/issues/207)) ([54a6b43](https://github.com/substrate-labs/substrate2/commit/54a6b43079d283a29bc0aa9e18dc6230b56fa385))
* **save-api:** add typed API for saving arbitrary signals ([#228](https://github.com/substrate-labs/substrate2/issues/228)) ([046be02](https://github.com/substrate-labs/substrate2/commit/046be02acbedc7fa2bb4896b92ec17babd80eee5))
* **schematics:** blackboxes can reference nodes ([#208](https://github.com/substrate-labs/substrate2/issues/208)) ([d998b4a](https://github.com/substrate-labs/substrate2/commit/d998b4a133d47d0123768dfb3c27f8ee32ed9db9))
* **schematics:** user-specified schematic hierarchy flattening ([#222](https://github.com/substrate-labs/substrate2/issues/222)) ([251f377](https://github.com/substrate-labs/substrate2/commit/251f37778526d2f1c08a2b3c66f72ffe273021fa))
* **spice-parser:** spice parser follows include directives ([#229](https://github.com/substrate-labs/substrate2/issues/229)) ([5259acf](https://github.com/substrate-labs/substrate2/commit/5259acfa703c3879d44d324279293278c46f1ff5))
* **validation:** SCIR driver analysis and validation ([#239](https://github.com/substrate-labs/substrate2/issues/239)) ([5a91448](https://github.com/substrate-labs/substrate2/commit/5a914489294bed06be1bd34aaa1036e4357d9a52))


### Bug Fixes

* **deps:** update rust crate rust_decimal to 1.31 ([#219](https://github.com/substrate-labs/substrate2/issues/219)) ([6f596d5](https://github.com/substrate-labs/substrate2/commit/6f596d5c46dc1bf045a1b8a5ef727adbc3b147cf))
* **deps:** update rust crate rust_decimal_macros to 1.31 ([#220](https://github.com/substrate-labs/substrate2/issues/220)) ([72147d3](https://github.com/substrate-labs/substrate2/commit/72147d385368e2bd302821c981dd75209aa87dcb))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * diagnostics bumped from 0.1.0 to 0.2.0

## [0.2.0](https://github.com/substrate-labs/substrate2/compare/scir-v0.1.0...scir-v0.2.0) (2023-07-23)


### Features

* **cache:** implement persistent caching ([#171](https://github.com/substrate-labs/substrate2/issues/171)) ([1f8ea24](https://github.com/substrate-labs/substrate2/commit/1f8ea24f805085392bfd1a2067bb8774d0fa4ae4))
* **merging:** add API for merging two SCIR libraries ([#183](https://github.com/substrate-labs/substrate2/issues/183)) ([a0006aa](https://github.com/substrate-labs/substrate2/commit/a0006aa4dbe62c2dda66eea306987e56eaabe181))
* **proc-macros:** codegen for schematic hard macros ([#191](https://github.com/substrate-labs/substrate2/issues/191)) ([50240b1](https://github.com/substrate-labs/substrate2/commit/50240b167876873c4133315d35298b44e8eeac51))
* **scir-instances:** allow Substrate users to instantiate raw SCIR instances ([#184](https://github.com/substrate-labs/substrate2/issues/184)) ([8fd5192](https://github.com/substrate-labs/substrate2/commit/8fd5192fd2017ab04e9e3220612d0a132702bb2e))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * uniquify bumped from 0.1.0 to 0.2.0

## 0.1.0 (2023-07-07)


### Features

* **api:** initial SCIR API definition ([#51](https://github.com/substrate-labs/substrate2/issues/51)) ([c175a48](https://github.com/substrate-labs/substrate2/commit/c175a484d63834787e25d46df416b6844d381686))
* **blackboxing:** support Substrate and SCIR blackboxes ([#135](https://github.com/substrate-labs/substrate2/issues/135)) ([049a598](https://github.com/substrate-labs/substrate2/commit/049a598e2b8d11228c63f03dc878fc4c56e036a6))
* **buses:** add support for 1D SCIR buses ([#57](https://github.com/substrate-labs/substrate2/issues/57)) ([162889c](https://github.com/substrate-labs/substrate2/commit/162889c6f3c89a575018274d8cda836eb8d0bbcf))
* **netlisting:** initial implementation of SPICE netlister ([#102](https://github.com/substrate-labs/substrate2/issues/102)) ([9125446](https://github.com/substrate-labs/substrate2/commit/91254466f76f5a89ee499fd2db13e63790a8379c))
* **node-naming:** create internal, named signals of any schematic type ([#118](https://github.com/substrate-labs/substrate2/issues/118)) ([1954bb9](https://github.com/substrate-labs/substrate2/commit/1954bb9a0b5e1663925b4a87fb8984b79cc0ede9))
* **pdks:** example instantiation of PDK-specific MOS ([#112](https://github.com/substrate-labs/substrate2/issues/112)) ([bbac00c](https://github.com/substrate-labs/substrate2/commit/bbac00cc6b48cb20b2761b8e6735065e9a024050))
* **schematics:** export Substrate schematics to SCIR ([#110](https://github.com/substrate-labs/substrate2/issues/110)) ([28115f0](https://github.com/substrate-labs/substrate2/commit/28115f0953400c38a82752e8358d0b267765282f))
* **simulation:** access nested nodes without strings in simulation ([#139](https://github.com/substrate-labs/substrate2/issues/139)) ([ed7989c](https://github.com/substrate-labs/substrate2/commit/ed7989cfb190528163a1722ae5fe3383ec3c4310))
* **simulation:** simplify SCIR paths for data access ([#143](https://github.com/substrate-labs/substrate2/issues/143)) ([d42e6f9](https://github.com/substrate-labs/substrate2/commit/d42e6f9b1d4236a9024d4a4b839319749033b8d3))
* **simulation:** support transient simulation in spectre ([#133](https://github.com/substrate-labs/substrate2/issues/133)) ([4605862](https://github.com/substrate-labs/substrate2/commit/460586252e3695ae32b0ab8d83b90023125d1a33))
* **tests:** add SCIR and SPICE netlister blackbox tests ([049a598](https://github.com/substrate-labs/substrate2/commit/049a598e2b8d11228c63f03dc878fc4c56e036a6))


### Bug Fixes

* **netlisting:** fix whitespace issues ([9125446](https://github.com/substrate-labs/substrate2/commit/91254466f76f5a89ee499fd2db13e63790a8379c))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * diagnostics bumped from 0.0.0 to 0.1.0
    * opacity bumped from 0.0.0 to 0.1.0
