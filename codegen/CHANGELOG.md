# Changelog

## 0.1.0 (2023-07-07)


### Features

* **custom-layout-io:** add way to derive custom layout IOs ([#117](https://github.com/substrate-labs/substrate2/issues/117)) ([61a8625](https://github.com/substrate-labs/substrate2/commit/61a86251978fde6e8d1095d33f197d5702d085cc))
* **docs:** add docs for layout IO ([#131](https://github.com/substrate-labs/substrate2/issues/131)) ([551d65e](https://github.com/substrate-labs/substrate2/commit/551d65e440ae3c7a9ccbe5d35a7ed5cd93d0d6b3))
* **gds-export:** add GDS crate and utilities for accessing GDS layers ([#87](https://github.com/substrate-labs/substrate2/issues/87)) ([5cf11cd](https://github.com/substrate-labs/substrate2/commit/5cf11cd0ff80d637ca7210a603625a3b950cdaa4))
* **gds-export:** implement GDS export of Substrate cells ([#97](https://github.com/substrate-labs/substrate2/issues/97)) ([ae5ca3d](https://github.com/substrate-labs/substrate2/commit/ae5ca3d0356848eb8e080a7714667193bb9d28fb))
* **layer-api:** add layer IDs to shapes ([#85](https://github.com/substrate-labs/substrate2/issues/85)) ([df7064d](https://github.com/substrate-labs/substrate2/commit/df7064d0268d1ef7d2ec8bfb5b66434a9b19e819))
* **layer-api:** initial layer API and codegen ([#84](https://github.com/substrate-labs/substrate2/issues/84)) ([42bd94c](https://github.com/substrate-labs/substrate2/commit/42bd94c1f1d5e0b013a9b479bf100c68cf9de9a1))
* **layer-families:** implement layer families and clean up codegen ([#127](https://github.com/substrate-labs/substrate2/issues/127)) ([06f50b8](https://github.com/substrate-labs/substrate2/commit/06f50b8236ba40f405d7a5e20987a28e01f69f7c))
* **layout-io:** initial layout port API implementation ([#111](https://github.com/substrate-labs/substrate2/issues/111)) ([ecc8838](https://github.com/substrate-labs/substrate2/commit/ecc8838678c98f137aca6f4955d89ba350540b44))
* **layout-ports:** initial implementation of layout port traits ([3c0527a](https://github.com/substrate-labs/substrate2/commit/3c0527a749b2ef7f3b42e46ce66d9f9bed3ff947))
* **mos:** add standard 4-terminal MosIo ([8678716](https://github.com/substrate-labs/substrate2/commit/86787160c49a1ac7c011d08ce1b9d7851bdfa0d8))
* **organization:** rename substrate to substrate_api, set up codegen crate ([#67](https://github.com/substrate-labs/substrate2/issues/67)) ([e07f099](https://github.com/substrate-labs/substrate2/commit/e07f09949551fd08e3f58b6ffb7d9a8c67b76ae9))
* **pdk:** add PDK trait and update context ([#68](https://github.com/substrate-labs/substrate2/issues/68)) ([a8fbd14](https://github.com/substrate-labs/substrate2/commit/a8fbd14a4b81e504c781e0656edce81853039afb))
* **pdks:** implement `supported_pdks` macro and add examples ([#72](https://github.com/substrate-labs/substrate2/issues/72)) ([5f4312f](https://github.com/substrate-labs/substrate2/commit/5f4312f5220ae6023d78d8f4e585032147195a75))
* **proc-macros:** add derive Block proc macro ([#151](https://github.com/substrate-labs/substrate2/issues/151)) ([e2c2f02](https://github.com/substrate-labs/substrate2/commit/e2c2f02771611ad4a79b3c9516fa1defabc20a66))
* **proc-macros:** allow missing docs on generated structs ([8678716](https://github.com/substrate-labs/substrate2/commit/86787160c49a1ac7c011d08ce1b9d7851bdfa0d8))
* **proc-macros:** macros respect field and struct visibilities ([8678716](https://github.com/substrate-labs/substrate2/commit/86787160c49a1ac7c011d08ce1b9d7851bdfa0d8))
* **proc-macros:** proc macros find substrate crate location ([#125](https://github.com/substrate-labs/substrate2/issues/125)) ([8678716](https://github.com/substrate-labs/substrate2/commit/86787160c49a1ac7c011d08ce1b9d7851bdfa0d8))
* **schematic:** nested node and instance access ([#134](https://github.com/substrate-labs/substrate2/issues/134)) ([3d0e9ce](https://github.com/substrate-labs/substrate2/commit/3d0e9ce96b66072cd9b7982c582fa2d67ed8f406))
* **schematics:** implement node naming trees, with codegen ([#105](https://github.com/substrate-labs/substrate2/issues/105)) ([5ef8e4b](https://github.com/substrate-labs/substrate2/commit/5ef8e4b8cdd20a274d1a4dadda8e186bed004763))
* **schematics:** implement proc macro to derive AnalogIo ([#99](https://github.com/substrate-labs/substrate2/issues/99)) ([2320c99](https://github.com/substrate-labs/substrate2/commit/2320c99e9852d4698c5b336de0af7ebe7cc94204))


### Bug Fixes

* **deps:** update rust crate syn to v2 ([#79](https://github.com/substrate-labs/substrate2/issues/79)) ([eee3593](https://github.com/substrate-labs/substrate2/commit/eee35938247f2660c15b0165b6ba3d609d7091b8))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * substrate_api bumped from 0.0.0 to 0.1.0
  * dev-dependencies
    * substrate bumped from 0.0.0 to 0.1.0
