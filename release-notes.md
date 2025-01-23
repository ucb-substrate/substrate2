:robot: I have created a release *beep* *boop*
---


<details><summary>cache: 0.8.0</summary>

## [0.8.0](https://github.com/ucb-substrate/substrate2/compare/cache-v0.7.0...cache-v0.8.0) (2025-01-23)


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
</details>

<details><summary>cdl2spice: 0.3.0</summary>

## [0.3.0](https://github.com/ucb-substrate/substrate2/compare/cdl2spice-v0.2.0...cdl2spice-v0.3.0) (2025-01-23)


### Features

* **cdl2spice:** add CDL to SPICE conversion command line tool ([#420](https://github.com/ucb-substrate/substrate2/issues/420)) ([1edb23a](https://github.com/ucb-substrate/substrate2/commit/1edb23a7bbd45d96bbb1c11418eb0d0843b7138b))
* **tests:** reorganize tests and documentation ([#464](https://github.com/ucb-substrate/substrate2/issues/464)) ([928b9b7](https://github.com/ucb-substrate/substrate2/commit/928b9b7c45dc334ca11d86e4564edc58bf6db6f2))


### Bug Fixes

* **release:** change cdl2spice version to 0.0.0 ([#421](https://github.com/ucb-substrate/substrate2/issues/421)) ([fc3ee67](https://github.com/ucb-substrate/substrate2/commit/fc3ee67735419239de3687929947df82a4b6b5cb))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * scir bumped from 0.9.0 to 0.10.0
    * spice bumped from 0.9.0 to 0.10.0
</details>

<details><summary>codegen: 0.11.0</summary>

## [0.11.0](https://github.com/ucb-substrate/substrate2/compare/codegen-v0.10.0...codegen-v0.11.0) (2025-01-23)


### Features

* **ac:** implement Save for Substrate types in Spectre AC sim, fix lints ([#471](https://github.com/ucb-substrate/substrate2/issues/471)) ([9825520](https://github.com/ucb-substrate/substrate2/commit/98255207569cc00bd9ddc35419c2df1e48f1999c))
* **atoll:** implement first cut ATOLL implementation ([#357](https://github.com/ucb-substrate/substrate2/issues/357)) ([372b927](https://github.com/ucb-substrate/substrate2/commit/372b9275c9d9c5cd58603f5a462a5e4b66b64cf7))
* **cadence:** add support for Pegasus and Quantus ([#462](https://github.com/ucb-substrate/substrate2/issues/462)) ([953e4cb](https://github.com/ucb-substrate/substrate2/commit/953e4cb761c510668f65f4825f1be3914db45e3c))
* **codegen:** codegen for layout types, example layouts ([#469](https://github.com/ucb-substrate/substrate2/issues/469)) ([255af05](https://github.com/ucb-substrate/substrate2/commit/255af05657c01fcb0b4ff1e6eb0a54244dfeca32))
* **codegen:** derive Block macro adds required trait bounds by default ([#249](https://github.com/ucb-substrate/substrate2/issues/249)) ([892bef5](https://github.com/ucb-substrate/substrate2/commit/892bef585548264e3fcdcc2e6523a2321c6c6897))
* **codegen:** derive macro for implementing FromSaved ([#243](https://github.com/ucb-substrate/substrate2/issues/243)) ([48acae0](https://github.com/ucb-substrate/substrate2/commit/48acae0fb8915c4f968223268c92077f2deda979))
* **codegen:** implement derive proc macro for layout hard macros ([#200](https://github.com/ucb-substrate/substrate2/issues/200)) ([5138224](https://github.com/ucb-substrate/substrate2/commit/5138224013f537e678dfb20204e964852ed40ccb))
* **codegen:** insert appropriate bounds in Io, SchematicType, LayoutType proc macros ([#251](https://github.com/ucb-substrate/substrate2/issues/251)) ([33dcc79](https://github.com/ucb-substrate/substrate2/commit/33dcc797fdbeb21ad046093e655acf965fd99321))
* **corners:** require specifying corner by default ([#221](https://github.com/ucb-substrate/substrate2/issues/221)) ([4c2c3e4](https://github.com/ucb-substrate/substrate2/commit/4c2c3e4a3cd8b7e68921baf3af8b87f1da048936))
* **custom-layout-io:** add way to derive custom layout IOs ([#117](https://github.com/ucb-substrate/substrate2/issues/117)) ([61a8625](https://github.com/ucb-substrate/substrate2/commit/61a86251978fde6e8d1095d33f197d5702d085cc))
* **docs:** add docs for layout IO ([#131](https://github.com/ucb-substrate/substrate2/issues/131)) ([551d65e](https://github.com/ucb-substrate/substrate2/commit/551d65e440ae3c7a9ccbe5d35a7ed5cd93d0d6b3))
* **docs:** fix user docs and update dev docs ([#480](https://github.com/ucb-substrate/substrate2/issues/480)) ([f727a1e](https://github.com/ucb-substrate/substrate2/commit/f727a1e7bd2a795ace1c51c3d6e02f3673d07a29))
* **docs:** reorganize docs and add code snippets ([#216](https://github.com/ucb-substrate/substrate2/issues/216)) ([d7c457d](https://github.com/ucb-substrate/substrate2/commit/d7c457d4e5c1d4846549a0e6df958243042285db))
* **docs:** update tutorials and revamp documentation website ([#315](https://github.com/ucb-substrate/substrate2/issues/315)) ([49bdf7f](https://github.com/ucb-substrate/substrate2/commit/49bdf7ff61e2fdbf19022697d518ad7fbafb465f))
* **docs:** versioned documentation between HEAD and release ([#470](https://github.com/ucb-substrate/substrate2/issues/470)) ([968182b](https://github.com/ucb-substrate/substrate2/commit/968182bf8f8d8b4cf923c0fd66f1ca1b32b12b16))
* **gds-export:** add GDS crate and utilities for accessing GDS layers ([#87](https://github.com/ucb-substrate/substrate2/issues/87)) ([5cf11cd](https://github.com/ucb-substrate/substrate2/commit/5cf11cd0ff80d637ca7210a603625a3b950cdaa4))
* **gds-export:** implement GDS export of Substrate cells ([#97](https://github.com/ucb-substrate/substrate2/issues/97)) ([ae5ca3d](https://github.com/ucb-substrate/substrate2/commit/ae5ca3d0356848eb8e080a7714667193bb9d28fb))
* **gds-import:** implement GDS to RawCell importer ([#196](https://github.com/ucb-substrate/substrate2/issues/196)) ([fc37eeb](https://github.com/ucb-substrate/substrate2/commit/fc37eeb6bac10779491b98bcadcc0eeaeb7d8ec5))
* **impl-dispatch:** remove impl dispatch in favor of trait bounds ([#283](https://github.com/ucb-substrate/substrate2/issues/283)) ([d954115](https://github.com/ucb-substrate/substrate2/commit/d9541152db52aebde928e41c0d800453e906d62b))
* **layer-api:** add layer IDs to shapes ([#85](https://github.com/ucb-substrate/substrate2/issues/85)) ([df7064d](https://github.com/ucb-substrate/substrate2/commit/df7064d0268d1ef7d2ec8bfb5b66434a9b19e819))
* **layer-api:** initial layer API and codegen ([#84](https://github.com/ucb-substrate/substrate2/issues/84)) ([42bd94c](https://github.com/ucb-substrate/substrate2/commit/42bd94c1f1d5e0b013a9b479bf100c68cf9de9a1))
* **layer-families:** implement layer families and clean up codegen ([#127](https://github.com/ucb-substrate/substrate2/issues/127)) ([06f50b8](https://github.com/ucb-substrate/substrate2/commit/06f50b8236ba40f405d7a5e20987a28e01f69f7c))
* **layout-io:** initial layout port API implementation ([#111](https://github.com/ucb-substrate/substrate2/issues/111)) ([ecc8838](https://github.com/ucb-substrate/substrate2/commit/ecc8838678c98f137aca6f4955d89ba350540b44))
* **layout-ports:** initial implementation of layout port traits ([3c0527a](https://github.com/ucb-substrate/substrate2/commit/3c0527a749b2ef7f3b42e46ce66d9f9bed3ff947))
* **layout:** rename `HasLayout` and `HasLayoutImpl` ([#227](https://github.com/ucb-substrate/substrate2/issues/227)) ([2cf1f7d](https://github.com/ucb-substrate/substrate2/commit/2cf1f7d435549df26ff15370e7324e9df76e0e4f))
* **layout:** simplified layout API, LayIR integration ([#459](https://github.com/ucb-substrate/substrate2/issues/459)) ([183d347](https://github.com/ucb-substrate/substrate2/commit/183d347c19e6fe98cf870be4716e7249f23bd423))
* **macros:** refactor derive NestedData, start organizing tests ([#477](https://github.com/ucb-substrate/substrate2/issues/477)) ([aca48ef](https://github.com/ucb-substrate/substrate2/commit/aca48ef7a49c959e35ec4614345a55e667ff5146))
* **macros:** refactor macro reexports ([#250](https://github.com/ucb-substrate/substrate2/issues/250)) ([a332717](https://github.com/ucb-substrate/substrate2/commit/a332717e549fdea50306067e1c92dc60293aed4c))
* **macros:** support ref, mut ref, and owned receiver styles ([#468](https://github.com/ucb-substrate/substrate2/issues/468)) ([b285476](https://github.com/ucb-substrate/substrate2/commit/b285476d3ac378522a1b40ae4e22a69f5e580fda))
* **montecarlo:** add Monte Carlo simulation support to Spectre plugin ([#347](https://github.com/ucb-substrate/substrate2/issues/347)) ([cc9dfe4](https://github.com/ucb-substrate/substrate2/commit/cc9dfe42db5be1a8aaeaf3fb81992a0ad7251ef8))
* **mos:** add standard 4-terminal MosIo ([8678716](https://github.com/ucb-substrate/substrate2/commit/86787160c49a1ac7c011d08ce1b9d7851bdfa0d8))
* **netlister:** reduce duplicate code between spectre and SPICE netlisters ([#261](https://github.com/ucb-substrate/substrate2/issues/261)) ([5ba3623](https://github.com/ucb-substrate/substrate2/commit/5ba36230e653e4dc77819c5c50b527311768cd83))
* **organization:** rename substrate to substrate_api, set up codegen crate ([#67](https://github.com/ucb-substrate/substrate2/issues/67)) ([e07f099](https://github.com/ucb-substrate/substrate2/commit/e07f09949551fd08e3f58b6ffb7d9a8c67b76ae9))
* **parameters:** substrate schematic primitives support parameters ([#233](https://github.com/ucb-substrate/substrate2/issues/233)) ([5dabcb2](https://github.com/ucb-substrate/substrate2/commit/5dabcb270cab0d259b7301d67f77de6d55261092))
* **pdk:** add PDK trait and update context ([#68](https://github.com/ucb-substrate/substrate2/issues/68)) ([a8fbd14](https://github.com/ucb-substrate/substrate2/commit/a8fbd14a4b81e504c781e0656edce81853039afb))
* **pdk:** remove `PdkData` object to clean up interface ([#218](https://github.com/ucb-substrate/substrate2/issues/218)) ([1dd166a](https://github.com/ucb-substrate/substrate2/commit/1dd166a8f23e7b3c011c01b5c8527b8c5494ddea))
* **pdks:** implement `supported_pdks` macro and add examples ([#72](https://github.com/ucb-substrate/substrate2/issues/72)) ([5f4312f](https://github.com/ucb-substrate/substrate2/commit/5f4312f5220ae6023d78d8f4e585032147195a75))
* **primitives:** revamp schematic primitives ([#291](https://github.com/ucb-substrate/substrate2/issues/291)) ([e5ba06a](https://github.com/ucb-substrate/substrate2/commit/e5ba06ab10008b72e78397ad70781caa6bc61791))
* **proc-macros:** add derive Block proc macro ([#151](https://github.com/ucb-substrate/substrate2/issues/151)) ([e2c2f02](https://github.com/ucb-substrate/substrate2/commit/e2c2f02771611ad4a79b3c9516fa1defabc20a66))
* **proc-macros:** allow missing docs on generated structs ([8678716](https://github.com/ucb-substrate/substrate2/commit/86787160c49a1ac7c011d08ce1b9d7851bdfa0d8))
* **proc-macros:** codegen for schematic hard macros ([#191](https://github.com/ucb-substrate/substrate2/issues/191)) ([50240b1](https://github.com/ucb-substrate/substrate2/commit/50240b167876873c4133315d35298b44e8eeac51))
* **proc-macros:** derive macros for geometry traits ([#164](https://github.com/ucb-substrate/substrate2/issues/164)) ([a86074a](https://github.com/ucb-substrate/substrate2/commit/a86074a69b714b1be551ae00c775beb04c13f776))
* **proc-macros:** macros respect field and struct visibilities ([8678716](https://github.com/ucb-substrate/substrate2/commit/86787160c49a1ac7c011d08ce1b9d7851bdfa0d8))
* **proc-macros:** proc macros find substrate crate location ([#125](https://github.com/ucb-substrate/substrate2/issues/125)) ([8678716](https://github.com/ucb-substrate/substrate2/commit/86787160c49a1ac7c011d08ce1b9d7851bdfa0d8))
* **proc-macros:** support enums, tuple structs, etc. ([#165](https://github.com/ucb-substrate/substrate2/issues/165)) ([bda83f7](https://github.com/ucb-substrate/substrate2/commit/bda83f7c3049178024b114eb4e1bf65c6a128998))
* **proc-macros:** support generics in derive schematic/layout data ([#169](https://github.com/ucb-substrate/substrate2/issues/169)) ([5bc11d8](https://github.com/ucb-substrate/substrate2/commit/5bc11d8eee266c21247694299285b6147631166e))
* **refactor:** rename Has_ and Has_Data ([#282](https://github.com/ucb-substrate/substrate2/issues/282)) ([2018153](https://github.com/ucb-substrate/substrate2/commit/2018153686dd7ef3df0e10874db3c656ca245026))
* **refactor:** significantly refactor IO APIs ([#348](https://github.com/ucb-substrate/substrate2/issues/348)) ([c85d043](https://github.com/ucb-substrate/substrate2/commit/c85d04334a0ba1740f9990b91fb55ab1f2ef77c5))
* **reorg:** move substrate-api into substrate ([#155](https://github.com/ucb-substrate/substrate2/issues/155)) ([e902a1b](https://github.com/ucb-substrate/substrate2/commit/e902a1b603cca6c719770c5cd742e081bfd33e51))
* **schematic:** associated type schema and bundle primitives ([#455](https://github.com/ucb-substrate/substrate2/issues/455)) ([f5fde78](https://github.com/ucb-substrate/substrate2/commit/f5fde78824ce9ed0be494ef68d71620181bf6b48))
* **schematic:** nested node and instance access ([#134](https://github.com/ucb-substrate/substrate2/issues/134)) ([3d0e9ce](https://github.com/ucb-substrate/substrate2/commit/3d0e9ce96b66072cd9b7982c582fa2d67ed8f406))
* **schematic:** rename bundle traits ([#458](https://github.com/ucb-substrate/substrate2/issues/458)) ([ed98443](https://github.com/ucb-substrate/substrate2/commit/ed9844318cbd7176a781fff0076d8b3385d408b5))
* **schematics:** implement node naming trees, with codegen ([#105](https://github.com/ucb-substrate/substrate2/issues/105)) ([5ef8e4b](https://github.com/ucb-substrate/substrate2/commit/5ef8e4b8cdd20a274d1a4dadda8e186bed004763))
* **schematics:** implement proc macro to derive AnalogIo ([#99](https://github.com/ucb-substrate/substrate2/issues/99)) ([2320c99](https://github.com/ucb-substrate/substrate2/commit/2320c99e9852d4698c5b336de0af7ebe7cc94204))
* **schematics:** rename `HasSchematic` and `HasSchematicImpl` ([#226](https://github.com/ucb-substrate/substrate2/issues/226)) ([a2b9c78](https://github.com/ucb-substrate/substrate2/commit/a2b9c78ea6ff56983e9a02aeafe655e92852c264))
* **schematics:** user-specified schematic hierarchy flattening ([#222](https://github.com/ucb-substrate/substrate2/issues/222)) ([251f377](https://github.com/ucb-substrate/substrate2/commit/251f37778526d2f1c08a2b3c66f72ffe273021fa))
* **simulation:** automatically generate saved data ([#457](https://github.com/ucb-substrate/substrate2/issues/457)) ([2c936d0](https://github.com/ucb-substrate/substrate2/commit/2c936d00e927b99b624f29e6450826e90f68f9bf))
* **simulation:** implement save for nested instances ([#476](https://github.com/ucb-substrate/substrate2/issues/476)) ([a47d905](https://github.com/ucb-substrate/substrate2/commit/a47d905097c6c196153b53f142ca7e1ffba5eb51))
* **simulation:** improve simulation APIs ([#320](https://github.com/ucb-substrate/substrate2/issues/320)) ([4ed59a1](https://github.com/ucb-substrate/substrate2/commit/4ed59a1283f9546e8336cc96015bd87c55682777))
* **simulation:** proc macros for implementing Supports on tuples ([#163](https://github.com/ucb-substrate/substrate2/issues/163)) ([bf77832](https://github.com/ucb-substrate/substrate2/commit/bf778329d6e9fd317bea789d093c4c7d8790f5ac))
* **spectre:** vsource uses primitives instead of being blackboxed ([5dabcb2](https://github.com/ucb-substrate/substrate2/commit/5dabcb270cab0d259b7301d67f77de6d55261092))
* **stdcells:** add standard cell support to Sky130 PDK ([#323](https://github.com/ucb-substrate/substrate2/issues/323)) ([0b2048e](https://github.com/ucb-substrate/substrate2/commit/0b2048ed44d89c5de87380cac48a4bbff2b4c20a))
* **terminals:** add support for terminal paths ([#236](https://github.com/ucb-substrate/substrate2/issues/236)) ([3fba7f6](https://github.com/ucb-substrate/substrate2/commit/3fba7f6227bbf2efcaf79d849c79175e44d783a4))
* **tests:** fix compilation and lint errors ([#482](https://github.com/ucb-substrate/substrate2/issues/482)) ([b55d04e](https://github.com/ucb-substrate/substrate2/commit/b55d04ecd2472f9f72b926ba5286f0d928bc2691))
* **tests:** reorganize tests and documentation ([#464](https://github.com/ucb-substrate/substrate2/issues/464)) ([928b9b7](https://github.com/ucb-substrate/substrate2/commit/928b9b7c45dc334ca11d86e4564edc58bf6db6f2))
* **tiling:** array and grid tiling API ([#201](https://github.com/ucb-substrate/substrate2/issues/201)) ([b3b7c2b](https://github.com/ucb-substrate/substrate2/commit/b3b7c2bfb7ba72198872d0f08ded3e0bc757479d))
* **transform:** default to Manhattan transformations ([#452](https://github.com/ucb-substrate/substrate2/issues/452)) ([3d8a410](https://github.com/ucb-substrate/substrate2/commit/3d8a4109febb11616d550c8cd6373e8f605b2e28))
* **type-dispatch:** add helper crate for dispatching types ([#225](https://github.com/ucb-substrate/substrate2/issues/225)) ([13ee1aa](https://github.com/ucb-substrate/substrate2/commit/13ee1aa1b287ed0c147549003c0af815b849577b))
* **views:** view API for improved codegen ([#463](https://github.com/ucb-substrate/substrate2/issues/463)) ([b75328c](https://github.com/ucb-substrate/substrate2/commit/b75328c9a4840ed9200a9035e28e27ac9265770f))


### Bug Fixes

* **ci:** add workaround for dev deps ([180c924](https://github.com/ucb-substrate/substrate2/commit/180c92434b38a5da8d5d1f0494faae6a0b227c26))
* **ci:** test another workaround for dev deps ([c15bc6d](https://github.com/ucb-substrate/substrate2/commit/c15bc6d30afc02512237223db5f31cd9cb089ede))
* **codegen:** update codegen to use fewer structs ([#461](https://github.com/ucb-substrate/substrate2/issues/461)) ([c371be5](https://github.com/ucb-substrate/substrate2/commit/c371be59adebb9482095284034d41a6905c431d4))
* **deps:** update rust crate syn to v2 ([#79](https://github.com/ucb-substrate/substrate2/issues/79)) ([eee3593](https://github.com/ucb-substrate/substrate2/commit/eee35938247f2660c15b0165b6ba3d609d7091b8))
* **docs:** fix snippet publishing ([#512](https://github.com/ucb-substrate/substrate2/issues/512)) ([456f8bf](https://github.com/ucb-substrate/substrate2/commit/456f8bfe659d4fa2a05f6d56394a6171c4fd34dd))
* **docs:** remove Cargo.tomls in CI to allow publishing of API docs ([#515](https://github.com/ucb-substrate/substrate2/issues/515)) ([2d14f50](https://github.com/ucb-substrate/substrate2/commit/2d14f50add396a1d775428b273df7d8d022aea05))
* **gds:** use u16 instead of u8 for GDS layerspecs ([#339](https://github.com/ucb-substrate/substrate2/issues/339)) ([4d1fce2](https://github.com/ucb-substrate/substrate2/commit/4d1fce25f9493c6975d43dba96ccaa4c0cf4a686))
* **generics:** change `Deserialize&lt;'static&gt;` bounds to `DeserializeOwned` ([#259](https://github.com/ucb-substrate/substrate2/issues/259)) ([8015063](https://github.com/ucb-substrate/substrate2/commit/80150630b094a04a75cfc5b681255b80caf4f895))
* **tests:** increase cache server wait time ([#167](https://github.com/ucb-substrate/substrate2/issues/167)) ([b0db3aa](https://github.com/ucb-substrate/substrate2/commit/b0db3aa6285367de1650e972c9cf7e2185a68250))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * snippets bumped from 0.7.0 to 0.8.0
    * macrotools bumped from 0.2.0 to 0.3.0
  * dev-dependencies
    * substrate bumped from <=0.10.0 to <=0.11.0
    * scir bumped from <=0.9.0 to <=0.10.0
  * build-dependencies
    * snippets bumped from 0.7.0 to 0.8.0
    * examples bumped from 0.2.0 to 0.3.0
</details>

<details><summary>colbuf: 0.0.0</summary>

### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * substrate bumped from 0.10.0 to 0.11.0
    * spice bumped from 0.9.0 to 0.10.0
    * spectre bumped from 0.11.0 to 0.12.0
    * ngspice bumped from 0.5.0 to 0.6.0
    * quantus bumped from 0.2.0 to 0.3.0
    * magic_netgen bumped from 0.1.1 to 0.1.2
</details>

<details><summary>config: 0.5.0</summary>

## [0.5.0](https://github.com/ucb-substrate/substrate2/compare/config-v0.4.0...config-v0.5.0) (2025-01-23)


### Features

* **ac:** implement Save for Substrate types in Spectre AC sim, fix lints ([#471](https://github.com/ucb-substrate/substrate2/issues/471)) ([9825520](https://github.com/ucb-substrate/substrate2/commit/98255207569cc00bd9ddc35419c2df1e48f1999c))
* **cache-config:** allow configuration of cache via config files ([#192](https://github.com/ucb-substrate/substrate2/issues/192)) ([0461402](https://github.com/ucb-substrate/substrate2/commit/0461402edfc1ec0886bbb25cf5471ee8480754fc))
* **config:** config merging and parsing functionality ([#40](https://github.com/ucb-substrate/substrate2/issues/40)) ([13c8925](https://github.com/ucb-substrate/substrate2/commit/13c8925fa5e341c1056e43e00f963fc4dcda8190))
* **docs:** add missing documentation to config crate ([#55](https://github.com/ucb-substrate/substrate2/issues/55)) ([cf10436](https://github.com/ucb-substrate/substrate2/commit/cf10436ef1f5881baf1c76247520ebc3cd39852a))
* **grid:** add ATOLL LCM routing grid and layer stack definition ([#338](https://github.com/ucb-substrate/substrate2/issues/338)) ([1e1ad90](https://github.com/ucb-substrate/substrate2/commit/1e1ad90d02b50dd0dd15516c306971241bf30b7c))
* **organization:** rename substrate to substrate_api, set up codegen crate ([#67](https://github.com/ucb-substrate/substrate2/issues/67)) ([e07f099](https://github.com/ucb-substrate/substrate2/commit/e07f09949551fd08e3f58b6ffb7d9a8c67b76ae9))
* **simulation:** support transient simulation in spectre ([#133](https://github.com/ucb-substrate/substrate2/issues/133)) ([4605862](https://github.com/ucb-substrate/substrate2/commit/460586252e3695ae32b0ab8d83b90023125d1a33))
* **tests:** fix compilation and lint errors ([#482](https://github.com/ucb-substrate/substrate2/issues/482)) ([b55d04e](https://github.com/ucb-substrate/substrate2/commit/b55d04ecd2472f9f72b926ba5286f0d928bc2691))
* **tiling:** array and grid tiling API ([#201](https://github.com/ucb-substrate/substrate2/issues/201)) ([b3b7c2b](https://github.com/ucb-substrate/substrate2/commit/b3b7c2bfb7ba72198872d0f08ded3e0bc757479d))
* **windows:** fix issues for windows ([#197](https://github.com/ucb-substrate/substrate2/issues/197)) ([008b607](https://github.com/ucb-substrate/substrate2/commit/008b607b2c21c14ac3106dca6eb74d806131ef8f))


### Bug Fixes

* **config:** add config to release manifest and fix version ([#41](https://github.com/ucb-substrate/substrate2/issues/41)) ([b7097f5](https://github.com/ucb-substrate/substrate2/commit/b7097f5ec981c0972a3ef018d182f786feac64d5))
* **deps:** fix dependencies and documentation ([#66](https://github.com/ucb-substrate/substrate2/issues/66)) ([a60ffc6](https://github.com/ucb-substrate/substrate2/commit/a60ffc6c5501200d56a6e76db0c1c2f7ef9cd086))
* **deps:** update rust crate toml_edit to 0.20 ([#307](https://github.com/ucb-substrate/substrate2/issues/307)) ([7681606](https://github.com/ucb-substrate/substrate2/commit/7681606c082c8f7b0ef98b114348c90f6ea83d16))
* **docs:** fix additional clippy errors and missing docs ([#56](https://github.com/ucb-substrate/substrate2/issues/56)) ([f76a169](https://github.com/ucb-substrate/substrate2/commit/f76a1693fa575753abefa798c103f84ca942a6e4))
* **docs:** fix broken links and check docs in CI ([#59](https://github.com/ucb-substrate/substrate2/issues/59)) ([13dc7a5](https://github.com/ucb-substrate/substrate2/commit/13dc7a50c21c3ba54e85b1d11d1e6ad22051b51f))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * cache bumped from 0.7.0 to 0.8.0
</details>

<details><summary>diagnostics: 0.5.0</summary>

## [0.5.0](https://github.com/ucb-substrate/substrate2/compare/diagnostics-v0.4.0...diagnostics-v0.5.0) (2025-01-23)


### Features

* **repo:** reorganize repo ([#207](https://github.com/ucb-substrate/substrate2/issues/207)) ([54a6b43](https://github.com/ucb-substrate/substrate2/commit/54a6b43079d283a29bc0aa9e18dc6230b56fa385))
* **validation:** create type for unvalidated SCIR library ([#252](https://github.com/ucb-substrate/substrate2/issues/252)) ([1550a22](https://github.com/ucb-substrate/substrate2/commit/1550a22b9a1c9f7cd9717feaa45d00487cc8848e))
</details>

<details><summary>enumify: 0.3.0</summary>

## [0.3.0](https://github.com/ucb-substrate/substrate2/compare/enumify-v0.2.0...enumify-v0.3.0) (2025-01-23)


### Features

* **codegen:** enumify attribute macro ([#284](https://github.com/ucb-substrate/substrate2/issues/284)) ([6e9e529](https://github.com/ucb-substrate/substrate2/commit/6e9e52951ef58e3a9b897417fb844a7706762d06))
* **docs:** add documentation for enumify ([#290](https://github.com/ucb-substrate/substrate2/issues/290)) ([42fbe70](https://github.com/ucb-substrate/substrate2/commit/42fbe707a63c5c95155e4b8d1b73605290b59d43))
* **primitives:** revamp schematic primitives ([#291](https://github.com/ucb-substrate/substrate2/issues/291)) ([e5ba06a](https://github.com/ucb-substrate/substrate2/commit/e5ba06ab10008b72e78397ad70781caa6bc61791))


### Bug Fixes

* formatting ([#285](https://github.com/ucb-substrate/substrate2/issues/285)) ([a2ca991](https://github.com/ucb-substrate/substrate2/commit/a2ca9913bba0cd7ee6da29223f873aaf2e861c11))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * enumify_macros bumped from 0.3.0 to 0.4.0
</details>

<details><summary>enumify_macros: 0.4.0</summary>

## [0.4.0](https://github.com/ucb-substrate/substrate2/compare/enumify_macros-v0.3.0...enumify_macros-v0.4.0) (2025-01-23)


### Features

* **codegen:** enumify attribute macro ([#284](https://github.com/ucb-substrate/substrate2/issues/284)) ([6e9e529](https://github.com/ucb-substrate/substrate2/commit/6e9e52951ef58e3a9b897417fb844a7706762d06))
* **docs:** add atoll design docs ([#293](https://github.com/ucb-substrate/substrate2/issues/293)) ([996f1bc](https://github.com/ucb-substrate/substrate2/commit/996f1bcd0f071ec845fa60ff45f404cd71d42632))
* **primitives:** revamp schematic primitives ([#291](https://github.com/ucb-substrate/substrate2/issues/291)) ([e5ba06a](https://github.com/ucb-substrate/substrate2/commit/e5ba06ab10008b72e78397ad70781caa6bc61791))
* **tests:** reorganize tests and documentation ([#464](https://github.com/ucb-substrate/substrate2/issues/464)) ([928b9b7](https://github.com/ucb-substrate/substrate2/commit/928b9b7c45dc334ca11d86e4564edc58bf6db6f2))


### Bug Fixes

* **scir:** remove use of opacity from SCIR ([#286](https://github.com/ucb-substrate/substrate2/issues/286)) ([5e38b28](https://github.com/ucb-substrate/substrate2/commit/5e38b288629b5f2d6d3ca372418a331b6bd98e5e))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * macrotools bumped from 0.2.0 to 0.3.0
</details>

<details><summary>examples: 0.3.0</summary>

## [0.3.0](https://github.com/ucb-substrate/substrate2/compare/examples-v0.2.0...examples-v0.3.0) (2025-01-23)


### Features

* **docs:** inverter design tutorial updates ([#154](https://github.com/ucb-substrate/substrate2/issues/154)) ([276cfd1](https://github.com/ucb-substrate/substrate2/commit/276cfd1733bd26e4b5b68a0667f610012c895261))
* **docs:** update docs for new simulation APIs ([#326](https://github.com/ucb-substrate/substrate2/issues/326)) ([ef133df](https://github.com/ucb-substrate/substrate2/commit/ef133dfac5f352121fe0e561b76541d5af62970e))
* **docs:** update tutorials and revamp documentation website ([#315](https://github.com/ucb-substrate/substrate2/issues/315)) ([49bdf7f](https://github.com/ucb-substrate/substrate2/commit/49bdf7ff61e2fdbf19022697d518ad7fbafb465f))
* **docs:** versioned documentation between HEAD and release ([#470](https://github.com/ucb-substrate/substrate2/issues/470)) ([968182b](https://github.com/ucb-substrate/substrate2/commit/968182bf8f8d8b4cf923c0fd66f1ca1b32b12b16))
* **examples:** separate workspace for examples ([#160](https://github.com/ucb-substrate/substrate2/issues/160)) ([5121ee9](https://github.com/ucb-substrate/substrate2/commit/5121ee9402ed5810923e0f37dabe10277588b15f))
* **macros:** refactor derive NestedData, start organizing tests ([#477](https://github.com/ucb-substrate/substrate2/issues/477)) ([aca48ef](https://github.com/ucb-substrate/substrate2/commit/aca48ef7a49c959e35ec4614345a55e667ff5146))
* **simulation:** improve simulation APIs ([#320](https://github.com/ucb-substrate/substrate2/issues/320)) ([4ed59a1](https://github.com/ucb-substrate/substrate2/commit/4ed59a1283f9546e8336cc96015bd87c55682777))
* **spice:** refactor netlisting and fix voltage source netlist ([#316](https://github.com/ucb-substrate/substrate2/issues/316)) ([7a3df69](https://github.com/ucb-substrate/substrate2/commit/7a3df695cf9b38c837ff86d5a5da2417c4db7aa2))
* **type-dispatch:** add helper crate for dispatching types ([#225](https://github.com/ucb-substrate/substrate2/issues/225)) ([13ee1aa](https://github.com/ucb-substrate/substrate2/commit/13ee1aa1b287ed0c147549003c0af815b849577b))


### Bug Fixes

* **ci:** fix ci for Substrate v2.1 ([#490](https://github.com/ucb-substrate/substrate2/issues/490)) ([cc09d71](https://github.com/ucb-substrate/substrate2/commit/cc09d7199b41fb2986d1d733aa3678db49464f70))
* **ci:** test release examples after crates are published ([#493](https://github.com/ucb-substrate/substrate2/issues/493)) ([686a972](https://github.com/ucb-substrate/substrate2/commit/686a972a9e6ca7833fd5ff548e3b3f0c5469952c))
* **docs:** fix snippet publishing ([#512](https://github.com/ucb-substrate/substrate2/issues/512)) ([456f8bf](https://github.com/ucb-substrate/substrate2/commit/456f8bfe659d4fa2a05f6d56394a6171c4fd34dd))
* **docs:** move use statement into code snippet ([#367](https://github.com/ucb-substrate/substrate2/issues/367)) ([c72bbc8](https://github.com/ucb-substrate/substrate2/commit/c72bbc8a89b0e617c7fae880313385a6383384bc))
* **docs:** remove Cargo.tomls in CI to allow publishing of API docs ([#515](https://github.com/ucb-substrate/substrate2/issues/515)) ([2d14f50](https://github.com/ucb-substrate/substrate2/commit/2d14f50add396a1d775428b273df7d8d022aea05))
* **docs:** update docs to latest release (substrate v0.3.0) ([#204](https://github.com/ucb-substrate/substrate2/issues/204)) ([df45b6c](https://github.com/ucb-substrate/substrate2/commit/df45b6c56c7eb8d01bd2ec104c5d2593bc8f80cc))
* **docs:** use numbered deps in inverter tutorial ([#159](https://github.com/ucb-substrate/substrate2/issues/159)) ([f2b8e08](https://github.com/ucb-substrate/substrate2/commit/f2b8e0846e7080d38748766acf3624415b4d0a29))
* **release:** use correct gds for `examples/release/colbuf/test_col_buffer_array.gds` ([#520](https://github.com/ucb-substrate/substrate2/issues/520)) ([63ea1ea](https://github.com/ucb-substrate/substrate2/commit/63ea1ea60426ac788218c5830cca92601737d5e4))
</details>

<details><summary>gds: 0.5.0</summary>

## [0.5.0](https://github.com/ucb-substrate/substrate2/compare/gds-v0.4.0...gds-v0.5.0) (2025-01-23)


### Features

* **repo:** reorganize repo ([#207](https://github.com/ucb-substrate/substrate2/issues/207)) ([54a6b43](https://github.com/ucb-substrate/substrate2/commit/54a6b43079d283a29bc0aa9e18dc6230b56fa385))


### Bug Fixes

* **layout:** fix issues in GDS export and ATOLL API ([#341](https://github.com/ucb-substrate/substrate2/issues/341)) ([08930b1](https://github.com/ucb-substrate/substrate2/commit/08930b1b25d018c20758986e206dc8882df782af))
</details>

<details><summary>gdsconv: 0.3.0</summary>

## [0.3.0](https://github.com/ucb-substrate/substrate2/compare/gdsconv-v0.2.0...gdsconv-v0.3.0) (2025-01-23)


### Features

* **ac:** implement Save for Substrate types in Spectre AC sim, fix lints ([#471](https://github.com/ucb-substrate/substrate2/issues/471)) ([9825520](https://github.com/ucb-substrate/substrate2/commit/98255207569cc00bd9ddc35419c2df1e48f1999c))
* **layir:** initial LayIR implementation ([#456](https://github.com/ucb-substrate/substrate2/issues/456)) ([4f76d41](https://github.com/ucb-substrate/substrate2/commit/4f76d41c86fd0c57e525f40c976b5eeb0bbd4c68))
* **layout:** simplified layout API, LayIR integration ([#459](https://github.com/ucb-substrate/substrate2/issues/459)) ([183d347](https://github.com/ucb-substrate/substrate2/commit/183d347c19e6fe98cf870be4716e7249f23bd423))
* **tests:** fix compilation and lint errors ([#482](https://github.com/ucb-substrate/substrate2/issues/482)) ([b55d04e](https://github.com/ucb-substrate/substrate2/commit/b55d04ecd2472f9f72b926ba5286f0d928bc2691))
* **tests:** reorganize tests and documentation ([#464](https://github.com/ucb-substrate/substrate2/issues/464)) ([928b9b7](https://github.com/ucb-substrate/substrate2/commit/928b9b7c45dc334ca11d86e4564edc58bf6db6f2))


### Bug Fixes

* **ci:** use head_ref instead of ref and fix gdsconv version ([#498](https://github.com/ucb-substrate/substrate2/issues/498)) ([bc5d66e](https://github.com/ucb-substrate/substrate2/commit/bc5d66e5aad82ea79436e2fb3ec33e960a58f7b6))
* **deps:** add missing `registry=substrate` for in-tree dependencies ([#517](https://github.com/ucb-substrate/substrate2/issues/517)) ([505d95c](https://github.com/ucb-substrate/substrate2/commit/505d95c17c5997166c1987cbc30e344fdd4c78fb))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * layir bumped from 0.2.0 to 0.3.0
    * gds bumped from 0.4.0 to 0.5.0
    * geometry bumped from 0.7.0 to 0.8.0
</details>

<details><summary>geometry: 0.8.0</summary>

## [0.8.0](https://github.com/ucb-substrate/substrate2/compare/geometry-v0.7.0...geometry-v0.8.0) (2025-01-23)


### Features

* **ac:** implement Save for Substrate types in Spectre AC sim, fix lints ([#471](https://github.com/ucb-substrate/substrate2/issues/471)) ([9825520](https://github.com/ucb-substrate/substrate2/commit/98255207569cc00bd9ddc35419c2df1e48f1999c))
* **atoll:** implement first cut ATOLL implementation ([#357](https://github.com/ucb-substrate/substrate2/issues/357)) ([372b927](https://github.com/ucb-substrate/substrate2/commit/372b9275c9d9c5cd58603f5a462a5e4b66b64cf7))
* **bbox:** add bbox_rect method ([#373](https://github.com/ucb-substrate/substrate2/issues/373)) ([55b2632](https://github.com/ucb-substrate/substrate2/commit/55b2632a3c1e1ad260b61c6545143a2b16ef1150))
* **blackboxing:** support Substrate and SCIR blackboxes ([#135](https://github.com/ucb-substrate/substrate2/issues/135)) ([049a598](https://github.com/ucb-substrate/substrate2/commit/049a598e2b8d11228c63f03dc878fc4c56e036a6))
* **def:** utilities for exporting def orientations ([#434](https://github.com/ucb-substrate/substrate2/issues/434)) ([43a2b29](https://github.com/ucb-substrate/substrate2/commit/43a2b2906231cd46f08e2c4aface260d34abac62))
* **dirs:** add `Dirs` struct ([#371](https://github.com/ucb-substrate/substrate2/issues/371)) ([6d6b834](https://github.com/ucb-substrate/substrate2/commit/6d6b8347eea60ed1fccaed16623d146c3bd0727e))
* **gds-import:** implement GDS to RawCell importer ([#196](https://github.com/ucb-substrate/substrate2/issues/196)) ([fc37eeb](https://github.com/ucb-substrate/substrate2/commit/fc37eeb6bac10779491b98bcadcc0eeaeb7d8ec5))
* **gds:** gds reexport test ([#199](https://github.com/ucb-substrate/substrate2/issues/199)) ([93d3cd5](https://github.com/ucb-substrate/substrate2/commit/93d3cd555c1cb4a76a8845f4401e98d327b5d674))
* **geometry:** implemented contains for polygon ([#292](https://github.com/ucb-substrate/substrate2/issues/292)) ([708053a](https://github.com/ucb-substrate/substrate2/commit/708053adfb9f3783fc03895ede7348ace51730f0))
* **geometry:** support for rectangular rings ([#408](https://github.com/ucb-substrate/substrate2/issues/408)) ([6fc0f36](https://github.com/ucb-substrate/substrate2/commit/6fc0f361f2215968f698281bfaf37d03d3ec131e))
* **grid:** add ATOLL LCM routing grid and layer stack definition ([#338](https://github.com/ucb-substrate/substrate2/issues/338)) ([1e1ad90](https://github.com/ucb-substrate/substrate2/commit/1e1ad90d02b50dd0dd15516c306971241bf30b7c))
* **layir:** initial LayIR implementation ([#456](https://github.com/ucb-substrate/substrate2/issues/456)) ([4f76d41](https://github.com/ucb-substrate/substrate2/commit/4f76d41c86fd0c57e525f40c976b5eeb0bbd4c68))
* **macros:** refactor macro reexports ([#250](https://github.com/ucb-substrate/substrate2/issues/250)) ([a332717](https://github.com/ucb-substrate/substrate2/commit/a332717e549fdea50306067e1c92dc60293aed4c))
* **netlists:** use consistent ordering via indexmap ([#266](https://github.com/ucb-substrate/substrate2/issues/266)) ([f275c19](https://github.com/ucb-substrate/substrate2/commit/f275c19396ed4f7d255836822ff72b808f89cde7)), closes [#265](https://github.com/ucb-substrate/substrate2/issues/265)
* **polygon:** polygon implemented in geometry ([#263](https://github.com/ucb-substrate/substrate2/issues/263)) ([4508570](https://github.com/ucb-substrate/substrate2/commit/45085706a30a12f4af6c5e3f642ca55b4c32dd24))
* **proc-macros:** derive macros for geometry traits ([#164](https://github.com/ucb-substrate/substrate2/issues/164)) ([a86074a](https://github.com/ucb-substrate/substrate2/commit/a86074a69b714b1be551ae00c775beb04c13f776))
* **tests:** add SCIR and SPICE netlister blackbox tests ([049a598](https://github.com/ucb-substrate/substrate2/commit/049a598e2b8d11228c63f03dc878fc4c56e036a6))
* **tests:** fix compilation and lint errors ([#482](https://github.com/ucb-substrate/substrate2/issues/482)) ([b55d04e](https://github.com/ucb-substrate/substrate2/commit/b55d04ecd2472f9f72b926ba5286f0d928bc2691))
* **tests:** reorganize tests and documentation ([#464](https://github.com/ucb-substrate/substrate2/issues/464)) ([928b9b7](https://github.com/ucb-substrate/substrate2/commit/928b9b7c45dc334ca11d86e4564edc58bf6db6f2))
* **tiling:** array and grid tiling API ([#201](https://github.com/ucb-substrate/substrate2/issues/201)) ([b3b7c2b](https://github.com/ucb-substrate/substrate2/commit/b3b7c2bfb7ba72198872d0f08ded3e0bc757479d))
* **transform:** default to Manhattan transformations ([#452](https://github.com/ucb-substrate/substrate2/issues/452)) ([3d8a410](https://github.com/ucb-substrate/substrate2/commit/3d8a4109febb11616d550c8cd6373e8f605b2e28))
* **transform:** make transformations use integers instead of floats ([#451](https://github.com/ucb-substrate/substrate2/issues/451)) ([aa9764e](https://github.com/ucb-substrate/substrate2/commit/aa9764e8b63b0a344d5e12ad3c678849c5c8ebea))
* **tutorial:** implement sky130 inverter layout tutorial ([#481](https://github.com/ucb-substrate/substrate2/issues/481)) ([440ab0e](https://github.com/ucb-substrate/substrate2/commit/440ab0e6ac33a8396c10f09637242efa32cfca62))


### Bug Fixes

* **deps:** bump rust to version 1.75.0 ([#362](https://github.com/ucb-substrate/substrate2/issues/362)) ([e1e82c9](https://github.com/ucb-substrate/substrate2/commit/e1e82c94cdf6ba4426f3f73f29dca40674a7f064))
* **deps:** update rust crate num-rational to 0.4 ([#294](https://github.com/ucb-substrate/substrate2/issues/294)) ([fc8f5ce](https://github.com/ucb-substrate/substrate2/commit/fc8f5ce9f35eb074acff45115e44ffbd37e0d237))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * geometry_macros bumped from 0.0.3 to 0.0.4
</details>

<details><summary>geometry_macros: 0.0.4</summary>

### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * macrotools bumped from 0.2.0 to 0.3.0
</details>

<details><summary>layir: 0.3.0</summary>

## [0.3.0](https://github.com/ucb-substrate/substrate2/compare/layir-v0.2.0...layir-v0.3.0) (2025-01-23)


### Features

* **layir:** initial LayIR implementation ([#456](https://github.com/ucb-substrate/substrate2/issues/456)) ([4f76d41](https://github.com/ucb-substrate/substrate2/commit/4f76d41c86fd0c57e525f40c976b5eeb0bbd4c68))
* **layout:** simplified layout API, LayIR integration ([#459](https://github.com/ucb-substrate/substrate2/issues/459)) ([183d347](https://github.com/ucb-substrate/substrate2/commit/183d347c19e6fe98cf870be4716e7249f23bd423))
* **mos:** layout for sky130 1.8V nmos/pmos, fix geometry macros ([#478](https://github.com/ucb-substrate/substrate2/issues/478)) ([55f17b7](https://github.com/ucb-substrate/substrate2/commit/55f17b72ab90e12efb57d97fdad6b4e5373c30e2))
* **tests:** reorganize tests and documentation ([#464](https://github.com/ucb-substrate/substrate2/issues/464)) ([928b9b7](https://github.com/ucb-substrate/substrate2/commit/928b9b7c45dc334ca11d86e4564edc58bf6db6f2))


### Bug Fixes

* **ci:** fix ci for Substrate v2.1 ([#490](https://github.com/ucb-substrate/substrate2/issues/490)) ([cc09d71](https://github.com/ucb-substrate/substrate2/commit/cc09d7199b41fb2986d1d733aa3678db49464f70))
* **deps:** add missing `registry=substrate` for in-tree dependencies ([#517](https://github.com/ucb-substrate/substrate2/issues/517)) ([505d95c](https://github.com/ucb-substrate/substrate2/commit/505d95c17c5997166c1987cbc30e344fdd4c78fb))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * diagnostics bumped from 0.4.0 to 0.5.0
    * uniquify bumped from 0.4.0 to 0.5.0
    * enumify bumped from 0.2.0 to 0.3.0
    * geometry bumped from 0.7.0 to 0.8.0
</details>

<details><summary>lefdef: 0.3.0</summary>

## [0.3.0](https://github.com/ucb-substrate/substrate2/compare/lefdef-v0.2.0...lefdef-v0.3.0) (2025-01-23)


### Features

* **def:** utilities for exporting def orientations ([#434](https://github.com/ucb-substrate/substrate2/issues/434)) ([43a2b29](https://github.com/ucb-substrate/substrate2/commit/43a2b2906231cd46f08e2c4aface260d34abac62))
* **lefdef:** initial DEF writer implementation ([#431](https://github.com/ucb-substrate/substrate2/issues/431)) ([d0ef249](https://github.com/ucb-substrate/substrate2/commit/d0ef249fa70f754a946f677b250ba0889dccd0c2))


### Bug Fixes

* **def:** fix def special nets routing status ([#439](https://github.com/ucb-substrate/substrate2/issues/439)) ([a6ffd9a](https://github.com/ucb-substrate/substrate2/commit/a6ffd9a4b63a5cf6c995cae7da78a271c652aeab))
* **def:** remove whitespace after END DESIGN stmt ([#436](https://github.com/ucb-substrate/substrate2/issues/436)) ([21fec8b](https://github.com/ucb-substrate/substrate2/commit/21fec8be19986200d41a0ca4e07581dfb72ed30b))
* **lefdef:** fix lefdef crate version ([#433](https://github.com/ucb-substrate/substrate2/issues/433)) ([42746db](https://github.com/ucb-substrate/substrate2/commit/42746dbb1c8f413446cb74d6ae94d17e2f5d45b4))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * geometry bumped from 0.7.0 to 0.8.0
</details>

<details><summary>macrotools: 0.3.0</summary>

## [0.3.0](https://github.com/ucb-substrate/substrate2/compare/macrotools-v0.2.0...macrotools-v0.3.0) (2025-01-23)


### Features

* **ac:** implement Save for Substrate types in Spectre AC sim, fix lints ([#471](https://github.com/ucb-substrate/substrate2/issues/471)) ([9825520](https://github.com/ucb-substrate/substrate2/commit/98255207569cc00bd9ddc35419c2df1e48f1999c))
* **macros:** support ref, mut ref, and owned receiver styles ([#468](https://github.com/ucb-substrate/substrate2/issues/468)) ([b285476](https://github.com/ucb-substrate/substrate2/commit/b285476d3ac378522a1b40ae4e22a69f5e580fda))
* **simulation:** implement save for nested instances ([#476](https://github.com/ucb-substrate/substrate2/issues/476)) ([a47d905](https://github.com/ucb-substrate/substrate2/commit/a47d905097c6c196153b53f142ca7e1ffba5eb51))
* **tests:** reorganize tests and documentation ([#464](https://github.com/ucb-substrate/substrate2/issues/464)) ([928b9b7](https://github.com/ucb-substrate/substrate2/commit/928b9b7c45dc334ca11d86e4564edc58bf6db6f2))
</details>

<details><summary>magic: 0.3.0</summary>

## [0.3.0](https://github.com/ucb-substrate/substrate2/compare/magic-v0.2.0...magic-v0.3.0) (2025-01-23)


### Features

* **ac:** implement Save for Substrate types in Spectre AC sim, fix lints ([#471](https://github.com/ucb-substrate/substrate2/issues/471)) ([9825520](https://github.com/ucb-substrate/substrate2/commit/98255207569cc00bd9ddc35419c2df1e48f1999c))
* **drc:** support running DRC with Magic ([849e674](https://github.com/ucb-substrate/substrate2/commit/849e674ebee9782a301069031e7473896eb8d2a5))
* **magic:** support magic for pex and lvs extraction ([#465](https://github.com/ucb-substrate/substrate2/issues/465)) ([c759341](https://github.com/ucb-substrate/substrate2/commit/c759341f065cf1e8aca8c4552a214391a7149cbf))
* **netgen:** support netgen for netlist comparison ([#466](https://github.com/ucb-substrate/substrate2/issues/466)) ([c3c7094](https://github.com/ucb-substrate/substrate2/commit/c3c70949de5df4ae4c08d63f2c01ed85c6e0b7fa))
* **pex:** magic-netgen pex mapping, reorganize pex tests ([#467](https://github.com/ucb-substrate/substrate2/issues/467)) ([e32802b](https://github.com/ucb-substrate/substrate2/commit/e32802bfc567f3dea50cc86b11576f7d6863fac2))
* **tutorial:** run Magic DRC, fix licon.8 DRC error ([#489](https://github.com/ucb-substrate/substrate2/issues/489)) ([c4bdc71](https://github.com/ucb-substrate/substrate2/commit/c4bdc7147abce19d023cf76be96426d58d4ed328))


### Bug Fixes

* **drc:** fix bugs in Magic DRC plugin ([#488](https://github.com/ucb-substrate/substrate2/issues/488)) ([d3c776a](https://github.com/ucb-substrate/substrate2/commit/d3c776a6bdd5e6301ed4a841239b0aed06bb2ab3))
* **extract:** throw error when Magic fails to write output netlists ([#522](https://github.com/ucb-substrate/substrate2/issues/522)) ([198eb88](https://github.com/ucb-substrate/substrate2/commit/198eb88ce83db53c72f32b38a419b2d888c177f8))
</details>

<details><summary>magic_netgen: 0.1.2</summary>

## [0.1.2](https://github.com/ucb-substrate/substrate2/compare/magic_netgen-v0.1.1...magic_netgen-v0.1.2) (2025-01-23)


### Bug Fixes

* **ci:** fix ci for Substrate v2.1 ([#490](https://github.com/ucb-substrate/substrate2/issues/490)) ([cc09d71](https://github.com/ucb-substrate/substrate2/commit/cc09d7199b41fb2986d1d733aa3678db49464f70))
* **deps:** add missing `registry=substrate` for in-tree dependencies ([#517](https://github.com/ucb-substrate/substrate2/issues/517)) ([505d95c](https://github.com/ucb-substrate/substrate2/commit/505d95c17c5997166c1987cbc30e344fdd4c78fb))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * substrate bumped from 0.10.0 to 0.11.0
    * scir bumped from 0.9.0 to 0.10.0
    * spice bumped from 0.9.0 to 0.10.0
    * magic bumped from 0.2.0 to 0.3.0
    * netgen bumped from 0.2.0 to 0.3.0
</details>

<details><summary>netgen: 0.3.0</summary>

## [0.3.0](https://github.com/ucb-substrate/substrate2/compare/netgen-v0.2.0...netgen-v0.3.0) (2025-01-23)


### Features

* **ac:** implement Save for Substrate types in Spectre AC sim, fix lints ([#471](https://github.com/ucb-substrate/substrate2/issues/471)) ([9825520](https://github.com/ucb-substrate/substrate2/commit/98255207569cc00bd9ddc35419c2df1e48f1999c))
* **netgen:** support netgen for netlist comparison ([#466](https://github.com/ucb-substrate/substrate2/issues/466)) ([c3c7094](https://github.com/ucb-substrate/substrate2/commit/c3c70949de5df4ae4c08d63f2c01ed85c6e0b7fa))
* **pex:** magic-netgen pex mapping, reorganize pex tests ([#467](https://github.com/ucb-substrate/substrate2/issues/467)) ([e32802b](https://github.com/ucb-substrate/substrate2/commit/e32802bfc567f3dea50cc86b11576f7d6863fac2))
</details>

<details><summary>ngspice: 0.6.0</summary>

## [0.6.0](https://github.com/ucb-substrate/substrate2/compare/ngspice-v0.5.0...ngspice-v0.6.0) (2025-01-23)


### Features

* **ac:** implement Save for Substrate types in Spectre AC sim, fix lints ([#471](https://github.com/ucb-substrate/substrate2/issues/471)) ([9825520](https://github.com/ucb-substrate/substrate2/commit/98255207569cc00bd9ddc35419c2df1e48f1999c))
* **cdl:** add CDL parser ([#419](https://github.com/ucb-substrate/substrate2/issues/419)) ([23f0dab](https://github.com/ucb-substrate/substrate2/commit/23f0dab7b7a94cbe8960371b89d15211bddf51da))
* **docs:** fix user docs and update dev docs ([#480](https://github.com/ucb-substrate/substrate2/issues/480)) ([f727a1e](https://github.com/ucb-substrate/substrate2/commit/f727a1e7bd2a795ace1c51c3d6e02f3673d07a29))
* **docs:** inverter tutorial cleanup and layout/pex sections ([#487](https://github.com/ucb-substrate/substrate2/issues/487)) ([5e509df](https://github.com/ucb-substrate/substrate2/commit/5e509df95a5c145fc69280269d36d860418fb1c0))
* **docs:** update docs for new simulation APIs ([#326](https://github.com/ucb-substrate/substrate2/issues/326)) ([ef133df](https://github.com/ucb-substrate/substrate2/commit/ef133dfac5f352121fe0e561b76541d5af62970e))
* **docs:** versioned documentation between HEAD and release ([#470](https://github.com/ucb-substrate/substrate2/issues/470)) ([968182b](https://github.com/ucb-substrate/substrate2/commit/968182bf8f8d8b4cf923c0fd66f1ca1b32b12b16))
* **isource:** add support for ngspice current sources ([#453](https://github.com/ucb-substrate/substrate2/issues/453)) ([098b8b8](https://github.com/ucb-substrate/substrate2/commit/098b8b8633d6998f5c5298484166ead7ac600c4d))
* **macros:** refactor derive NestedData, start organizing tests ([#477](https://github.com/ucb-substrate/substrate2/issues/477)) ([aca48ef](https://github.com/ucb-substrate/substrate2/commit/aca48ef7a49c959e35ec4614345a55e667ff5146))
* **montecarlo:** add Monte Carlo simulation support to Spectre plugin ([#347](https://github.com/ucb-substrate/substrate2/issues/347)) ([cc9dfe4](https://github.com/ucb-substrate/substrate2/commit/cc9dfe42db5be1a8aaeaf3fb81992a0ad7251ef8))
* **ngspice:** create ngspice simulator ([#274](https://github.com/ucb-substrate/substrate2/issues/274)) ([0205300](https://github.com/ucb-substrate/substrate2/commit/02053006bc26d0b3d9e1d380def89836d7921857))
* **parser:** use nutmeg format for spectre output ([#289](https://github.com/ucb-substrate/substrate2/issues/289)) ([034f58f](https://github.com/ucb-substrate/substrate2/commit/034f58f99c587c61003761971e76c26038de9b3b))
* **pex:** magic-netgen pex mapping, reorganize pex tests ([#467](https://github.com/ucb-substrate/substrate2/issues/467)) ([e32802b](https://github.com/ucb-substrate/substrate2/commit/e32802bfc567f3dea50cc86b11576f7d6863fac2))
* **primitives:** revamp schematic primitives ([#291](https://github.com/ucb-substrate/substrate2/issues/291)) ([e5ba06a](https://github.com/ucb-substrate/substrate2/commit/e5ba06ab10008b72e78397ad70781caa6bc61791))
* **refactor:** rename Has_ and Has_Data ([#282](https://github.com/ucb-substrate/substrate2/issues/282)) ([2018153](https://github.com/ucb-substrate/substrate2/commit/2018153686dd7ef3df0e10874db3c656ca245026))
* **refactor:** significantly refactor IO APIs ([#348](https://github.com/ucb-substrate/substrate2/issues/348)) ([c85d043](https://github.com/ucb-substrate/substrate2/commit/c85d04334a0ba1740f9990b91fb55ab1f2ef77c5))
* **simulation:** improve simulation APIs ([#320](https://github.com/ucb-substrate/substrate2/issues/320)) ([4ed59a1](https://github.com/ucb-substrate/substrate2/commit/4ed59a1283f9546e8336cc96015bd87c55682777))
* **spice:** refactor netlisting and fix voltage source netlist ([#316](https://github.com/ucb-substrate/substrate2/issues/316)) ([7a3df69](https://github.com/ucb-substrate/substrate2/commit/7a3df695cf9b38c837ff86d5a5da2417c4db7aa2))
* **stdcells:** add standard cell support to Sky130 PDK ([#323](https://github.com/ucb-substrate/substrate2/issues/323)) ([0b2048e](https://github.com/ucb-substrate/substrate2/commit/0b2048ed44d89c5de87380cac48a4bbff2b4c20a))
* **tests:** fix compilation and lint errors ([#482](https://github.com/ucb-substrate/substrate2/issues/482)) ([b55d04e](https://github.com/ucb-substrate/substrate2/commit/b55d04ecd2472f9f72b926ba5286f0d928bc2691))
* **tests:** reorganize tests and documentation ([#464](https://github.com/ucb-substrate/substrate2/issues/464)) ([928b9b7](https://github.com/ucb-substrate/substrate2/commit/928b9b7c45dc334ca11d86e4564edc58bf6db6f2))


### Bug Fixes

* **deps:** remove opacity from substrate and deps ([#288](https://github.com/ucb-substrate/substrate2/issues/288)) ([a8c97b3](https://github.com/ucb-substrate/substrate2/commit/a8c97b30b4d075343903fa580437e9a099a745a2))
* **deps:** update rust crate rust_decimal to 1.32 ([#296](https://github.com/ucb-substrate/substrate2/issues/296)) ([a2fe877](https://github.com/ucb-substrate/substrate2/commit/a2fe877d03d3f907f348d7711a2132194ae91034))
* **deps:** update rust crate rust_decimal_macros to 1.32 ([#297](https://github.com/ucb-substrate/substrate2/issues/297)) ([5474cc8](https://github.com/ucb-substrate/substrate2/commit/5474cc8778b81c30b34fc7d146eec6e5e2532a26))
* ngspice tests ([#310](https://github.com/ucb-substrate/substrate2/issues/310)) ([62e16bd](https://github.com/ucb-substrate/substrate2/commit/62e16bdf296a6150066369f6465f49d299d86842))
* **simulation:** add missing SPICE functionality and update Sky 130 PDK ([#336](https://github.com/ucb-substrate/substrate2/issues/336)) ([f802be5](https://github.com/ucb-substrate/substrate2/commit/f802be5bf0361c38b415d976dbb0f2c984a2e304))
* **simulation:** standardize ngspice and spectre transient data formats ([#327](https://github.com/ucb-substrate/substrate2/issues/327)) ([0aa42d6](https://github.com/ucb-substrate/substrate2/commit/0aa42d6000d28a8aecb655e06330f4545e155b9b))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * cache bumped from 0.7.0 to 0.8.0
    * scir bumped from 0.9.0 to 0.10.0
    * substrate bumped from 0.10.0 to 0.11.0
    * nutlex bumped from 0.4.0 to 0.5.0
    * spice bumped from 0.9.0 to 0.10.0
</details>

<details><summary>nutlex: 0.5.0</summary>

## [0.5.0](https://github.com/ucb-substrate/substrate2/compare/nutlex-v0.4.0...nutlex-v0.5.0) (2025-01-23)


### Features

* **macros:** refactor derive NestedData, start organizing tests ([#477](https://github.com/ucb-substrate/substrate2/issues/477)) ([aca48ef](https://github.com/ucb-substrate/substrate2/commit/aca48ef7a49c959e35ec4614345a55e667ff5146))
* **parser:** use nutmeg format for spectre output ([#289](https://github.com/ucb-substrate/substrate2/issues/289)) ([034f58f](https://github.com/ucb-substrate/substrate2/commit/034f58f99c587c61003761971e76c26038de9b3b))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * enumify bumped from 0.2.0 to 0.3.0
</details>

<details><summary>pathtree: 0.4.0</summary>

## [0.4.0](https://github.com/ucb-substrate/substrate2/compare/pathtree-v0.3.0...pathtree-v0.4.0) (2025-01-23)


### Features

* **blackboxing:** support Substrate and SCIR blackboxes ([#135](https://github.com/ucb-substrate/substrate2/issues/135)) ([049a598](https://github.com/ucb-substrate/substrate2/commit/049a598e2b8d11228c63f03dc878fc4c56e036a6))
* **simulation:** access nested nodes without strings in simulation ([#139](https://github.com/ucb-substrate/substrate2/issues/139)) ([ed7989c](https://github.com/ucb-substrate/substrate2/commit/ed7989cfb190528163a1722ae5fe3383ec3c4310))
* **terminals:** add support for terminal paths ([#236](https://github.com/ucb-substrate/substrate2/issues/236)) ([3fba7f6](https://github.com/ucb-substrate/substrate2/commit/3fba7f6227bbf2efcaf79d849c79175e44d783a4))
* **tests:** add SCIR and SPICE netlister blackbox tests ([049a598](https://github.com/ucb-substrate/substrate2/commit/049a598e2b8d11228c63f03dc878fc4c56e036a6))
</details>

<details><summary>pegasus: 0.3.0</summary>

## [0.3.0](https://github.com/ucb-substrate/substrate2/compare/pegasus-v0.2.0...pegasus-v0.3.0) (2025-01-23)


### Features

* **ac:** implement Save for Substrate types in Spectre AC sim, fix lints ([#471](https://github.com/ucb-substrate/substrate2/issues/471)) ([9825520](https://github.com/ucb-substrate/substrate2/commit/98255207569cc00bd9ddc35419c2df1e48f1999c))
* **cadence:** add support for Pegasus and Quantus ([#462](https://github.com/ucb-substrate/substrate2/issues/462)) ([953e4cb](https://github.com/ucb-substrate/substrate2/commit/953e4cb761c510668f65f4825f1be3914db45e3c))
* **pex:** magic-netgen pex mapping, reorganize pex tests ([#467](https://github.com/ucb-substrate/substrate2/issues/467)) ([e32802b](https://github.com/ucb-substrate/substrate2/commit/e32802bfc567f3dea50cc86b11576f7d6863fac2))
* **tests:** fix compilation and lint errors ([#482](https://github.com/ucb-substrate/substrate2/issues/482)) ([b55d04e](https://github.com/ucb-substrate/substrate2/commit/b55d04ecd2472f9f72b926ba5286f0d928bc2691))
</details>

<details><summary>quantus: 0.3.0</summary>

## [0.3.0](https://github.com/ucb-substrate/substrate2/compare/quantus-v0.2.0...quantus-v0.3.0) (2025-01-23)


### Features

* **ac:** implement Save for Substrate types in Spectre AC sim, fix lints ([#471](https://github.com/ucb-substrate/substrate2/issues/471)) ([9825520](https://github.com/ucb-substrate/substrate2/commit/98255207569cc00bd9ddc35419c2df1e48f1999c))
* **cadence:** add support for Pegasus and Quantus ([#462](https://github.com/ucb-substrate/substrate2/issues/462)) ([953e4cb](https://github.com/ucb-substrate/substrate2/commit/953e4cb761c510668f65f4825f1be3914db45e3c))
* **pex:** magic-netgen pex mapping, reorganize pex tests ([#467](https://github.com/ucb-substrate/substrate2/issues/467)) ([e32802b](https://github.com/ucb-substrate/substrate2/commit/e32802bfc567f3dea50cc86b11576f7d6863fac2))
* **tests:** fix compilation and lint errors ([#482](https://github.com/ucb-substrate/substrate2/issues/482)) ([b55d04e](https://github.com/ucb-substrate/substrate2/commit/b55d04ecd2472f9f72b926ba5286f0d928bc2691))
* **tests:** reorganize tests and documentation ([#464](https://github.com/ucb-substrate/substrate2/issues/464)) ([928b9b7](https://github.com/ucb-substrate/substrate2/commit/928b9b7c45dc334ca11d86e4564edc58bf6db6f2))


### Bug Fixes

* **deps:** add missing `registry=substrate` for in-tree dependencies ([#517](https://github.com/ucb-substrate/substrate2/issues/517)) ([505d95c](https://github.com/ucb-substrate/substrate2/commit/505d95c17c5997166c1987cbc30e344fdd4c78fb))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * substrate bumped from 0.10.0 to 0.11.0
    * scir bumped from 0.9.0 to 0.10.0
    * spice bumped from 0.9.0 to 0.10.0
    * pegasus bumped from 0.2.0 to 0.3.0
  * dev-dependencies
    * spectre bumped from <=0.11.0 to <=0.12.0
</details>

<details><summary>scir: 0.10.0</summary>

## [0.10.0](https://github.com/ucb-substrate/substrate2/compare/scir-v0.9.0...scir-v0.10.0) (2025-01-23)


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
    * diagnostics bumped from 0.4.0 to 0.5.0
    * uniquify bumped from 0.4.0 to 0.5.0
    * enumify bumped from 0.2.0 to 0.3.0
</details>

<details><summary>sky130: 0.11.0</summary>

## [0.11.0](https://github.com/ucb-substrate/substrate2/compare/sky130-v0.10.0...sky130-v0.11.0) (2025-01-23)


### Features

* **docs:** inverter tutorial cleanup and layout/pex sections ([#487](https://github.com/ucb-substrate/substrate2/issues/487)) ([5e509df](https://github.com/ucb-substrate/substrate2/commit/5e509df95a5c145fc69280269d36d860418fb1c0))


### Bug Fixes

* **ci:** use head_ref instead of ref and fix gdsconv version ([#498](https://github.com/ucb-substrate/substrate2/issues/498)) ([bc5d66e](https://github.com/ucb-substrate/substrate2/commit/bc5d66e5aad82ea79436e2fb3ec33e960a58f7b6))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * substrate bumped from 0.10.0 to 0.11.0
    * scir bumped from 0.9.0 to 0.10.0
    * layir bumped from 0.2.0 to 0.3.0
    * gdsconv bumped from 0.2.0 to 0.3.0
    * gds bumped from 0.4.0 to 0.5.0
    * spectre bumped from 0.11.0 to 0.12.0
    * ngspice bumped from 0.5.0 to 0.6.0
    * spice bumped from 0.9.0 to 0.10.0
    * geometry_macros bumped from 0.0.3 to 0.0.4
    * geometry bumped from 0.7.0 to 0.8.0
</details>

<details><summary>sky130_inverter: 0.0.0</summary>

### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * substrate bumped from 0.10.0 to 0.11.0
    * sky130 bumped from 0.10.0 to 0.11.0
    * layir bumped from 0.2.0 to 0.3.0
    * gdsconv bumped from 0.2.0 to 0.3.0
    * gds bumped from 0.4.0 to 0.5.0
    * scir bumped from 0.9.0 to 0.10.0
    * spice bumped from 0.9.0 to 0.10.0
    * ngspice bumped from 0.5.0 to 0.6.0
    * magic_netgen bumped from 0.1.1 to 0.1.2
    * magic bumped from 0.2.0 to 0.3.0
    * spectre bumped from 0.11.0 to 0.12.0
    * quantus bumped from 0.2.0 to 0.3.0
    * pegasus bumped from 0.2.0 to 0.3.0
</details>

<details><summary>snippets: 0.8.0</summary>

## [0.8.0](https://github.com/ucb-substrate/substrate2/compare/snippets-v0.7.0...snippets-v0.8.0) (2025-01-23)


### Features

* **ac:** implement Save for Substrate types in Spectre AC sim, fix lints ([#471](https://github.com/ucb-substrate/substrate2/issues/471)) ([9825520](https://github.com/ucb-substrate/substrate2/commit/98255207569cc00bd9ddc35419c2df1e48f1999c))
* **tests:** reorganize tests and documentation ([#464](https://github.com/ucb-substrate/substrate2/issues/464)) ([928b9b7](https://github.com/ucb-substrate/substrate2/commit/928b9b7c45dc334ca11d86e4564edc58bf6db6f2))
</details>

<details><summary>spectre: 0.12.0</summary>

## [0.12.0](https://github.com/ucb-substrate/substrate2/compare/spectre-v0.11.0...spectre-v0.12.0) (2025-01-23)


### Features

* **ac:** implement Save for Substrate types in Spectre AC sim, fix lints ([#471](https://github.com/ucb-substrate/substrate2/issues/471)) ([9825520](https://github.com/ucb-substrate/substrate2/commit/98255207569cc00bd9ddc35419c2df1e48f1999c))
* **blackboxing:** support Substrate and SCIR blackboxes ([#135](https://github.com/ucb-substrate/substrate2/issues/135)) ([049a598](https://github.com/ucb-substrate/substrate2/commit/049a598e2b8d11228c63f03dc878fc4c56e036a6))
* **cache:** implement persistent caching ([#171](https://github.com/ucb-substrate/substrate2/issues/171)) ([1f8ea24](https://github.com/ucb-substrate/substrate2/commit/1f8ea24f805085392bfd1a2067bb8774d0fa4ae4))
* **cadence:** add support for Pegasus and Quantus ([#462](https://github.com/ucb-substrate/substrate2/issues/462)) ([953e4cb](https://github.com/ucb-substrate/substrate2/commit/953e4cb761c510668f65f4825f1be3914db45e3c))
* **cdl:** add CDL parser ([#419](https://github.com/ucb-substrate/substrate2/issues/419)) ([23f0dab](https://github.com/ucb-substrate/substrate2/commit/23f0dab7b7a94cbe8960371b89d15211bddf51da))
* **codegen:** implement derive proc macro for layout hard macros ([#200](https://github.com/ucb-substrate/substrate2/issues/200)) ([5138224](https://github.com/ucb-substrate/substrate2/commit/5138224013f537e678dfb20204e964852ed40ccb))
* **conv:** better error messages in schema conversions ([#440](https://github.com/ucb-substrate/substrate2/issues/440)) ([bad9503](https://github.com/ucb-substrate/substrate2/commit/bad9503b8a3b98d8e0bc19779ed45e7628164f41))
* **corners:** add API for process corners ([#141](https://github.com/ucb-substrate/substrate2/issues/141)) ([a61b15a](https://github.com/ucb-substrate/substrate2/commit/a61b15a80851a6393aaa9da2db41e01a34f0ce5b))
* **docs:** fix user docs and update dev docs ([#480](https://github.com/ucb-substrate/substrate2/issues/480)) ([f727a1e](https://github.com/ucb-substrate/substrate2/commit/f727a1e7bd2a795ace1c51c3d6e02f3673d07a29))
* **docs:** update docs for new simulation APIs ([#326](https://github.com/ucb-substrate/substrate2/issues/326)) ([ef133df](https://github.com/ucb-substrate/substrate2/commit/ef133dfac5f352121fe0e561b76541d5af62970e))
* **dspf:** propagate nested nodes from DSPF instances ([#407](https://github.com/ucb-substrate/substrate2/issues/407)) ([8455bd2](https://github.com/ucb-substrate/substrate2/commit/8455bd2a523bb872dc1ce3fc0e89a185108dca3c))
* **executors:** executor API and local executor implementation ([#161](https://github.com/ucb-substrate/substrate2/issues/161)) ([c372d51](https://github.com/ucb-substrate/substrate2/commit/c372d511e1c67ad976dc86958c87e9ad13bdfde4))
* **ics:** spectre initial conditions ([#275](https://github.com/ucb-substrate/substrate2/issues/275)) ([ce3724e](https://github.com/ucb-substrate/substrate2/commit/ce3724e9e907f3eb3653dbf39f763865914235e3))
* **includes:** add support for including sections in spectre ([#205](https://github.com/ucb-substrate/substrate2/issues/205)) ([8522ff4](https://github.com/ucb-substrate/substrate2/commit/8522ff4241755d4c194bacb893765e608122814e))
* **inverter-design:** initial inverter design logic ([#144](https://github.com/ucb-substrate/substrate2/issues/144)) ([911a376](https://github.com/ucb-substrate/substrate2/commit/911a3768bf730d7fd134310d500560873f008783))
* **io:** composable port directions and runtime connection checking ([#231](https://github.com/ucb-substrate/substrate2/issues/231)) ([e1e367a](https://github.com/ucb-substrate/substrate2/commit/e1e367a2b8940319cb4f804888746a094f06e161))
* **macros:** refactor derive NestedData, start organizing tests ([#477](https://github.com/ucb-substrate/substrate2/issues/477)) ([aca48ef](https://github.com/ucb-substrate/substrate2/commit/aca48ef7a49c959e35ec4614345a55e667ff5146))
* **macros:** refactor macro reexports ([#250](https://github.com/ucb-substrate/substrate2/issues/250)) ([a332717](https://github.com/ucb-substrate/substrate2/commit/a332717e549fdea50306067e1c92dc60293aed4c))
* **montecarlo:** add Monte Carlo simulation support to Spectre plugin ([#347](https://github.com/ucb-substrate/substrate2/issues/347)) ([cc9dfe4](https://github.com/ucb-substrate/substrate2/commit/cc9dfe42db5be1a8aaeaf3fb81992a0ad7251ef8))
* **netlister:** reduce duplicate code between spectre and SPICE netlisters ([#261](https://github.com/ucb-substrate/substrate2/issues/261)) ([5ba3623](https://github.com/ucb-substrate/substrate2/commit/5ba36230e653e4dc77819c5c50b527311768cd83))
* **netlisting:** implement basic Spectre netlister ([#130](https://github.com/ucb-substrate/substrate2/issues/130)) ([506837f](https://github.com/ucb-substrate/substrate2/commit/506837fca0f1964ebc622df970575b321456ab68))
* **netlists:** consistent Spectre/Spice netlist API ([#349](https://github.com/ucb-substrate/substrate2/issues/349)) ([2f9fabf](https://github.com/ucb-substrate/substrate2/commit/2f9fabf336fa1048d759e78834979ef892fc0bcf))
* **netlists:** support ideal 2-terminal capacitors ([#269](https://github.com/ucb-substrate/substrate2/issues/269)) ([7de9843](https://github.com/ucb-substrate/substrate2/commit/7de9843c9b629ea06518448fe26d384de4a66cdc))
* **netlists:** use consistent ordering via indexmap ([#266](https://github.com/ucb-substrate/substrate2/issues/266)) ([f275c19](https://github.com/ucb-substrate/substrate2/commit/f275c19396ed4f7d255836822ff72b808f89cde7)), closes [#265](https://github.com/ucb-substrate/substrate2/issues/265)
* **ngspice:** create ngspice simulator ([#274](https://github.com/ucb-substrate/substrate2/issues/274)) ([0205300](https://github.com/ucb-substrate/substrate2/commit/02053006bc26d0b3d9e1d380def89836d7921857))
* **parameters:** substrate schematic primitives support parameters ([#233](https://github.com/ucb-substrate/substrate2/issues/233)) ([5dabcb2](https://github.com/ucb-substrate/substrate2/commit/5dabcb270cab0d259b7301d67f77de6d55261092))
* **parser:** use nutmeg format for spectre output ([#289](https://github.com/ucb-substrate/substrate2/issues/289)) ([034f58f](https://github.com/ucb-substrate/substrate2/commit/034f58f99c587c61003761971e76c26038de9b3b))
* **primitives:** revamp schematic primitives ([#291](https://github.com/ucb-substrate/substrate2/issues/291)) ([e5ba06a](https://github.com/ucb-substrate/substrate2/commit/e5ba06ab10008b72e78397ad70781caa6bc61791))
* **primitives:** support parameters in SCIR primitive devices ([#232](https://github.com/ucb-substrate/substrate2/issues/232)) ([a8f5b45](https://github.com/ucb-substrate/substrate2/commit/a8f5b45a00b77d050f6a812c469e19da3305e064))
* **psf:** use PSF binary format for Spectre plugin ([#345](https://github.com/ucb-substrate/substrate2/issues/345)) ([a4ec152](https://github.com/ucb-substrate/substrate2/commit/a4ec152d5e1299bc38f2664fe900dd7d34ba8b5c))
* **refactor:** rename Has_ and Has_Data ([#282](https://github.com/ucb-substrate/substrate2/issues/282)) ([2018153](https://github.com/ucb-substrate/substrate2/commit/2018153686dd7ef3df0e10874db3c656ca245026))
* **refactor:** significantly refactor IO APIs ([#348](https://github.com/ucb-substrate/substrate2/issues/348)) ([c85d043](https://github.com/ucb-substrate/substrate2/commit/c85d04334a0ba1740f9990b91fb55ab1f2ef77c5))
* **reorg:** move substrate-api into substrate ([#155](https://github.com/ucb-substrate/substrate2/issues/155)) ([e902a1b](https://github.com/ucb-substrate/substrate2/commit/e902a1b603cca6c719770c5cd742e081bfd33e51))
* **repo:** reorganize repo ([#207](https://github.com/ucb-substrate/substrate2/issues/207)) ([54a6b43](https://github.com/ucb-substrate/substrate2/commit/54a6b43079d283a29bc0aa9e18dc6230b56fa385))
* **save-api:** add typed API for saving arbitrary signals ([#228](https://github.com/ucb-substrate/substrate2/issues/228)) ([046be02](https://github.com/ucb-substrate/substrate2/commit/046be02acbedc7fa2bb4896b92ec17babd80eee5))
* **schematic:** associated type schema and bundle primitives ([#455](https://github.com/ucb-substrate/substrate2/issues/455)) ([f5fde78](https://github.com/ucb-substrate/substrate2/commit/f5fde78824ce9ed0be494ef68d71620181bf6b48))
* **schematics:** blackboxes can reference nodes ([#208](https://github.com/ucb-substrate/substrate2/issues/208)) ([d998b4a](https://github.com/ucb-substrate/substrate2/commit/d998b4a133d47d0123768dfb3c27f8ee32ed9db9))
* **schematics:** rename `HasSchematic` and `HasSchematicImpl` ([#226](https://github.com/ucb-substrate/substrate2/issues/226)) ([a2b9c78](https://github.com/ucb-substrate/substrate2/commit/a2b9c78ea6ff56983e9a02aeafe655e92852c264))
* **scir:** uniquify names when exporting to SCIR ([#148](https://github.com/ucb-substrate/substrate2/issues/148)) ([29c2f72](https://github.com/ucb-substrate/substrate2/commit/29c2f729f5a205b144053b61c0d8c0ca2446071b))
* **sim:** allow setting temp in Spectre sims ([#401](https://github.com/ucb-substrate/substrate2/issues/401)) ([0557fce](https://github.com/ucb-substrate/substrate2/commit/0557fceb1f0da4799914b0ea4a1e0919aed97bc7))
* **simulation:** access nested nodes without strings in simulation ([#139](https://github.com/ucb-substrate/substrate2/issues/139)) ([ed7989c](https://github.com/ucb-substrate/substrate2/commit/ed7989cfb190528163a1722ae5fe3383ec3c4310))
* **simulation:** automatically generate saved data ([#457](https://github.com/ucb-substrate/substrate2/issues/457)) ([2c936d0](https://github.com/ucb-substrate/substrate2/commit/2c936d00e927b99b624f29e6450826e90f68f9bf))
* **simulation:** implement save for nested instances ([#476](https://github.com/ucb-substrate/substrate2/issues/476)) ([a47d905](https://github.com/ucb-substrate/substrate2/commit/a47d905097c6c196153b53f142ca7e1ffba5eb51))
* **simulation:** improve simulation APIs ([#320](https://github.com/ucb-substrate/substrate2/issues/320)) ([4ed59a1](https://github.com/ucb-substrate/substrate2/commit/4ed59a1283f9546e8336cc96015bd87c55682777))
* **simulation:** simplify SCIR paths for data access ([#143](https://github.com/ucb-substrate/substrate2/issues/143)) ([d42e6f9](https://github.com/ucb-substrate/substrate2/commit/d42e6f9b1d4236a9024d4a4b839319749033b8d3))
* **simulation:** support transient simulation in spectre ([#133](https://github.com/ucb-substrate/substrate2/issues/133)) ([4605862](https://github.com/ucb-substrate/substrate2/commit/460586252e3695ae32b0ab8d83b90023125d1a33))
* **simulation:** testbench schematic components ([#136](https://github.com/ucb-substrate/substrate2/issues/136)) ([97e6b0f](https://github.com/ucb-substrate/substrate2/commit/97e6b0ffd5ea7abd2a547952d5c963745854ed75))
* **slices:** use `SliceOne` instead of `Slice` where possible ([#253](https://github.com/ucb-substrate/substrate2/issues/253)) ([8eba8ed](https://github.com/ucb-substrate/substrate2/commit/8eba8ed5aad0aa4911ae31f4521d297487256087))
* **spectre:** add `global 0` to spectre netlists ([#387](https://github.com/ucb-substrate/substrate2/issues/387)) ([19257b4](https://github.com/ucb-substrate/substrate2/commit/19257b45cbdf02acb22c1408cff0d9a578d437c3))
* **spectre:** add isource (current source) ([#369](https://github.com/ucb-substrate/substrate2/issues/369)) ([f318644](https://github.com/ucb-substrate/substrate2/commit/f318644d5ae554985a22d8abf274b6a8ff9c7ec9))
* **spectre:** allow overriding spectre flags ([#443](https://github.com/ucb-substrate/substrate2/issues/443)) ([5eebbe7](https://github.com/ucb-substrate/substrate2/commit/5eebbe7d3cd0e07a8431621c564af1d626fd8e7f))
* **spectre:** allow setting global save option ([#405](https://github.com/ucb-substrate/substrate2/issues/405)) ([7836a34](https://github.com/ucb-substrate/substrate2/commit/7836a34b1677332603ec6c437e0e8468f00f6c8d))
* **spectre:** support AC simulation ([#390](https://github.com/ucb-substrate/substrate2/issues/390)) ([dc3584a](https://github.com/ucb-substrate/substrate2/commit/dc3584a50ff8ebed525566a86d82033cf87d7b29))
* **spectre:** support n-port primitives ([#410](https://github.com/ucb-substrate/substrate2/issues/410)) ([693ab82](https://github.com/ucb-substrate/substrate2/commit/693ab8287876b3cd0517d34674c3ff069da2eff8))
* **spectre:** support SPF format primitives ([#386](https://github.com/ucb-substrate/substrate2/issues/386)) ([06adc0f](https://github.com/ucb-substrate/substrate2/commit/06adc0fb155161e2f05a735fe21d2c2361cd4930))
* **spectre:** support transient noise fmax/fmin ([#411](https://github.com/ucb-substrate/substrate2/issues/411)) ([df09ef0](https://github.com/ucb-substrate/substrate2/commit/df09ef00dfc361d2d542266a82a156a4948dbb66))
* **spectre:** use APS and multithreading flags ([#395](https://github.com/ucb-substrate/substrate2/issues/395)) ([facbca6](https://github.com/ucb-substrate/substrate2/commit/facbca6087d058bb6a421d09e0ec149eba6e3456))
* **spectre:** vsource uses primitives instead of being blackboxed ([5dabcb2](https://github.com/ucb-substrate/substrate2/commit/5dabcb270cab0d259b7301d67f77de6d55261092))
* **spice:** refactor netlisting and fix voltage source netlist ([#316](https://github.com/ucb-substrate/substrate2/issues/316)) ([7a3df69](https://github.com/ucb-substrate/substrate2/commit/7a3df695cf9b38c837ff86d5a5da2417c4db7aa2))
* **stdcells:** add standard cell support to Sky130 PDK ([#323](https://github.com/ucb-substrate/substrate2/issues/323)) ([0b2048e](https://github.com/ucb-substrate/substrate2/commit/0b2048ed44d89c5de87380cac48a4bbff2b4c20a))
* **terminals:** add support for terminal paths ([#236](https://github.com/ucb-substrate/substrate2/issues/236)) ([3fba7f6](https://github.com/ucb-substrate/substrate2/commit/3fba7f6227bbf2efcaf79d849c79175e44d783a4))
* **tests:** add SCIR and SPICE netlister blackbox tests ([049a598](https://github.com/ucb-substrate/substrate2/commit/049a598e2b8d11228c63f03dc878fc4c56e036a6))
* **tests:** fix compilation and lint errors ([#482](https://github.com/ucb-substrate/substrate2/issues/482)) ([b55d04e](https://github.com/ucb-substrate/substrate2/commit/b55d04ecd2472f9f72b926ba5286f0d928bc2691))
* **tests:** reorganize tests and documentation ([#464](https://github.com/ucb-substrate/substrate2/issues/464)) ([928b9b7](https://github.com/ucb-substrate/substrate2/commit/928b9b7c45dc334ca11d86e4564edc58bf6db6f2))
* **views:** view API for improved codegen ([#463](https://github.com/ucb-substrate/substrate2/issues/463)) ([b75328c](https://github.com/ucb-substrate/substrate2/commit/b75328c9a4840ed9200a9035e28e27ac9265770f))
* **waveform:** support generic waveform datatypes ([#379](https://github.com/ucb-substrate/substrate2/issues/379)) ([93e59fd](https://github.com/ucb-substrate/substrate2/commit/93e59fd8c005e2f7f2aeece9a637dff337e4ce68))
* **windows:** fix issues for windows ([#197](https://github.com/ucb-substrate/substrate2/issues/197)) ([008b607](https://github.com/ucb-substrate/substrate2/commit/008b607b2c21c14ac3106dca6eb74d806131ef8f))


### Bug Fixes

* **deps:** bump rust to version 1.75.0 ([#362](https://github.com/ucb-substrate/substrate2/issues/362)) ([e1e82c9](https://github.com/ucb-substrate/substrate2/commit/e1e82c94cdf6ba4426f3f73f29dca40674a7f064))
* **deps:** remove opacity from substrate and deps ([#288](https://github.com/ucb-substrate/substrate2/issues/288)) ([a8c97b3](https://github.com/ucb-substrate/substrate2/commit/a8c97b30b4d075343903fa580437e9a099a745a2))
* **deps:** update rust crate rust_decimal to 1.31 ([#219](https://github.com/ucb-substrate/substrate2/issues/219)) ([6f596d5](https://github.com/ucb-substrate/substrate2/commit/6f596d5c46dc1bf045a1b8a5ef727adbc3b147cf))
* **deps:** update rust crate rust_decimal to 1.32 ([#296](https://github.com/ucb-substrate/substrate2/issues/296)) ([a2fe877](https://github.com/ucb-substrate/substrate2/commit/a2fe877d03d3f907f348d7711a2132194ae91034))
* **deps:** update rust crate rust_decimal_macros to 1.31 ([#220](https://github.com/ucb-substrate/substrate2/issues/220)) ([72147d3](https://github.com/ucb-substrate/substrate2/commit/72147d385368e2bd302821c981dd75209aa87dcb))
* **deps:** update rust crate rust_decimal_macros to 1.32 ([#297](https://github.com/ucb-substrate/substrate2/issues/297)) ([5474cc8](https://github.com/ucb-substrate/substrate2/commit/5474cc8778b81c30b34fc7d146eec6e5e2532a26))
* **dspf:** add derives to dspf types ([#409](https://github.com/ucb-substrate/substrate2/issues/409)) ([81f00cd](https://github.com/ucb-substrate/substrate2/commit/81f00cde52a12fc1b96c007d556da55eafc4d0be))
* **netlisting:** update release manifests ([506837f](https://github.com/ucb-substrate/substrate2/commit/506837fca0f1964ebc622df970575b321456ab68))
* **scir:** add additional functionality for SCIR and SPICE libraries ([#337](https://github.com/ucb-substrate/substrate2/issues/337)) ([e49f075](https://github.com/ucb-substrate/substrate2/commit/e49f07529273c38cc8ec9ae1a5020ae48fb2a202))
* **simulation:** add missing SPICE functionality and update Sky 130 PDK ([#336](https://github.com/ucb-substrate/substrate2/issues/336)) ([f802be5](https://github.com/ucb-substrate/substrate2/commit/f802be5bf0361c38b415d976dbb0f2c984a2e304))
* **simulation:** standardize ngspice and spectre transient data formats ([#327](https://github.com/ucb-substrate/substrate2/issues/327)) ([0aa42d6](https://github.com/ucb-substrate/substrate2/commit/0aa42d6000d28a8aecb655e06330f4545e155b9b))
* **spectre:** escape ports in subckt declarations ([#441](https://github.com/ucb-substrate/substrate2/issues/441)) ([3eae4ad](https://github.com/ucb-substrate/substrate2/commit/3eae4adac5b03a326724d16bee722df6c4ec7cf2))
* **spectre:** make monte carlo return vec of analysis outputs ([#388](https://github.com/ucb-substrate/substrate2/issues/388)) ([01c382d](https://github.com/ucb-substrate/substrate2/commit/01c382d908939327bd9c1344be9d928524cba021))
* **spectre:** use default number of threads ([#414](https://github.com/ucb-substrate/substrate2/issues/414)) ([748c9e4](https://github.com/ucb-substrate/substrate2/commit/748c9e42c4a922a6f858d44291fafdceb1c1e11d))
* **waveform:** fix spectre PWL waveform netlisting ([#380](https://github.com/ucb-substrate/substrate2/issues/380)) ([a47d55c](https://github.com/ucb-substrate/substrate2/commit/a47d55cca56d2359a3f0522a2c9ed8205bbb49e3))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * cache bumped from 0.7.0 to 0.8.0
    * scir bumped from 0.9.0 to 0.10.0
    * substrate bumped from 0.10.0 to 0.11.0
    * spice bumped from 0.9.0 to 0.10.0
    * type_dispatch bumped from 0.5.0 to 0.6.0
</details>

<details><summary>spice: 0.10.0</summary>

## [0.10.0](https://github.com/ucb-substrate/substrate2/compare/spice-v0.9.0...spice-v0.10.0) (2025-01-23)


### Features

* **ac:** implement Save for Substrate types in Spectre AC sim, fix lints ([#471](https://github.com/ucb-substrate/substrate2/issues/471)) ([9825520](https://github.com/ucb-substrate/substrate2/commit/98255207569cc00bd9ddc35419c2df1e48f1999c))
* **bjt:** add support for BJTs ([#432](https://github.com/ucb-substrate/substrate2/issues/432)) ([e0c4516](https://github.com/ucb-substrate/substrate2/commit/e0c45162da072ea21567b8e23d11dce36b4cff17))
* **cadence:** add support for Pegasus and Quantus ([#462](https://github.com/ucb-substrate/substrate2/issues/462)) ([953e4cb](https://github.com/ucb-substrate/substrate2/commit/953e4cb761c510668f65f4825f1be3914db45e3c))
* **cdl2spice:** add CDL to SPICE conversion command line tool ([#420](https://github.com/ucb-substrate/substrate2/issues/420)) ([1edb23a](https://github.com/ucb-substrate/substrate2/commit/1edb23a7bbd45d96bbb1c11418eb0d0843b7138b))
* **cdl:** add CDL parser ([#419](https://github.com/ucb-substrate/substrate2/issues/419)) ([23f0dab](https://github.com/ucb-substrate/substrate2/commit/23f0dab7b7a94cbe8960371b89d15211bddf51da))
* **conv:** better error messages in schema conversions ([#440](https://github.com/ucb-substrate/substrate2/issues/440)) ([bad9503](https://github.com/ucb-substrate/substrate2/commit/bad9503b8a3b98d8e0bc19779ed45e7628164f41))
* **conversion:** convert parsed SPICE to SCIR ([#178](https://github.com/ucb-substrate/substrate2/issues/178)) ([9cb7bc3](https://github.com/ucb-substrate/substrate2/commit/9cb7bc3ba549ae12e7a59465241c848800c39363))
* **docs:** fix user docs and update dev docs ([#480](https://github.com/ucb-substrate/substrate2/issues/480)) ([f727a1e](https://github.com/ucb-substrate/substrate2/commit/f727a1e7bd2a795ace1c51c3d6e02f3673d07a29))
* **dspf:** propagate nested nodes from DSPF instances ([#407](https://github.com/ucb-substrate/substrate2/issues/407)) ([8455bd2](https://github.com/ucb-substrate/substrate2/commit/8455bd2a523bb872dc1ce3fc0e89a185108dca3c))
* **netlister:** reduce duplicate code between spectre and SPICE netlisters ([#261](https://github.com/ucb-substrate/substrate2/issues/261)) ([5ba3623](https://github.com/ucb-substrate/substrate2/commit/5ba36230e653e4dc77819c5c50b527311768cd83))
* **netlists:** consistent Spectre/Spice netlist API ([#349](https://github.com/ucb-substrate/substrate2/issues/349)) ([2f9fabf](https://github.com/ucb-substrate/substrate2/commit/2f9fabf336fa1048d759e78834979ef892fc0bcf))
* **netlists:** use consistent ordering via indexmap ([#266](https://github.com/ucb-substrate/substrate2/issues/266)) ([f275c19](https://github.com/ucb-substrate/substrate2/commit/f275c19396ed4f7d255836822ff72b808f89cde7)), closes [#265](https://github.com/ucb-substrate/substrate2/issues/265)
* **ngspice:** create ngspice simulator ([#274](https://github.com/ucb-substrate/substrate2/issues/274)) ([0205300](https://github.com/ucb-substrate/substrate2/commit/02053006bc26d0b3d9e1d380def89836d7921857))
* **organization:** move `spice` from netlist/ to libs/ ([#174](https://github.com/ucb-substrate/substrate2/issues/174)) ([bd31a44](https://github.com/ucb-substrate/substrate2/commit/bd31a4481aef357daeb2c217dd7f403f6f882f78))
* **parser:** add support for 2-terminal diodes ([b74afa1](https://github.com/ucb-substrate/substrate2/commit/b74afa1118cbb37f6865eb8d472218658ee6f1b4))
* **parser:** be able to parse PEX netlists ([#363](https://github.com/ucb-substrate/substrate2/issues/363)) ([2e2f8ac](https://github.com/ucb-substrate/substrate2/commit/2e2f8ac229434fc0c03fce9e9f3ca1d0915b3469))
* **parser:** parse negative numbers and exponents ([#364](https://github.com/ucb-substrate/substrate2/issues/364)) ([53c01f6](https://github.com/ucb-substrate/substrate2/commit/53c01f60177d3d50e0302e24873be3e29f55aaa3))
* **pex:** magic-netgen pex mapping, reorganize pex tests ([#467](https://github.com/ucb-substrate/substrate2/issues/467)) ([e32802b](https://github.com/ucb-substrate/substrate2/commit/e32802bfc567f3dea50cc86b11576f7d6863fac2))
* **primitives:** revamp schematic primitives ([#291](https://github.com/ucb-substrate/substrate2/issues/291)) ([e5ba06a](https://github.com/ucb-substrate/substrate2/commit/e5ba06ab10008b72e78397ad70781caa6bc61791))
* **primitives:** support parameters in SCIR primitive devices ([#232](https://github.com/ucb-substrate/substrate2/issues/232)) ([a8f5b45](https://github.com/ucb-substrate/substrate2/commit/a8f5b45a00b77d050f6a812c469e19da3305e064))
* **proc-macros:** codegen for schematic hard macros ([#191](https://github.com/ucb-substrate/substrate2/issues/191)) ([50240b1](https://github.com/ucb-substrate/substrate2/commit/50240b167876873c4133315d35298b44e8eeac51))
* **refactor:** significantly refactor IO APIs ([#348](https://github.com/ucb-substrate/substrate2/issues/348)) ([c85d043](https://github.com/ucb-substrate/substrate2/commit/c85d04334a0ba1740f9990b91fb55ab1f2ef77c5))
* **repo:** reorganize repo ([#207](https://github.com/ucb-substrate/substrate2/issues/207)) ([54a6b43](https://github.com/ucb-substrate/substrate2/commit/54a6b43079d283a29bc0aa9e18dc6230b56fa385))
* **schematic:** associated type schema and bundle primitives ([#455](https://github.com/ucb-substrate/substrate2/issues/455)) ([f5fde78](https://github.com/ucb-substrate/substrate2/commit/f5fde78824ce9ed0be494ef68d71620181bf6b48))
* **schematics:** blackboxes can reference nodes ([#208](https://github.com/ucb-substrate/substrate2/issues/208)) ([d998b4a](https://github.com/ucb-substrate/substrate2/commit/d998b4a133d47d0123768dfb3c27f8ee32ed9db9))
* **scir-instances:** allow Substrate users to instantiate raw SCIR instances ([#184](https://github.com/ucb-substrate/substrate2/issues/184)) ([8fd5192](https://github.com/ucb-substrate/substrate2/commit/8fd5192fd2017ab04e9e3220612d0a132702bb2e))
* **scir:** expose port directions, update docs ([#426](https://github.com/ucb-substrate/substrate2/issues/426)) ([fd883b7](https://github.com/ucb-substrate/substrate2/commit/fd883b7ca803f7b45d4736a7b4b460e602b84704))
* **simulation:** automatically generate saved data ([#457](https://github.com/ucb-substrate/substrate2/issues/457)) ([2c936d0](https://github.com/ucb-substrate/substrate2/commit/2c936d00e927b99b624f29e6450826e90f68f9bf))
* **simulation:** improve simulation APIs ([#320](https://github.com/ucb-substrate/substrate2/issues/320)) ([4ed59a1](https://github.com/ucb-substrate/substrate2/commit/4ed59a1283f9546e8336cc96015bd87c55682777))
* **slices:** use `SliceOne` instead of `Slice` where possible ([#253](https://github.com/ucb-substrate/substrate2/issues/253)) ([8eba8ed](https://github.com/ucb-substrate/substrate2/commit/8eba8ed5aad0aa4911ae31f4521d297487256087))
* **spice-parser:** spice parser follows include directives ([#229](https://github.com/ucb-substrate/substrate2/issues/229)) ([5259acf](https://github.com/ucb-substrate/substrate2/commit/5259acfa703c3879d44d324279293278c46f1ff5))
* **spice-to-scir:** do not convert blackboxed subcircuits ([#179](https://github.com/ucb-substrate/substrate2/issues/179)) ([c501313](https://github.com/ucb-substrate/substrate2/commit/c501313334279b636f1d8b581357dd805177f1ca))
* **spice:** add `RawInstanceWithCell` primitive ([#384](https://github.com/ucb-substrate/substrate2/issues/384)) ([847d76b](https://github.com/ucb-substrate/substrate2/commit/847d76b2a92265faf7b8bbd079f126d1b1ba4802))
* **spice:** refactor netlisting and fix voltage source netlist ([#316](https://github.com/ucb-substrate/substrate2/issues/316)) ([7a3df69](https://github.com/ucb-substrate/substrate2/commit/7a3df695cf9b38c837ff86d5a5da2417c4db7aa2))
* **stdcells:** add standard cell support to Sky130 PDK ([#323](https://github.com/ucb-substrate/substrate2/issues/323)) ([0b2048e](https://github.com/ucb-substrate/substrate2/commit/0b2048ed44d89c5de87380cac48a4bbff2b4c20a))
* **terminals:** add support for terminal paths ([#236](https://github.com/ucb-substrate/substrate2/issues/236)) ([3fba7f6](https://github.com/ucb-substrate/substrate2/commit/3fba7f6227bbf2efcaf79d849c79175e44d783a4))
* **tests:** fix compilation and lint errors ([#482](https://github.com/ucb-substrate/substrate2/issues/482)) ([b55d04e](https://github.com/ucb-substrate/substrate2/commit/b55d04ecd2472f9f72b926ba5286f0d928bc2691))
* **tests:** reorganize tests and documentation ([#464](https://github.com/ucb-substrate/substrate2/issues/464)) ([928b9b7](https://github.com/ucb-substrate/substrate2/commit/928b9b7c45dc334ca11d86e4564edc58bf6db6f2))
* **validation:** create type for unvalidated SCIR library ([#252](https://github.com/ucb-substrate/substrate2/issues/252)) ([1550a22](https://github.com/ucb-substrate/substrate2/commit/1550a22b9a1c9f7cd9717feaa45d00487cc8848e))
* **validation:** SCIR driver analysis and validation ([#239](https://github.com/ucb-substrate/substrate2/issues/239)) ([5a91448](https://github.com/ucb-substrate/substrate2/commit/5a914489294bed06be1bd34aaa1036e4357d9a52))
* **views:** view API for improved codegen ([#463](https://github.com/ucb-substrate/substrate2/issues/463)) ([b75328c](https://github.com/ucb-substrate/substrate2/commit/b75328c9a4840ed9200a9035e28e27ac9265770f))


### Bug Fixes

* **cdl:** CDL parser ignores slashes ([#423](https://github.com/ucb-substrate/substrate2/issues/423)) ([e2b259f](https://github.com/ucb-substrate/substrate2/commit/e2b259f040913df5d73a81f778be43b716a4bbfc))
* **deps:** add missing `registry=substrate` for in-tree dependencies ([#517](https://github.com/ucb-substrate/substrate2/issues/517)) ([505d95c](https://github.com/ucb-substrate/substrate2/commit/505d95c17c5997166c1987cbc30e344fdd4c78fb))
* **deps:** remove opacity from spice library ([#287](https://github.com/ucb-substrate/substrate2/issues/287)) ([a45b728](https://github.com/ucb-substrate/substrate2/commit/a45b7288e240a9955d91acb437fa251fccb66b75))
* **parser:** fix bug in SPICE exponent parser ([#366](https://github.com/ucb-substrate/substrate2/issues/366)) ([4ced97a](https://github.com/ucb-substrate/substrate2/commit/4ced97a660f166837ec6f1468bc5f363a7b1a3ba))
* **scir:** add additional functionality for SCIR and SPICE libraries ([#337](https://github.com/ucb-substrate/substrate2/issues/337)) ([e49f075](https://github.com/ucb-substrate/substrate2/commit/e49f07529273c38cc8ec9ae1a5020ae48fb2a202))
* **simulation:** add missing SPICE functionality and update Sky 130 PDK ([#336](https://github.com/ucb-substrate/substrate2/issues/336)) ([f802be5](https://github.com/ucb-substrate/substrate2/commit/f802be5bf0361c38b415d976dbb0f2c984a2e304))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * scir bumped from 0.9.0 to 0.10.0
    * substrate bumped from 0.10.0 to 0.11.0
    * enumify bumped from 0.2.0 to 0.3.0
</details>

<details><summary>spice_vdivider: 0.0.0</summary>

### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * substrate bumped from 0.10.0 to 0.11.0
    * spice bumped from 0.9.0 to 0.10.0
</details>

<details><summary>substrate: 0.11.0</summary>

## [0.11.0](https://github.com/ucb-substrate/substrate2/compare/substrate-v0.10.0...substrate-v0.11.0) (2025-01-23)


### Features

* **ac:** implement Save for Substrate types in Spectre AC sim, fix lints ([#471](https://github.com/ucb-substrate/substrate2/issues/471)) ([9825520](https://github.com/ucb-substrate/substrate2/commit/98255207569cc00bd9ddc35419c2df1e48f1999c))
* **atoll:** Hierarchical ATOLL and configurable via spacing ([#374](https://github.com/ucb-substrate/substrate2/issues/374)) ([542b9a9](https://github.com/ucb-substrate/substrate2/commit/542b9a956d5c993908e33d3e707fc6bdb97d2c84))
* **atoll:** implement first cut ATOLL implementation ([#357](https://github.com/ucb-substrate/substrate2/issues/357)) ([372b927](https://github.com/ucb-substrate/substrate2/commit/372b9275c9d9c5cd58603f5a462a5e4b66b64cf7))
* **blackboxing:** support Substrate and SCIR blackboxes ([#135](https://github.com/ucb-substrate/substrate2/issues/135)) ([049a598](https://github.com/ucb-substrate/substrate2/commit/049a598e2b8d11228c63f03dc878fc4c56e036a6))
* **cache-config:** allow configuration of cache via config files ([#192](https://github.com/ucb-substrate/substrate2/issues/192)) ([0461402](https://github.com/ucb-substrate/substrate2/commit/0461402edfc1ec0886bbb25cf5471ee8480754fc))
* **cache:** implement persistent caching ([#171](https://github.com/ucb-substrate/substrate2/issues/171)) ([1f8ea24](https://github.com/ucb-substrate/substrate2/commit/1f8ea24f805085392bfd1a2067bb8774d0fa4ae4))
* **cadence:** add support for Pegasus and Quantus ([#462](https://github.com/ucb-substrate/substrate2/issues/462)) ([953e4cb](https://github.com/ucb-substrate/substrate2/commit/953e4cb761c510668f65f4825f1be3914db45e3c))
* **cdl:** add CDL parser ([#419](https://github.com/ucb-substrate/substrate2/issues/419)) ([23f0dab](https://github.com/ucb-substrate/substrate2/commit/23f0dab7b7a94cbe8960371b89d15211bddf51da))
* **codegen:** codegen for layout types, example layouts ([#469](https://github.com/ucb-substrate/substrate2/issues/469)) ([255af05](https://github.com/ucb-substrate/substrate2/commit/255af05657c01fcb0b4ff1e6eb0a54244dfeca32))
* **codegen:** derive Block macro adds required trait bounds by default ([#249](https://github.com/ucb-substrate/substrate2/issues/249)) ([892bef5](https://github.com/ucb-substrate/substrate2/commit/892bef585548264e3fcdcc2e6523a2321c6c6897))
* **codegen:** derive macro for implementing FromSaved ([#243](https://github.com/ucb-substrate/substrate2/issues/243)) ([48acae0](https://github.com/ucb-substrate/substrate2/commit/48acae0fb8915c4f968223268c92077f2deda979))
* **codegen:** implement derive proc macro for layout hard macros ([#200](https://github.com/ucb-substrate/substrate2/issues/200)) ([5138224](https://github.com/ucb-substrate/substrate2/commit/5138224013f537e678dfb20204e964852ed40ccb))
* **codegen:** insert appropriate bounds in Io, SchematicType, LayoutType proc macros ([#251](https://github.com/ucb-substrate/substrate2/issues/251)) ([33dcc79](https://github.com/ucb-substrate/substrate2/commit/33dcc797fdbeb21ad046093e655acf965fd99321))
* **corners:** require specifying corner by default ([#221](https://github.com/ucb-substrate/substrate2/issues/221)) ([4c2c3e4](https://github.com/ucb-substrate/substrate2/commit/4c2c3e4a3cd8b7e68921baf3af8b87f1da048936))
* **def:** utilities for exporting def orientations ([#434](https://github.com/ucb-substrate/substrate2/issues/434)) ([43a2b29](https://github.com/ucb-substrate/substrate2/commit/43a2b2906231cd46f08e2c4aface260d34abac62))
* **docs:** add code examples to documentation ([#65](https://github.com/ucb-substrate/substrate2/issues/65)) ([bfafd05](https://github.com/ucb-substrate/substrate2/commit/bfafd050c1b68d2e9e29e760ca3ff939e26aaeca))
* **docs:** fix user docs and update dev docs ([#480](https://github.com/ucb-substrate/substrate2/issues/480)) ([f727a1e](https://github.com/ucb-substrate/substrate2/commit/f727a1e7bd2a795ace1c51c3d6e02f3673d07a29))
* **docs:** reorganize docs and add code snippets ([#216](https://github.com/ucb-substrate/substrate2/issues/216)) ([d7c457d](https://github.com/ucb-substrate/substrate2/commit/d7c457d4e5c1d4846549a0e6df958243042285db))
* **docs:** update tutorials and revamp documentation website ([#315](https://github.com/ucb-substrate/substrate2/issues/315)) ([49bdf7f](https://github.com/ucb-substrate/substrate2/commit/49bdf7ff61e2fdbf19022697d518ad7fbafb465f))
* **docs:** versioned documentation between HEAD and release ([#470](https://github.com/ucb-substrate/substrate2/issues/470)) ([968182b](https://github.com/ucb-substrate/substrate2/commit/968182bf8f8d8b4cf923c0fd66f1ca1b32b12b16))
* **dspf:** propagate nested nodes from DSPF instances ([#407](https://github.com/ucb-substrate/substrate2/issues/407)) ([8455bd2](https://github.com/ucb-substrate/substrate2/commit/8455bd2a523bb872dc1ce3fc0e89a185108dca3c))
* **errors:** add error message for unconnected scir bindings ([#365](https://github.com/ucb-substrate/substrate2/issues/365)) ([acb25d5](https://github.com/ucb-substrate/substrate2/commit/acb25d5bd555d144e1edc7d3ef5009bf3d4c8e2a))
* **executors:** executor API and local executor implementation ([#161](https://github.com/ucb-substrate/substrate2/issues/161)) ([c372d51](https://github.com/ucb-substrate/substrate2/commit/c372d511e1c67ad976dc86958c87e9ad13bdfde4))
* **executors:** LSF (bsub) executor implementation ([#162](https://github.com/ucb-substrate/substrate2/issues/162)) ([ff8404a](https://github.com/ucb-substrate/substrate2/commit/ff8404abf75e6d6a4c82109adde0bcac48b6f33f))
* **gds-import:** implement GDS to RawCell importer ([#196](https://github.com/ucb-substrate/substrate2/issues/196)) ([fc37eeb](https://github.com/ucb-substrate/substrate2/commit/fc37eeb6bac10779491b98bcadcc0eeaeb7d8ec5))
* **gds:** add support for 1D GDS paths ([#422](https://github.com/ucb-substrate/substrate2/issues/422)) ([2034f8e](https://github.com/ucb-substrate/substrate2/commit/2034f8e75d51feecbe669d95191ec0bf05de60bf))
* **gds:** add support for square endcaps ([#438](https://github.com/ucb-substrate/substrate2/issues/438)) ([662a7dd](https://github.com/ucb-substrate/substrate2/commit/662a7dd5c34b6aca8b40fb29ac5f3bc59a65d56e))
* **gds:** gds reexport test ([#199](https://github.com/ucb-substrate/substrate2/issues/199)) ([93d3cd5](https://github.com/ucb-substrate/substrate2/commit/93d3cd555c1cb4a76a8845f4401e98d327b5d674))
* **geometry:** implemented contains for polygon ([#292](https://github.com/ucb-substrate/substrate2/issues/292)) ([708053a](https://github.com/ucb-substrate/substrate2/commit/708053adfb9f3783fc03895ede7348ace51730f0))
* **grid:** add ATOLL LCM routing grid and layer stack definition ([#338](https://github.com/ucb-substrate/substrate2/issues/338)) ([1e1ad90](https://github.com/ucb-substrate/substrate2/commit/1e1ad90d02b50dd0dd15516c306971241bf30b7c))
* **ics:** spectre initial conditions ([#275](https://github.com/ucb-substrate/substrate2/issues/275)) ([ce3724e](https://github.com/ucb-substrate/substrate2/commit/ce3724e9e907f3eb3653dbf39f763865914235e3))
* **impl-dispatch:** remove impl dispatch in favor of trait bounds ([#283](https://github.com/ucb-substrate/substrate2/issues/283)) ([d954115](https://github.com/ucb-substrate/substrate2/commit/d9541152db52aebde928e41c0d800453e906d62b))
* **io:** add diff pair io ([#344](https://github.com/ucb-substrate/substrate2/issues/344)) ([556d2ef](https://github.com/ucb-substrate/substrate2/commit/556d2ef202b6b6b8469d5a92bd3d0632b41234e9))
* **io:** composable port directions and runtime connection checking ([#231](https://github.com/ucb-substrate/substrate2/issues/231)) ([e1e367a](https://github.com/ucb-substrate/substrate2/commit/e1e367a2b8940319cb4f804888746a094f06e161))
* **ios:** panic when shorting IOs ([#234](https://github.com/ucb-substrate/substrate2/issues/234)) ([62ff08c](https://github.com/ucb-substrate/substrate2/commit/62ff08cfce531a4a7446813868f9c40e15c1c120))
* **layout-api:** initial implementation of layout API ([#61](https://github.com/ucb-substrate/substrate2/issues/61)) ([c4cdac7](https://github.com/ucb-substrate/substrate2/commit/c4cdac728fd4d4ef5defb97b3c1e1660ee78d672))
* **layout:** add `Bbox` implementation for `PortGeometry` ([#382](https://github.com/ucb-substrate/substrate2/issues/382)) ([e295119](https://github.com/ucb-substrate/substrate2/commit/e295119357318b1e0398bf57393b1a7405178ce6))
* **layout:** import LayIR cells into Substrate ([#460](https://github.com/ucb-substrate/substrate2/issues/460)) ([d623e4c](https://github.com/ucb-substrate/substrate2/commit/d623e4ccc5a9b555b49e59ae2f1d529d6c02299e))
* **layout:** rename `HasLayout` and `HasLayoutImpl` ([#227](https://github.com/ucb-substrate/substrate2/issues/227)) ([2cf1f7d](https://github.com/ucb-substrate/substrate2/commit/2cf1f7d435549df26ff15370e7324e9df76e0e4f))
* **layout:** simplified layout API, LayIR integration ([#459](https://github.com/ucb-substrate/substrate2/issues/459)) ([183d347](https://github.com/ucb-substrate/substrate2/commit/183d347c19e6fe98cf870be4716e7249f23bd423))
* **layouts:** support exporting layouts with multiple top cells ([#425](https://github.com/ucb-substrate/substrate2/issues/425)) ([991e467](https://github.com/ucb-substrate/substrate2/commit/991e4676d81d23c4e618991a5cadbb71e8df7c8e))
* **lut:** add basic 1D and 2D lookup tables ([#396](https://github.com/ucb-substrate/substrate2/issues/396)) ([b6c945a](https://github.com/ucb-substrate/substrate2/commit/b6c945a6e595f3df53de788da9967cb5e07be622))
* **macros:** refactor derive NestedData, start organizing tests ([#477](https://github.com/ucb-substrate/substrate2/issues/477)) ([aca48ef](https://github.com/ucb-substrate/substrate2/commit/aca48ef7a49c959e35ec4614345a55e667ff5146))
* **macros:** refactor macro reexports ([#250](https://github.com/ucb-substrate/substrate2/issues/250)) ([a332717](https://github.com/ucb-substrate/substrate2/commit/a332717e549fdea50306067e1c92dc60293aed4c))
* **montecarlo:** add Monte Carlo simulation support to Spectre plugin ([#347](https://github.com/ucb-substrate/substrate2/issues/347)) ([cc9dfe4](https://github.com/ucb-substrate/substrate2/commit/cc9dfe42db5be1a8aaeaf3fb81992a0ad7251ef8))
* **mos:** add sky130pdk transistor blocks ([#126](https://github.com/ucb-substrate/substrate2/issues/126)) ([3e9ee79](https://github.com/ucb-substrate/substrate2/commit/3e9ee7935e030ca3e5c4d56f19ccafc27445a6f0))
* **mos:** add standard 4-terminal MosIo ([8678716](https://github.com/ucb-substrate/substrate2/commit/86787160c49a1ac7c011d08ce1b9d7851bdfa0d8))
* **mos:** layout for sky130 1.8V nmos/pmos, fix geometry macros ([#478](https://github.com/ucb-substrate/substrate2/issues/478)) ([55f17b7](https://github.com/ucb-substrate/substrate2/commit/55f17b72ab90e12efb57d97fdad6b4e5373c30e2))
* **netlister:** reduce duplicate code between spectre and SPICE netlisters ([#261](https://github.com/ucb-substrate/substrate2/issues/261)) ([5ba3623](https://github.com/ucb-substrate/substrate2/commit/5ba36230e653e4dc77819c5c50b527311768cd83))
* **netlists:** consistent Spectre/Spice netlist API ([#349](https://github.com/ucb-substrate/substrate2/issues/349)) ([2f9fabf](https://github.com/ucb-substrate/substrate2/commit/2f9fabf336fa1048d759e78834979ef892fc0bcf))
* **netlists:** use consistent ordering via indexmap ([#266](https://github.com/ucb-substrate/substrate2/issues/266)) ([f275c19](https://github.com/ucb-substrate/substrate2/commit/f275c19396ed4f7d255836822ff72b808f89cde7)), closes [#265](https://github.com/ucb-substrate/substrate2/issues/265)
* **ngspice:** create ngspice simulator ([#274](https://github.com/ucb-substrate/substrate2/issues/274)) ([0205300](https://github.com/ucb-substrate/substrate2/commit/02053006bc26d0b3d9e1d380def89836d7921857))
* **organization:** rename substrate to substrate_api, set up codegen crate ([#67](https://github.com/ucb-substrate/substrate2/issues/67)) ([e07f099](https://github.com/ucb-substrate/substrate2/commit/e07f09949551fd08e3f58b6ffb7d9a8c67b76ae9))
* **parameters:** substrate schematic primitives support parameters ([#233](https://github.com/ucb-substrate/substrate2/issues/233)) ([5dabcb2](https://github.com/ucb-substrate/substrate2/commit/5dabcb270cab0d259b7301d67f77de6d55261092))
* **pdk:** add PDK trait and update context ([#68](https://github.com/ucb-substrate/substrate2/issues/68)) ([a8fbd14](https://github.com/ucb-substrate/substrate2/commit/a8fbd14a4b81e504c781e0656edce81853039afb))
* **pdk:** remove `PdkData` object to clean up interface ([#218](https://github.com/ucb-substrate/substrate2/issues/218)) ([1dd166a](https://github.com/ucb-substrate/substrate2/commit/1dd166a8f23e7b3c011c01b5c8527b8c5494ddea))
* **pdks:** implement `supported_pdks` macro and add examples ([#72](https://github.com/ucb-substrate/substrate2/issues/72)) ([5f4312f](https://github.com/ucb-substrate/substrate2/commit/5f4312f5220ae6023d78d8f4e585032147195a75))
* **pdks:** PDKs store the names of schematic primitives ([#185](https://github.com/ucb-substrate/substrate2/issues/185)) ([3446ba8](https://github.com/ucb-substrate/substrate2/commit/3446ba869f564f844b39ee524b52106954a293c5))
* **pex:** magic-netgen pex mapping, reorganize pex tests ([#467](https://github.com/ucb-substrate/substrate2/issues/467)) ([e32802b](https://github.com/ucb-substrate/substrate2/commit/e32802bfc567f3dea50cc86b11576f7d6863fac2))
* **polygon:** polygon implemented in geometry ([#263](https://github.com/ucb-substrate/substrate2/issues/263)) ([4508570](https://github.com/ucb-substrate/substrate2/commit/45085706a30a12f4af6c5e3f642ca55b4c32dd24))
* **primitives:** add 2-terminal capacitor primitive ([#262](https://github.com/ucb-substrate/substrate2/issues/262)) ([bc622b9](https://github.com/ucb-substrate/substrate2/commit/bc622b936a77719dbf92f76fdc3cbfbae61e9021))
* **primitives:** add built-in resistor and capacitor schematic blocks ([bc622b9](https://github.com/ucb-substrate/substrate2/commit/bc622b936a77719dbf92f76fdc3cbfbae61e9021))
* **primitives:** revamp schematic primitives ([#291](https://github.com/ucb-substrate/substrate2/issues/291)) ([e5ba06a](https://github.com/ucb-substrate/substrate2/commit/e5ba06ab10008b72e78397ad70781caa6bc61791))
* **primitives:** support parameters in SCIR primitive devices ([#232](https://github.com/ucb-substrate/substrate2/issues/232)) ([a8f5b45](https://github.com/ucb-substrate/substrate2/commit/a8f5b45a00b77d050f6a812c469e19da3305e064))
* **proc-macros:** allow missing docs on generated structs ([8678716](https://github.com/ucb-substrate/substrate2/commit/86787160c49a1ac7c011d08ce1b9d7851bdfa0d8))
* **proc-macros:** codegen for schematic hard macros ([#191](https://github.com/ucb-substrate/substrate2/issues/191)) ([50240b1](https://github.com/ucb-substrate/substrate2/commit/50240b167876873c4133315d35298b44e8eeac51))
* **proc-macros:** macros respect field and struct visibilities ([8678716](https://github.com/ucb-substrate/substrate2/commit/86787160c49a1ac7c011d08ce1b9d7851bdfa0d8))
* **proc-macros:** proc macros find substrate crate location ([#125](https://github.com/ucb-substrate/substrate2/issues/125)) ([8678716](https://github.com/ucb-substrate/substrate2/commit/86787160c49a1ac7c011d08ce1b9d7851bdfa0d8))
* **refactor:** rename Has_ and Has_Data ([#282](https://github.com/ucb-substrate/substrate2/issues/282)) ([2018153](https://github.com/ucb-substrate/substrate2/commit/2018153686dd7ef3df0e10874db3c656ca245026))
* **refactor:** significantly refactor IO APIs ([#348](https://github.com/ucb-substrate/substrate2/issues/348)) ([c85d043](https://github.com/ucb-substrate/substrate2/commit/c85d04334a0ba1740f9990b91fb55ab1f2ef77c5))
* **remote-cache:** add initial implementation of remote-cache ([#166](https://github.com/ucb-substrate/substrate2/issues/166)) ([7d90aab](https://github.com/ucb-substrate/substrate2/commit/7d90aab47c282cf90e814ffce357a1e694c0c357))
* **reorg:** move substrate-api into substrate ([#155](https://github.com/ucb-substrate/substrate2/issues/155)) ([e902a1b](https://github.com/ucb-substrate/substrate2/commit/e902a1b603cca6c719770c5cd742e081bfd33e51))
* **repo:** reorganize repo ([#207](https://github.com/ucb-substrate/substrate2/issues/207)) ([54a6b43](https://github.com/ucb-substrate/substrate2/commit/54a6b43079d283a29bc0aa9e18dc6230b56fa385))
* **save-api:** add typed API for saving arbitrary signals ([#228](https://github.com/ucb-substrate/substrate2/issues/228)) ([046be02](https://github.com/ucb-substrate/substrate2/commit/046be02acbedc7fa2bb4896b92ec17babd80eee5))
* **schematic:** associated type schema and bundle primitives ([#455](https://github.com/ucb-substrate/substrate2/issues/455)) ([f5fde78](https://github.com/ucb-substrate/substrate2/commit/f5fde78824ce9ed0be494ef68d71620181bf6b48))
* **schematic:** rename bundle traits ([#458](https://github.com/ucb-substrate/substrate2/issues/458)) ([ed98443](https://github.com/ucb-substrate/substrate2/commit/ed9844318cbd7176a781fff0076d8b3385d408b5))
* **schematics:** add `instantiate_connected_named` ([#447](https://github.com/ucb-substrate/substrate2/issues/447)) ([6c31948](https://github.com/ucb-substrate/substrate2/commit/6c31948d07b682c395a7c6188f3df6de67a3177b))
* **schematics:** allow explicit instance naming ([#444](https://github.com/ucb-substrate/substrate2/issues/444)) ([163b9eb](https://github.com/ucb-substrate/substrate2/commit/163b9eb10b895d69de8898a2951d0a64155da869))
* **schematics:** blackboxes can reference nodes ([#208](https://github.com/ucb-substrate/substrate2/issues/208)) ([d998b4a](https://github.com/ucb-substrate/substrate2/commit/d998b4a133d47d0123768dfb3c27f8ee32ed9db9))
* **schematics:** expose number of elems from ArrayData ([#381](https://github.com/ucb-substrate/substrate2/issues/381)) ([3422a39](https://github.com/ucb-substrate/substrate2/commit/3422a39bcab63ee2082e7c07a48f133c180a36ac))
* **schematics:** rename `HasSchematic` and `HasSchematicImpl` ([#226](https://github.com/ucb-substrate/substrate2/issues/226)) ([a2b9c78](https://github.com/ucb-substrate/substrate2/commit/a2b9c78ea6ff56983e9a02aeafe655e92852c264))
* **schematics:** support SCIR netlist exports with multiple top cells ([#424](https://github.com/ucb-substrate/substrate2/issues/424)) ([fc40421](https://github.com/ucb-substrate/substrate2/commit/fc40421dc973fac623133a219e092bb67ef8220a))
* **schematics:** user-specified schematic hierarchy flattening ([#222](https://github.com/ucb-substrate/substrate2/issues/222)) ([251f377](https://github.com/ucb-substrate/substrate2/commit/251f37778526d2f1c08a2b3c66f72ffe273021fa))
* **scir-instances:** allow Substrate users to instantiate raw SCIR instances ([#184](https://github.com/ucb-substrate/substrate2/issues/184)) ([8fd5192](https://github.com/ucb-substrate/substrate2/commit/8fd5192fd2017ab04e9e3220612d0a132702bb2e))
* **scir:** expose port directions, update docs ([#426](https://github.com/ucb-substrate/substrate2/issues/426)) ([fd883b7](https://github.com/ucb-substrate/substrate2/commit/fd883b7ca803f7b45d4736a7b4b460e602b84704))
* **scir:** SCIR lib imports merge only the instantiated cell ([#437](https://github.com/ucb-substrate/substrate2/issues/437)) ([7a0b285](https://github.com/ucb-substrate/substrate2/commit/7a0b285446b224569d430a2764e3a4e6d30ee031))
* **sim:** allow setting temp in Spectre sims ([#401](https://github.com/ucb-substrate/substrate2/issues/401)) ([0557fce](https://github.com/ucb-substrate/substrate2/commit/0557fceb1f0da4799914b0ea4a1e0919aed97bc7))
* **simulation:** automatically generate saved data ([#457](https://github.com/ucb-substrate/substrate2/issues/457)) ([2c936d0](https://github.com/ucb-substrate/substrate2/commit/2c936d00e927b99b624f29e6450826e90f68f9bf))
* **simulation:** implement save for nested instances ([#476](https://github.com/ucb-substrate/substrate2/issues/476)) ([a47d905](https://github.com/ucb-substrate/substrate2/commit/a47d905097c6c196153b53f142ca7e1ffba5eb51))
* **simulation:** improve simulation APIs ([#320](https://github.com/ucb-substrate/substrate2/issues/320)) ([4ed59a1](https://github.com/ucb-substrate/substrate2/commit/4ed59a1283f9546e8336cc96015bd87c55682777))
* **simulation:** proc macros for implementing Supports on tuples ([#163](https://github.com/ucb-substrate/substrate2/issues/163)) ([bf77832](https://github.com/ucb-substrate/substrate2/commit/bf778329d6e9fd317bea789d093c4c7d8790f5ac))
* **simulation:** simplify SCIR paths for data access ([#143](https://github.com/ucb-substrate/substrate2/issues/143)) ([d42e6f9](https://github.com/ucb-substrate/substrate2/commit/d42e6f9b1d4236a9024d4a4b839319749033b8d3))
* **simulation:** support transient simulation in spectre ([#133](https://github.com/ucb-substrate/substrate2/issues/133)) ([4605862](https://github.com/ucb-substrate/substrate2/commit/460586252e3695ae32b0ab8d83b90023125d1a33))
* **simulation:** testbench schematic components ([#136](https://github.com/ucb-substrate/substrate2/issues/136)) ([97e6b0f](https://github.com/ucb-substrate/substrate2/commit/97e6b0ffd5ea7abd2a547952d5c963745854ed75))
* **sky130:** Fix ATOLL plugin implementation ([#376](https://github.com/ucb-substrate/substrate2/issues/376)) ([aef1ed1](https://github.com/ucb-substrate/substrate2/commit/aef1ed10e6104d55a5fdf755ae4c26955d647a42))
* **slices:** use `SliceOne` instead of `Slice` where possible ([#253](https://github.com/ucb-substrate/substrate2/issues/253)) ([8eba8ed](https://github.com/ucb-substrate/substrate2/commit/8eba8ed5aad0aa4911ae31f4521d297487256087))
* **spectre:** support AC simulation ([#390](https://github.com/ucb-substrate/substrate2/issues/390)) ([dc3584a](https://github.com/ucb-substrate/substrate2/commit/dc3584a50ff8ebed525566a86d82033cf87d7b29))
* **spectre:** vsource uses primitives instead of being blackboxed ([5dabcb2](https://github.com/ucb-substrate/substrate2/commit/5dabcb270cab0d259b7301d67f77de6d55261092))
* **stdcells:** add standard cell support to Sky130 PDK ([#323](https://github.com/ucb-substrate/substrate2/issues/323)) ([0b2048e](https://github.com/ucb-substrate/substrate2/commit/0b2048ed44d89c5de87380cac48a4bbff2b4c20a))
* **terminals:** add support for terminal paths ([#236](https://github.com/ucb-substrate/substrate2/issues/236)) ([3fba7f6](https://github.com/ucb-substrate/substrate2/commit/3fba7f6227bbf2efcaf79d849c79175e44d783a4))
* **testing:** add test for terminal path API ([#245](https://github.com/ucb-substrate/substrate2/issues/245)) ([de55691](https://github.com/ucb-substrate/substrate2/commit/de556912ba4460a26d2b89510070976b8d8afcfe))
* **tests:** add SCIR and SPICE netlister blackbox tests ([049a598](https://github.com/ucb-substrate/substrate2/commit/049a598e2b8d11228c63f03dc878fc4c56e036a6))
* **tests:** fix compilation and lint errors ([#482](https://github.com/ucb-substrate/substrate2/issues/482)) ([b55d04e](https://github.com/ucb-substrate/substrate2/commit/b55d04ecd2472f9f72b926ba5286f0d928bc2691))
* **tests:** reorganize tests and documentation ([#464](https://github.com/ucb-substrate/substrate2/issues/464)) ([928b9b7](https://github.com/ucb-substrate/substrate2/commit/928b9b7c45dc334ca11d86e4564edc58bf6db6f2))
* **tiling:** array and grid tiling API ([#201](https://github.com/ucb-substrate/substrate2/issues/201)) ([b3b7c2b](https://github.com/ucb-substrate/substrate2/commit/b3b7c2bfb7ba72198872d0f08ded3e0bc757479d))
* **tracks:** uniform and enumerated track manager ([#295](https://github.com/ucb-substrate/substrate2/issues/295)) ([ed5cceb](https://github.com/ucb-substrate/substrate2/commit/ed5cceb27bb1fa2525c88c32e766312880390dcc))
* **transform:** default to Manhattan transformations ([#452](https://github.com/ucb-substrate/substrate2/issues/452)) ([3d8a410](https://github.com/ucb-substrate/substrate2/commit/3d8a4109febb11616d550c8cd6373e8f605b2e28))
* **transform:** make transformations use integers instead of floats ([#451](https://github.com/ucb-substrate/substrate2/issues/451)) ([aa9764e](https://github.com/ucb-substrate/substrate2/commit/aa9764e8b63b0a344d5e12ad3c678849c5c8ebea))
* **tutorial:** implement sky130 inverter layout tutorial ([#481](https://github.com/ucb-substrate/substrate2/issues/481)) ([440ab0e](https://github.com/ucb-substrate/substrate2/commit/440ab0e6ac33a8396c10f09637242efa32cfca62))
* **tutorial:** PEX testbench for inverter tutorial (open and CDS PDKs) ([#484](https://github.com/ucb-substrate/substrate2/issues/484)) ([454b169](https://github.com/ucb-substrate/substrate2/commit/454b1691862850976e4ce36baa5070bd165d957e))
* **type-dispatch:** add helper crate for dispatching types ([#225](https://github.com/ucb-substrate/substrate2/issues/225)) ([13ee1aa](https://github.com/ucb-substrate/substrate2/commit/13ee1aa1b287ed0c147549003c0af815b849577b))
* **validation:** create type for unvalidated SCIR library ([#252](https://github.com/ucb-substrate/substrate2/issues/252)) ([1550a22](https://github.com/ucb-substrate/substrate2/commit/1550a22b9a1c9f7cd9717feaa45d00487cc8848e))
* **validation:** SCIR driver analysis and validation ([#239](https://github.com/ucb-substrate/substrate2/issues/239)) ([5a91448](https://github.com/ucb-substrate/substrate2/commit/5a914489294bed06be1bd34aaa1036e4357d9a52))
* **views:** view API for improved codegen ([#463](https://github.com/ucb-substrate/substrate2/issues/463)) ([b75328c](https://github.com/ucb-substrate/substrate2/commit/b75328c9a4840ed9200a9035e28e27ac9265770f))
* **waveform:** support generic waveform datatypes ([#379](https://github.com/ucb-substrate/substrate2/issues/379)) ([93e59fd](https://github.com/ucb-substrate/substrate2/commit/93e59fd8c005e2f7f2aeece9a637dff337e4ce68))


### Bug Fixes

* **atoll:** abstract/autorouter fixes and APIs ([#398](https://github.com/ucb-substrate/substrate2/issues/398)) ([4dfac76](https://github.com/ucb-substrate/substrate2/commit/4dfac76647347ca8fc0131adb7ec5b066a1685de))
* **atoll:** Use ATOLL virtual layer for abstract bounding box ([#389](https://github.com/ucb-substrate/substrate2/issues/389)) ([d1060af](https://github.com/ucb-substrate/substrate2/commit/d1060af4c116351f0e55adc341f72b12b57b631f))
* **ci:** fix broken substrate Cargo.toml and update API release docs deploy ([#518](https://github.com/ucb-substrate/substrate2/issues/518)) ([62c3e9e](https://github.com/ucb-substrate/substrate2/commit/62c3e9edb5efbd40c36daad25587aa0894ab5dd9))
* **ci:** fix doc tests for substrate crate ([#158](https://github.com/ucb-substrate/substrate2/issues/158)) ([d7e9437](https://github.com/ucb-substrate/substrate2/commit/d7e943734b1eadfe64deabb7602f5bbf41cd8806))
* **ci:** use head_ref instead of ref and fix gdsconv version ([#498](https://github.com/ucb-substrate/substrate2/issues/498)) ([bc5d66e](https://github.com/ucb-substrate/substrate2/commit/bc5d66e5aad82ea79436e2fb3ec33e960a58f7b6))
* **codegen:** update codegen to use fewer structs ([#461](https://github.com/ucb-substrate/substrate2/issues/461)) ([c371be5](https://github.com/ucb-substrate/substrate2/commit/c371be59adebb9482095284034d41a6905c431d4))
* **deps:** bump rust to version 1.75.0 ([#362](https://github.com/ucb-substrate/substrate2/issues/362)) ([e1e82c9](https://github.com/ucb-substrate/substrate2/commit/e1e82c94cdf6ba4426f3f73f29dca40674a7f064))
* **deps:** fix dependencies and documentation ([#66](https://github.com/ucb-substrate/substrate2/issues/66)) ([a60ffc6](https://github.com/ucb-substrate/substrate2/commit/a60ffc6c5501200d56a6e76db0c1c2f7ef9cd086))
* **deps:** remove opacity from substrate and deps ([#288](https://github.com/ucb-substrate/substrate2/issues/288)) ([a8c97b3](https://github.com/ucb-substrate/substrate2/commit/a8c97b30b4d075343903fa580437e9a099a745a2))
* **docs:** fix snippet publishing ([#512](https://github.com/ucb-substrate/substrate2/issues/512)) ([456f8bf](https://github.com/ucb-substrate/substrate2/commit/456f8bfe659d4fa2a05f6d56394a6171c4fd34dd))
* **docs:** remove Cargo.tomls in CI to allow publishing of API docs ([#515](https://github.com/ucb-substrate/substrate2/issues/515)) ([2d14f50](https://github.com/ucb-substrate/substrate2/commit/2d14f50add396a1d775428b273df7d8d022aea05))
* **gds:** fix `GdsArrayRef` import ([#418](https://github.com/ucb-substrate/substrate2/issues/418)) ([51bbe93](https://github.com/ucb-substrate/substrate2/commit/51bbe93982f4278b947dce4ec5d6ce3c5fd8ad85))
* **gds:** fix GDS unit checks during import ([#397](https://github.com/ucb-substrate/substrate2/issues/397)) ([c943004](https://github.com/ucb-substrate/substrate2/commit/c943004cd479abcfdde54796e71959e2cc1511e7))
* **gds:** fix user units for GDS export ([#342](https://github.com/ucb-substrate/substrate2/issues/342)) ([d7c25c0](https://github.com/ucb-substrate/substrate2/commit/d7c25c00fe1e171ddc6dacfb816d0b85e74fd761))
* **gds:** use u16 instead of u8 for GDS layerspecs ([#339](https://github.com/ucb-substrate/substrate2/issues/339)) ([4d1fce2](https://github.com/ucb-substrate/substrate2/commit/4d1fce25f9493c6975d43dba96ccaa4c0cf4a686))
* **generics:** change `Deserialize&lt;'static&gt;` bounds to `DeserializeOwned` ([#259](https://github.com/ucb-substrate/substrate2/issues/259)) ([8015063](https://github.com/ucb-substrate/substrate2/commit/80150630b094a04a75cfc5b681255b80caf4f895))
* **io:** schematic nodes should not be Default ([#378](https://github.com/ucb-substrate/substrate2/issues/378)) ([863da3c](https://github.com/ucb-substrate/substrate2/commit/863da3cd3fbd27dd0b3bca1ba67f98c77b1f89d4))
* **layout:** fix issues in GDS export and ATOLL API ([#341](https://github.com/ucb-substrate/substrate2/issues/341)) ([08930b1](https://github.com/ucb-substrate/substrate2/commit/08930b1b25d018c20758986e206dc8882df782af))
* **re-exports:** move all re-exports to substrate ([#132](https://github.com/ucb-substrate/substrate2/issues/132)) ([8b3d867](https://github.com/ucb-substrate/substrate2/commit/8b3d867c7b76a16f422a38a04f5643eb050f14e6))
* **schematic:** correctly deduplicate SCIR cell names during export ([#435](https://github.com/ucb-substrate/substrate2/issues/435)) ([48af6fc](https://github.com/ucb-substrate/substrate2/commit/48af6fcd360fe9f2e8246ed0198945bfbae72724))
* **schematics:** add derives for ConvertedNodePath ([#399](https://github.com/ucb-substrate/substrate2/issues/399)) ([d50848b](https://github.com/ucb-substrate/substrate2/commit/d50848b9fe4911d127278359109e930b177cd367))
* **schematics:** clean up SCIR export code ([#224](https://github.com/ucb-substrate/substrate2/issues/224)) ([79d6501](https://github.com/ucb-substrate/substrate2/commit/79d6501f855fc3410f63c2355596c535584e5922))
* **schematics:** fix bugs with instance naming, cell ID allocation ([#445](https://github.com/ucb-substrate/substrate2/issues/445)) ([e7da085](https://github.com/ucb-substrate/substrate2/commit/e7da08583fefe96625017d32c03fc3cdd39aa9b4))
* **scir:** remove use of opacity from SCIR ([#286](https://github.com/ucb-substrate/substrate2/issues/286)) ([5e38b28](https://github.com/ucb-substrate/substrate2/commit/5e38b288629b5f2d6d3ca372418a331b6bd98e5e))
* **sim:** add `Sky130CommercialSchema` and simplify trait bounds ([#351](https://github.com/ucb-substrate/substrate2/issues/351)) ([c95e5c0](https://github.com/ucb-substrate/substrate2/commit/c95e5c08e5fc3bf6e34e00731ab4e38e9e586c01))
* **waveform:** fix waveform `sample_at` bugs ([#442](https://github.com/ucb-substrate/substrate2/issues/442)) ([dac7b53](https://github.com/ucb-substrate/substrate2/commit/dac7b5367f6890c9917952ef56a8e72be8fe5077))
* **waveforms:** add derive implementations to `WaveformRef` ([#394](https://github.com/ucb-substrate/substrate2/issues/394)) ([fd016a5](https://github.com/ucb-substrate/substrate2/commit/fd016a58d4d0c8046150bdb7e57d4566d33975ac))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * config bumped from 0.4.0 to 0.5.0
    * snippets bumped from 0.7.0 to 0.8.0
    * cache bumped from 0.7.0 to 0.8.0
    * codegen bumped from 0.10.0 to 0.11.0
    * layir bumped from 0.2.0 to 0.3.0
    * geometry bumped from 0.7.0 to 0.8.0
    * gds bumped from 0.4.0 to 0.5.0
    * gdsconv bumped from 0.2.0 to 0.3.0
    * enumify bumped from 0.2.0 to 0.3.0
    * scir bumped from 0.9.0 to 0.10.0
    * pathtree bumped from 0.3.0 to 0.4.0
    * type_dispatch bumped from 0.5.0 to 0.6.0
    * uniquify bumped from 0.4.0 to 0.5.0
  * build-dependencies
    * snippets bumped from 0.7.0 to 0.8.0
    * examples bumped from 0.2.0 to 0.3.0
</details>

<details><summary>substrate_api_examples: 0.0.0</summary>

### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * substrate bumped from 0.10.0 to 0.11.0
    * scir bumped from 0.9.0 to 0.10.0
</details>

<details><summary>type_dispatch: 0.6.0</summary>

## [0.6.0](https://github.com/ucb-substrate/substrate2/compare/type_dispatch-v0.5.0...type_dispatch-v0.6.0) (2025-01-23)


### Features

* **codegen:** insert appropriate bounds in Io, SchematicType, LayoutType proc macros ([#251](https://github.com/ucb-substrate/substrate2/issues/251)) ([33dcc79](https://github.com/ucb-substrate/substrate2/commit/33dcc797fdbeb21ad046093e655acf965fd99321))
* **grid:** add ATOLL LCM routing grid and layer stack definition ([#338](https://github.com/ucb-substrate/substrate2/issues/338)) ([1e1ad90](https://github.com/ucb-substrate/substrate2/commit/1e1ad90d02b50dd0dd15516c306971241bf30b7c))
* **macros:** refactor macro reexports ([#250](https://github.com/ucb-substrate/substrate2/issues/250)) ([a332717](https://github.com/ucb-substrate/substrate2/commit/a332717e549fdea50306067e1c92dc60293aed4c))
* **terminals:** add support for terminal paths ([#236](https://github.com/ucb-substrate/substrate2/issues/236)) ([3fba7f6](https://github.com/ucb-substrate/substrate2/commit/3fba7f6227bbf2efcaf79d849c79175e44d783a4))
* **tests:** reorganize tests and documentation ([#464](https://github.com/ucb-substrate/substrate2/issues/464)) ([928b9b7](https://github.com/ucb-substrate/substrate2/commit/928b9b7c45dc334ca11d86e4564edc58bf6db6f2))
* **type-dispatch:** add helper crate for dispatching types ([#225](https://github.com/ucb-substrate/substrate2/issues/225)) ([13ee1aa](https://github.com/ucb-substrate/substrate2/commit/13ee1aa1b287ed0c147549003c0af815b849577b))


### Bug Fixes

* **deps:** add missing `registry=substrate` for in-tree dependencies ([#517](https://github.com/ucb-substrate/substrate2/issues/517)) ([505d95c](https://github.com/ucb-substrate/substrate2/commit/505d95c17c5997166c1987cbc30e344fdd4c78fb))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * type_dispatch_macros bumped from 0.4.0 to 0.5.0
</details>

<details><summary>type_dispatch_macros: 0.5.0</summary>

## [0.5.0](https://github.com/ucb-substrate/substrate2/compare/type_dispatch_macros-v0.4.0...type_dispatch_macros-v0.5.0) (2025-01-23)


### Features

* **macros:** refactor macro reexports ([#250](https://github.com/ucb-substrate/substrate2/issues/250)) ([a332717](https://github.com/ucb-substrate/substrate2/commit/a332717e549fdea50306067e1c92dc60293aed4c))
* **terminals:** add support for terminal paths ([#236](https://github.com/ucb-substrate/substrate2/issues/236)) ([3fba7f6](https://github.com/ucb-substrate/substrate2/commit/3fba7f6227bbf2efcaf79d849c79175e44d783a4))
* **type-dispatch:** add helper crate for dispatching types ([#225](https://github.com/ucb-substrate/substrate2/issues/225)) ([13ee1aa](https://github.com/ucb-substrate/substrate2/commit/13ee1aa1b287ed0c147549003c0af815b849577b))


### Dependencies

* The following workspace dependencies were updated
  * dev-dependencies
    * type_dispatch bumped from <=0.5.0 to <=0.6.0
</details>

<details><summary>uniquify: 0.5.0</summary>

## [0.5.0](https://github.com/ucb-substrate/substrate2/compare/uniquify-v0.4.0...uniquify-v0.5.0) (2025-01-23)


### Features

* **layir:** initial LayIR implementation ([#456](https://github.com/ucb-substrate/substrate2/issues/456)) ([4f76d41](https://github.com/ucb-substrate/substrate2/commit/4f76d41c86fd0c57e525f40c976b5eeb0bbd4c68))
* **merging:** add API for merging two SCIR libraries ([#183](https://github.com/ucb-substrate/substrate2/issues/183)) ([a0006aa](https://github.com/ucb-substrate/substrate2/commit/a0006aa4dbe62c2dda66eea306987e56eaabe181))
* **scir:** uniquify names when exporting to SCIR ([#148](https://github.com/ucb-substrate/substrate2/issues/148)) ([29c2f72](https://github.com/ucb-substrate/substrate2/commit/29c2f729f5a205b144053b61c0d8c0ca2446071b))
* **uniquify:** create uniquify crate for assigning unique names ([#147](https://github.com/ucb-substrate/substrate2/issues/147)) ([d4b83be](https://github.com/ucb-substrate/substrate2/commit/d4b83be335047052f0cf6ea2bddcdb64ce3141c4))
</details>

<details><summary>vdivider: 0.0.0</summary>

### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * substrate bumped from 0.10.0 to 0.11.0
    * spectre bumped from 0.11.0 to 0.12.0
</details>

<details><summary>verilog: 0.3.0</summary>

## [0.3.0](https://github.com/ucb-substrate/substrate2/compare/verilog-v0.2.0...verilog-v0.3.0) (2025-01-23)


### Features

* **verilog:** add helpers for exporting verilog shells ([#427](https://github.com/ucb-substrate/substrate2/issues/427)) ([0cb8695](https://github.com/ucb-substrate/substrate2/commit/0cb8695be31fac131b5df106508cd0546eb96b45))
* **verilog:** support exporting all cells ([#428](https://github.com/ucb-substrate/substrate2/issues/428)) ([4d5498a](https://github.com/ucb-substrate/substrate2/commit/4d5498a3467cd54af9a0abe7afc53e0c356e781f))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * scir bumped from 0.9.0 to 0.10.0
</details>

<details><summary>via: 0.0.0</summary>

### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * substrate bumped from 0.10.0 to 0.11.0
    * layir bumped from 0.2.0 to 0.3.0
</details>

---
This PR was generated with [Release Please](https://github.com/googleapis/release-please). See [documentation](https://github.com/googleapis/release-please#release-please).