# Changelog

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
