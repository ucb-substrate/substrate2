# Changelog

* The following workspace dependencies were updated
  * dependencies
    * substrate bumped from 0.1.0 to 0.1.1

## [0.4.0](https://github.com/substrate-labs/substrate2/compare/spectre-v0.3.0...spectre-v0.4.0) (2023-08-04)


### Features

* **includes:** add support for including sections in spectre ([#205](https://github.com/substrate-labs/substrate2/issues/205)) ([8522ff4](https://github.com/substrate-labs/substrate2/commit/8522ff4241755d4c194bacb893765e608122814e))
* **io:** composable port directions and runtime connection checking ([#231](https://github.com/substrate-labs/substrate2/issues/231)) ([e1e367a](https://github.com/substrate-labs/substrate2/commit/e1e367a2b8940319cb4f804888746a094f06e161))
* **parameters:** substrate schematic primitives support parameters ([#233](https://github.com/substrate-labs/substrate2/issues/233)) ([5dabcb2](https://github.com/substrate-labs/substrate2/commit/5dabcb270cab0d259b7301d67f77de6d55261092))
* **primitives:** support parameters in SCIR primitive devices ([#232](https://github.com/substrate-labs/substrate2/issues/232)) ([a8f5b45](https://github.com/substrate-labs/substrate2/commit/a8f5b45a00b77d050f6a812c469e19da3305e064))
* **repo:** reorganize repo ([#207](https://github.com/substrate-labs/substrate2/issues/207)) ([54a6b43](https://github.com/substrate-labs/substrate2/commit/54a6b43079d283a29bc0aa9e18dc6230b56fa385))
* **save-api:** add typed API for saving arbitrary signals ([#228](https://github.com/substrate-labs/substrate2/issues/228)) ([046be02](https://github.com/substrate-labs/substrate2/commit/046be02acbedc7fa2bb4896b92ec17babd80eee5))
* **schematics:** blackboxes can reference nodes ([#208](https://github.com/substrate-labs/substrate2/issues/208)) ([d998b4a](https://github.com/substrate-labs/substrate2/commit/d998b4a133d47d0123768dfb3c27f8ee32ed9db9))
* **schematics:** rename `HasSchematic` and `HasSchematicImpl` ([#226](https://github.com/substrate-labs/substrate2/issues/226)) ([a2b9c78](https://github.com/substrate-labs/substrate2/commit/a2b9c78ea6ff56983e9a02aeafe655e92852c264))
* **spectre:** vsource uses primitives instead of being blackboxed ([5dabcb2](https://github.com/substrate-labs/substrate2/commit/5dabcb270cab0d259b7301d67f77de6d55261092))


### Bug Fixes

* **deps:** update rust crate rust_decimal to 1.31 ([#219](https://github.com/substrate-labs/substrate2/issues/219)) ([6f596d5](https://github.com/substrate-labs/substrate2/commit/6f596d5c46dc1bf045a1b8a5ef727adbc3b147cf))
* **deps:** update rust crate rust_decimal_macros to 1.31 ([#220](https://github.com/substrate-labs/substrate2/issues/220)) ([72147d3](https://github.com/substrate-labs/substrate2/commit/72147d385368e2bd302821c981dd75209aa87dcb))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * cache bumped from 0.2.0 to 0.2.1
    * scir bumped from 0.2.0 to 0.3.0
    * substrate bumped from 0.3.0 to 0.4.0

## [0.3.0](https://github.com/substrate-labs/substrate2/compare/spectre-v0.2.0...spectre-v0.3.0) (2023-07-23)


### Features

* **cache:** implement persistent caching ([#171](https://github.com/substrate-labs/substrate2/issues/171)) ([1f8ea24](https://github.com/substrate-labs/substrate2/commit/1f8ea24f805085392bfd1a2067bb8774d0fa4ae4))
* **codegen:** implement derive proc macro for layout hard macros ([#200](https://github.com/substrate-labs/substrate2/issues/200)) ([5138224](https://github.com/substrate-labs/substrate2/commit/5138224013f537e678dfb20204e964852ed40ccb))
* **executors:** executor API and local executor implementation ([#161](https://github.com/substrate-labs/substrate2/issues/161)) ([c372d51](https://github.com/substrate-labs/substrate2/commit/c372d511e1c67ad976dc86958c87e9ad13bdfde4))
* **windows:** fix issues for windows ([#197](https://github.com/substrate-labs/substrate2/issues/197)) ([008b607](https://github.com/substrate-labs/substrate2/commit/008b607b2c21c14ac3106dca6eb74d806131ef8f))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * scir bumped from 0.1.0 to 0.2.0
    * cache bumped from 0.1.0 to 0.2.0
    * substrate bumped from 0.2.0 to 0.3.0

## [0.2.0](https://github.com/substrate-labs/substrate2/compare/spectre-v0.1.1...spectre-v0.2.0) (2023-07-07)


### Features

* **reorg:** move substrate-api into substrate ([#155](https://github.com/substrate-labs/substrate2/issues/155)) ([e902a1b](https://github.com/substrate-labs/substrate2/commit/e902a1b603cca6c719770c5cd742e081bfd33e51))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * substrate bumped from 0.1.1 to 0.2.0

## 0.1.0 (2023-07-07)


### Features

* **blackboxing:** support Substrate and SCIR blackboxes ([#135](https://github.com/substrate-labs/substrate2/issues/135)) ([049a598](https://github.com/substrate-labs/substrate2/commit/049a598e2b8d11228c63f03dc878fc4c56e036a6))
* **corners:** add API for process corners ([#141](https://github.com/substrate-labs/substrate2/issues/141)) ([a61b15a](https://github.com/substrate-labs/substrate2/commit/a61b15a80851a6393aaa9da2db41e01a34f0ce5b))
* **inverter-design:** initial inverter design logic ([#144](https://github.com/substrate-labs/substrate2/issues/144)) ([911a376](https://github.com/substrate-labs/substrate2/commit/911a3768bf730d7fd134310d500560873f008783))
* **netlisting:** implement basic Spectre netlister ([#130](https://github.com/substrate-labs/substrate2/issues/130)) ([506837f](https://github.com/substrate-labs/substrate2/commit/506837fca0f1964ebc622df970575b321456ab68))
* **scir:** uniquify names when exporting to SCIR ([#148](https://github.com/substrate-labs/substrate2/issues/148)) ([29c2f72](https://github.com/substrate-labs/substrate2/commit/29c2f729f5a205b144053b61c0d8c0ca2446071b))
* **simulation:** access nested nodes without strings in simulation ([#139](https://github.com/substrate-labs/substrate2/issues/139)) ([ed7989c](https://github.com/substrate-labs/substrate2/commit/ed7989cfb190528163a1722ae5fe3383ec3c4310))
* **simulation:** simplify SCIR paths for data access ([#143](https://github.com/substrate-labs/substrate2/issues/143)) ([d42e6f9](https://github.com/substrate-labs/substrate2/commit/d42e6f9b1d4236a9024d4a4b839319749033b8d3))
* **simulation:** support transient simulation in spectre ([#133](https://github.com/substrate-labs/substrate2/issues/133)) ([4605862](https://github.com/substrate-labs/substrate2/commit/460586252e3695ae32b0ab8d83b90023125d1a33))
* **simulation:** testbench schematic components ([#136](https://github.com/substrate-labs/substrate2/issues/136)) ([97e6b0f](https://github.com/substrate-labs/substrate2/commit/97e6b0ffd5ea7abd2a547952d5c963745854ed75))
* **tests:** add SCIR and SPICE netlister blackbox tests ([049a598](https://github.com/substrate-labs/substrate2/commit/049a598e2b8d11228c63f03dc878fc4c56e036a6))


### Bug Fixes

* **netlisting:** update release manifests ([506837f](https://github.com/substrate-labs/substrate2/commit/506837fca0f1964ebc622df970575b321456ab68))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * scir bumped from 0.0.0 to 0.1.0
    * substrate bumped from 0.0.0 to 0.1.0
    * opacity bumped from 0.0.0 to 0.1.0
