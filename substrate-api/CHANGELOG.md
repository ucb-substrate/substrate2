# Changelog

## 0.1.0 (2023-07-07)


### Features

* **blackboxing:** support Substrate and SCIR blackboxes ([#135](https://github.com/substrate-labs/substrate2/issues/135)) ([049a598](https://github.com/substrate-labs/substrate2/commit/049a598e2b8d11228c63f03dc878fc4c56e036a6))
* **cache:** add initial implementation of in-memory caching ([#150](https://github.com/substrate-labs/substrate2/issues/150)) ([2b26077](https://github.com/substrate-labs/substrate2/commit/2b26077d5d9726c2689d489ac428c67c039dbb1d))
* **corners:** add API for process corners ([#141](https://github.com/substrate-labs/substrate2/issues/141)) ([a61b15a](https://github.com/substrate-labs/substrate2/commit/a61b15a80851a6393aaa9da2db41e01a34f0ce5b))
* **custom-layout-io:** add way to derive custom layout IOs ([#117](https://github.com/substrate-labs/substrate2/issues/117)) ([61a8625](https://github.com/substrate-labs/substrate2/commit/61a86251978fde6e8d1095d33f197d5702d085cc))
* **docs:** add docs for layout IO ([#131](https://github.com/substrate-labs/substrate2/issues/131)) ([551d65e](https://github.com/substrate-labs/substrate2/commit/551d65e440ae3c7a9ccbe5d35a7ed5cd93d0d6b3))
* **docs:** add documentation for schematic API ([#128](https://github.com/substrate-labs/substrate2/issues/128)) ([b78b2e6](https://github.com/substrate-labs/substrate2/commit/b78b2e69c471cd14f95abeb5673277268c1ac4e8))
* **gds-export:** add GDS crate and utilities for accessing GDS layers ([#87](https://github.com/substrate-labs/substrate2/issues/87)) ([5cf11cd](https://github.com/substrate-labs/substrate2/commit/5cf11cd0ff80d637ca7210a603625a3b950cdaa4))
* **gds-export:** implement GDS export of Substrate cells ([#97](https://github.com/substrate-labs/substrate2/issues/97)) ([ae5ca3d](https://github.com/substrate-labs/substrate2/commit/ae5ca3d0356848eb8e080a7714667193bb9d28fb))
* **generation:** generating cells no longer blocks unless absolutely necessary ([#140](https://github.com/substrate-labs/substrate2/issues/140)) ([1d33dd0](https://github.com/substrate-labs/substrate2/commit/1d33dd066b7b63932d787a918c7d0fcc2846c1dd))
* **layer-api:** add layer IDs to shapes ([#85](https://github.com/substrate-labs/substrate2/issues/85)) ([df7064d](https://github.com/substrate-labs/substrate2/commit/df7064d0268d1ef7d2ec8bfb5b66434a9b19e819))
* **layer-api:** initial layer API and codegen ([#84](https://github.com/substrate-labs/substrate2/issues/84)) ([42bd94c](https://github.com/substrate-labs/substrate2/commit/42bd94c1f1d5e0b013a9b479bf100c68cf9de9a1))
* **layer-families:** implement layer families and clean up codegen ([#127](https://github.com/substrate-labs/substrate2/issues/127)) ([06f50b8](https://github.com/substrate-labs/substrate2/commit/06f50b8236ba40f405d7a5e20987a28e01f69f7c))
* **layout-io:** implement exporting layout IO to GDS ([#129](https://github.com/substrate-labs/substrate2/issues/129)) ([e9973a0](https://github.com/substrate-labs/substrate2/commit/e9973a07c10ba5867824ec32fcd55e5a0d4070fa))
* **layout-io:** initial layout port API implementation ([#111](https://github.com/substrate-labs/substrate2/issues/111)) ([ecc8838](https://github.com/substrate-labs/substrate2/commit/ecc8838678c98f137aca6f4955d89ba350540b44))
* **layout-ports:** initial implementation of layout port traits ([3c0527a](https://github.com/substrate-labs/substrate2/commit/3c0527a749b2ef7f3b42e46ce66d9f9bed3ff947))
* **mos:** add sky130pdk transistor blocks ([#126](https://github.com/substrate-labs/substrate2/issues/126)) ([3e9ee79](https://github.com/substrate-labs/substrate2/commit/3e9ee7935e030ca3e5c4d56f19ccafc27445a6f0))
* **node-naming:** cell builder maintains node name map ([#109](https://github.com/substrate-labs/substrate2/issues/109)) ([d9cf26e](https://github.com/substrate-labs/substrate2/commit/d9cf26ec78839e8709683b732bccb5c7221040b3))
* **node-naming:** create internal, named signals of any schematic type ([#118](https://github.com/substrate-labs/substrate2/issues/118)) ([1954bb9](https://github.com/substrate-labs/substrate2/commit/1954bb9a0b5e1663925b4a87fb8984b79cc0ede9))
* **node-naming:** priority system for determining node names ([#115](https://github.com/substrate-labs/substrate2/issues/115)) ([fe746e2](https://github.com/substrate-labs/substrate2/commit/fe746e26278625190a8658097ef92738d3ce5a41))
* **organization:** rename substrate to substrate_api, set up codegen crate ([#67](https://github.com/substrate-labs/substrate2/issues/67)) ([e07f099](https://github.com/substrate-labs/substrate2/commit/e07f09949551fd08e3f58b6ffb7d9a8c67b76ae9))
* **pdks:** example instantiation of PDK-specific MOS ([#112](https://github.com/substrate-labs/substrate2/issues/112)) ([bbac00c](https://github.com/substrate-labs/substrate2/commit/bbac00cc6b48cb20b2761b8e6735065e9a024050))
* **pdks:** implement `supported_pdks` macro and add examples ([#72](https://github.com/substrate-labs/substrate2/issues/72)) ([5f4312f](https://github.com/substrate-labs/substrate2/commit/5f4312f5220ae6023d78d8f4e585032147195a75))
* **schematic:** nested node and instance access ([#134](https://github.com/substrate-labs/substrate2/issues/134)) ([3d0e9ce](https://github.com/substrate-labs/substrate2/commit/3d0e9ce96b66072cd9b7982c582fa2d67ed8f406))
* **schematics:** add instantiate connected API function ([#122](https://github.com/substrate-labs/substrate2/issues/122)) ([b36eab6](https://github.com/substrate-labs/substrate2/commit/b36eab627e5a8d31f3bcf85e51e798cec2fd5fc0))
* **schematics:** allow indexing/slicing of arrays ([#100](https://github.com/substrate-labs/substrate2/issues/100)) ([363344e](https://github.com/substrate-labs/substrate2/commit/363344e7619513e8cba2f78241415b1044f537d8))
* **schematics:** export Substrate schematics to SCIR ([#110](https://github.com/substrate-labs/substrate2/issues/110)) ([28115f0](https://github.com/substrate-labs/substrate2/commit/28115f0953400c38a82752e8358d0b267765282f))
* **schematics:** implement `Array&lt;T&gt;` HW type ([#98](https://github.com/substrate-labs/substrate2/issues/98)) ([6625538](https://github.com/substrate-labs/substrate2/commit/662553899669d96c26305250afca09f1fc4b9b5b))
* **schematics:** implement `flip` for `Direction` ([#96](https://github.com/substrate-labs/substrate2/issues/96)) ([d03fa82](https://github.com/substrate-labs/substrate2/commit/d03fa8259fc9e485cdd6e2057295ed67ec624f3b))
* **schematics:** implement node naming trees, with codegen ([#105](https://github.com/substrate-labs/substrate2/issues/105)) ([5ef8e4b](https://github.com/substrate-labs/substrate2/commit/5ef8e4b8cdd20a274d1a4dadda8e186bed004763))
* **schematics:** implement proc macro to derive AnalogIo ([#99](https://github.com/substrate-labs/substrate2/issues/99)) ([2320c99](https://github.com/substrate-labs/substrate2/commit/2320c99e9852d4698c5b336de0af7ebe7cc94204))
* **schematics:** initial schematic API implementation ([#86](https://github.com/substrate-labs/substrate2/issues/86)) ([332d5d7](https://github.com/substrate-labs/substrate2/commit/332d5d7d4eb83583c8426ad63444d57bb466b8a5))
* **schematics:** support port directions ([#95](https://github.com/substrate-labs/substrate2/issues/95)) ([1c660f7](https://github.com/substrate-labs/substrate2/commit/1c660f71e31a86a24891744e9ded7cdfa5e3a66b))
* **scir:** uniquify names when exporting to SCIR ([#148](https://github.com/substrate-labs/substrate2/issues/148)) ([29c2f72](https://github.com/substrate-labs/substrate2/commit/29c2f729f5a205b144053b61c0d8c0ca2446071b))
* **simulation:** access nested nodes without strings in simulation ([#139](https://github.com/substrate-labs/substrate2/issues/139)) ([ed7989c](https://github.com/substrate-labs/substrate2/commit/ed7989cfb190528163a1722ae5fe3383ec3c4310))
* **simulation:** initial simulator API ([#80](https://github.com/substrate-labs/substrate2/issues/80)) ([249c557](https://github.com/substrate-labs/substrate2/commit/249c557e60f4dc140325b7f1a3d44b330a4a74bc))
* **simulation:** simplify SCIR paths for data access ([#143](https://github.com/substrate-labs/substrate2/issues/143)) ([d42e6f9](https://github.com/substrate-labs/substrate2/commit/d42e6f9b1d4236a9024d4a4b839319749033b8d3))
* **simulation:** support transient simulation in spectre ([#133](https://github.com/substrate-labs/substrate2/issues/133)) ([4605862](https://github.com/substrate-labs/substrate2/commit/460586252e3695ae32b0ab8d83b90023125d1a33))
* **simulation:** testbench schematic components ([#136](https://github.com/substrate-labs/substrate2/issues/136)) ([97e6b0f](https://github.com/substrate-labs/substrate2/commit/97e6b0ffd5ea7abd2a547952d5c963745854ed75))
* **tests:** add SCIR and SPICE netlister blackbox tests ([049a598](https://github.com/substrate-labs/substrate2/commit/049a598e2b8d11228c63f03dc878fc4c56e036a6))
* **transformed-view:** add TransformedView trait for lazy evaluation of collections ([70ba0bb](https://github.com/substrate-labs/substrate2/commit/70ba0bb23f403fad331071f196a128018f01eb61))
* **uniquify:** create uniquify crate for assigning unique names ([#147](https://github.com/substrate-labs/substrate2/issues/147)) ([d4b83be](https://github.com/substrate-labs/substrate2/commit/d4b83be335047052f0cf6ea2bddcdb64ce3141c4))
* **waveforms:** clocked digital waveform builder ([5f5beab](https://github.com/substrate-labs/substrate2/commit/5f5beabfc6084254b6eb2d30cdb44ac766fb152b))
* **waveforms:** edge and transition APIs ([5f5beab](https://github.com/substrate-labs/substrate2/commit/5f5beabfc6084254b6eb2d30cdb44ac766fb152b))
* **waveforms:** implement Waveform and WaveformRef ([5f5beab](https://github.com/substrate-labs/substrate2/commit/5f5beabfc6084254b6eb2d30cdb44ac766fb152b))
* **waveforms:** time-domain waveform API ([#145](https://github.com/substrate-labs/substrate2/issues/145)) ([5f5beab](https://github.com/substrate-labs/substrate2/commit/5f5beabfc6084254b6eb2d30cdb44ac766fb152b))


### Bug Fixes

* **node-naming:** implement flatten for name trees ([#108](https://github.com/substrate-labs/substrate2/issues/108)) ([45baa4d](https://github.com/substrate-labs/substrate2/commit/45baa4df8789433741691b28b772b4699709b9f1))
* **re-exports:** move all re-exports to substrate ([#132](https://github.com/substrate-labs/substrate2/issues/132)) ([8b3d867](https://github.com/substrate-labs/substrate2/commit/8b3d867c7b76a16f422a38a04f5643eb050f14e6))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * geometry bumped from 0.1.0 to 0.2.0
    * gds bumped from 0.0.0 to 0.1.0
    * opacity bumped from 0.0.0 to 0.1.0
    * scir bumped from 0.0.0 to 0.1.0
    * pathtree bumped from 0.0.0 to 0.1.0
    * uniquify bumped from 0.0.0 to 0.1.0
  * dev-dependencies
    * substrate bumped from 0.0.0 to 0.1.0
