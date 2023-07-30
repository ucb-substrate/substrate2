# Changelog

* The following workspace dependencies were updated
  * dependencies
    * codegen bumped from 0.1.0 to 0.1.1
    * substrate_api bumped from 0.1.0 to 0.1.1

## [0.3.0](https://github.com/substrate-labs/substrate2/compare/substrate-v0.2.0...substrate-v0.3.0) (2023-07-23)


### Features

* **cache-config:** allow configuration of cache via config files ([#192](https://github.com/substrate-labs/substrate2/issues/192)) ([0461402](https://github.com/substrate-labs/substrate2/commit/0461402edfc1ec0886bbb25cf5471ee8480754fc))
* **cache:** implement persistent caching ([#171](https://github.com/substrate-labs/substrate2/issues/171)) ([1f8ea24](https://github.com/substrate-labs/substrate2/commit/1f8ea24f805085392bfd1a2067bb8774d0fa4ae4))
* **codegen:** implement derive proc macro for layout hard macros ([#200](https://github.com/substrate-labs/substrate2/issues/200)) ([5138224](https://github.com/substrate-labs/substrate2/commit/5138224013f537e678dfb20204e964852ed40ccb))
* **executors:** executor API and local executor implementation ([#161](https://github.com/substrate-labs/substrate2/issues/161)) ([c372d51](https://github.com/substrate-labs/substrate2/commit/c372d511e1c67ad976dc86958c87e9ad13bdfde4))
* **executors:** LSF (bsub) executor implementation ([#162](https://github.com/substrate-labs/substrate2/issues/162)) ([ff8404a](https://github.com/substrate-labs/substrate2/commit/ff8404abf75e6d6a4c82109adde0bcac48b6f33f))
* **gds-import:** implement GDS to RawCell importer ([#196](https://github.com/substrate-labs/substrate2/issues/196)) ([fc37eeb](https://github.com/substrate-labs/substrate2/commit/fc37eeb6bac10779491b98bcadcc0eeaeb7d8ec5))
* **gds:** gds reexport test ([#199](https://github.com/substrate-labs/substrate2/issues/199)) ([93d3cd5](https://github.com/substrate-labs/substrate2/commit/93d3cd555c1cb4a76a8845f4401e98d327b5d674))
* **pdks:** PDKs store the names of schematic primitives ([#185](https://github.com/substrate-labs/substrate2/issues/185)) ([3446ba8](https://github.com/substrate-labs/substrate2/commit/3446ba869f564f844b39ee524b52106954a293c5))
* **proc-macros:** codegen for schematic hard macros ([#191](https://github.com/substrate-labs/substrate2/issues/191)) ([50240b1](https://github.com/substrate-labs/substrate2/commit/50240b167876873c4133315d35298b44e8eeac51))
* **remote-cache:** add initial implementation of remote-cache ([#166](https://github.com/substrate-labs/substrate2/issues/166)) ([7d90aab](https://github.com/substrate-labs/substrate2/commit/7d90aab47c282cf90e814ffce357a1e694c0c357))
* **scir-instances:** allow Substrate users to instantiate raw SCIR instances ([#184](https://github.com/substrate-labs/substrate2/issues/184)) ([8fd5192](https://github.com/substrate-labs/substrate2/commit/8fd5192fd2017ab04e9e3220612d0a132702bb2e))
* **simulation:** proc macros for implementing Supports on tuples ([#163](https://github.com/substrate-labs/substrate2/issues/163)) ([bf77832](https://github.com/substrate-labs/substrate2/commit/bf778329d6e9fd317bea789d093c4c7d8790f5ac))
* **tiling:** array and grid tiling API ([#201](https://github.com/substrate-labs/substrate2/issues/201)) ([b3b7c2b](https://github.com/substrate-labs/substrate2/commit/b3b7c2bfb7ba72198872d0f08ded3e0bc757479d))


### Bug Fixes

* **ci:** fix doc tests for substrate crate ([#158](https://github.com/substrate-labs/substrate2/issues/158)) ([d7e9437](https://github.com/substrate-labs/substrate2/commit/d7e943734b1eadfe64deabb7602f5bbf41cd8806))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * config bumped from 0.1.0 to 0.2.0
    * cache bumped from 0.1.0 to 0.2.0
    * codegen bumped from 0.2.0 to 0.3.0
    * geometry bumped from 0.2.0 to 0.3.0
    * gds bumped from 0.1.0 to 0.2.0
    * scir bumped from 0.1.0 to 0.2.0
    * uniquify bumped from 0.1.0 to 0.2.0
    * spice bumped from 0.0.0 to 0.1.0
  * dev-dependencies
    * sky130pdk bumped from <=0.2.0 to <=0.3.0

## [0.2.0](https://github.com/substrate-labs/substrate2/compare/substrate-v0.1.1...substrate-v0.2.0) (2023-07-07)


### Features

* **reorg:** move substrate-api into substrate ([#155](https://github.com/substrate-labs/substrate2/issues/155)) ([e902a1b](https://github.com/substrate-labs/substrate2/commit/e902a1b603cca6c719770c5cd742e081bfd33e51))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * codegen bumped from 0.1.0 to 0.2.0

## 0.1.0 (2023-07-07)


### Features

* **blackboxing:** support Substrate and SCIR blackboxes ([#135](https://github.com/substrate-labs/substrate2/issues/135)) ([049a598](https://github.com/substrate-labs/substrate2/commit/049a598e2b8d11228c63f03dc878fc4c56e036a6))
* **docs:** add code examples to documentation ([#65](https://github.com/substrate-labs/substrate2/issues/65)) ([bfafd05](https://github.com/substrate-labs/substrate2/commit/bfafd050c1b68d2e9e29e760ca3ff939e26aaeca))
* **layout-api:** initial implementation of layout API ([#61](https://github.com/substrate-labs/substrate2/issues/61)) ([c4cdac7](https://github.com/substrate-labs/substrate2/commit/c4cdac728fd4d4ef5defb97b3c1e1660ee78d672))
* **mos:** add sky130pdk transistor blocks ([#126](https://github.com/substrate-labs/substrate2/issues/126)) ([3e9ee79](https://github.com/substrate-labs/substrate2/commit/3e9ee7935e030ca3e5c4d56f19ccafc27445a6f0))
* **mos:** add standard 4-terminal MosIo ([8678716](https://github.com/substrate-labs/substrate2/commit/86787160c49a1ac7c011d08ce1b9d7851bdfa0d8))
* **organization:** rename substrate to substrate_api, set up codegen crate ([#67](https://github.com/substrate-labs/substrate2/issues/67)) ([e07f099](https://github.com/substrate-labs/substrate2/commit/e07f09949551fd08e3f58b6ffb7d9a8c67b76ae9))
* **pdk:** add PDK trait and update context ([#68](https://github.com/substrate-labs/substrate2/issues/68)) ([a8fbd14](https://github.com/substrate-labs/substrate2/commit/a8fbd14a4b81e504c781e0656edce81853039afb))
* **pdks:** implement `supported_pdks` macro and add examples ([#72](https://github.com/substrate-labs/substrate2/issues/72)) ([5f4312f](https://github.com/substrate-labs/substrate2/commit/5f4312f5220ae6023d78d8f4e585032147195a75))
* **proc-macros:** allow missing docs on generated structs ([8678716](https://github.com/substrate-labs/substrate2/commit/86787160c49a1ac7c011d08ce1b9d7851bdfa0d8))
* **proc-macros:** macros respect field and struct visibilities ([8678716](https://github.com/substrate-labs/substrate2/commit/86787160c49a1ac7c011d08ce1b9d7851bdfa0d8))
* **proc-macros:** proc macros find substrate crate location ([#125](https://github.com/substrate-labs/substrate2/issues/125)) ([8678716](https://github.com/substrate-labs/substrate2/commit/86787160c49a1ac7c011d08ce1b9d7851bdfa0d8))
* **simulation:** simplify SCIR paths for data access ([#143](https://github.com/substrate-labs/substrate2/issues/143)) ([d42e6f9](https://github.com/substrate-labs/substrate2/commit/d42e6f9b1d4236a9024d4a4b839319749033b8d3))
* **simulation:** support transient simulation in spectre ([#133](https://github.com/substrate-labs/substrate2/issues/133)) ([4605862](https://github.com/substrate-labs/substrate2/commit/460586252e3695ae32b0ab8d83b90023125d1a33))
* **simulation:** testbench schematic components ([#136](https://github.com/substrate-labs/substrate2/issues/136)) ([97e6b0f](https://github.com/substrate-labs/substrate2/commit/97e6b0ffd5ea7abd2a547952d5c963745854ed75))
* **tests:** add SCIR and SPICE netlister blackbox tests ([049a598](https://github.com/substrate-labs/substrate2/commit/049a598e2b8d11228c63f03dc878fc4c56e036a6))


### Bug Fixes

* **deps:** fix dependencies and documentation ([#66](https://github.com/substrate-labs/substrate2/issues/66)) ([a60ffc6](https://github.com/substrate-labs/substrate2/commit/a60ffc6c5501200d56a6e76db0c1c2f7ef9cd086))
* **re-exports:** move all re-exports to substrate ([#132](https://github.com/substrate-labs/substrate2/issues/132)) ([8b3d867](https://github.com/substrate-labs/substrate2/commit/8b3d867c7b76a16f422a38a04f5643eb050f14e6))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * geometry bumped from 0.1.0 to 0.2.0
    * codegen bumped from 0.0.0 to 0.1.0
    * substrate_api bumped from 0.0.0 to 0.1.0
    * scir bumped from 0.0.0 to 0.1.0
