# Changelog

* The following workspace dependencies were updated
  * dependencies
    * codegen bumped from 0.1.0 to 0.1.1
    * substrate_api bumped from 0.1.0 to 0.1.1

* The following workspace dependencies were updated
  * dependencies
    * examples bumped from 0.4.0 to 0.4.1
    * codegen bumped from 0.7.0 to 0.7.1
    * scir bumped from 0.6.0 to 0.7.0
  * dev-dependencies
    * sky130pdk bumped from <=0.7.0 to <=0.7.1
    * spectre bumped from <=0.7.0 to <=0.8.0
    * spice bumped from <=0.5.0 to <=0.6.0

* The following workspace dependencies were updated
  * dependencies
    * examples bumped from 0.5.0 to 0.5.1
    * codegen bumped from 0.8.0 to 0.8.1
  * dev-dependencies
    * sky130pdk bumped from <=0.8.0 to <=0.8.1
    * spectre bumped from <=0.9.0 to <=0.9.1
    * spice bumped from <=0.7.0 to <=0.7.1

## [0.8.0](https://github.com/ucb-substrate/substrate2/compare/substrate-v0.7.1...substrate-v0.8.0) (2023-11-25)


### Features

* **docs:** update tutorials and revamp documentation website ([#315](https://github.com/ucb-substrate/substrate2/issues/315)) ([49bdf7f](https://github.com/ucb-substrate/substrate2/commit/49bdf7ff61e2fdbf19022697d518ad7fbafb465f))
* **simulation:** improve simulation APIs ([#320](https://github.com/ucb-substrate/substrate2/issues/320)) ([4ed59a1](https://github.com/ucb-substrate/substrate2/commit/4ed59a1283f9546e8336cc96015bd87c55682777))
* **stdcells:** add standard cell support to Sky130 PDK ([#323](https://github.com/ucb-substrate/substrate2/issues/323)) ([0b2048e](https://github.com/ucb-substrate/substrate2/commit/0b2048ed44d89c5de87380cac48a4bbff2b4c20a))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * config bumped from 0.2.4 to 0.2.5
    * examples bumped from 0.4.1 to 0.5.0
    * cache bumped from 0.4.0 to 0.5.0
    * codegen bumped from 0.7.1 to 0.8.0
  * dev-dependencies
    * sky130pdk bumped from <=0.7.1 to <=0.8.0
    * spectre bumped from <=0.8.0 to <=0.9.0
    * spice bumped from <=0.6.0 to <=0.7.0

## [0.7.0](https://github.com/ucb-substrate/substrate2/compare/substrate-v0.6.1...substrate-v0.7.0) (2023-11-02)


### Features

* **geometry:** implemented contains for polygon ([#292](https://github.com/ucb-substrate/substrate2/issues/292)) ([708053a](https://github.com/ucb-substrate/substrate2/commit/708053adfb9f3783fc03895ede7348ace51730f0))
* **ics:** spectre initial conditions ([#275](https://github.com/ucb-substrate/substrate2/issues/275)) ([ce3724e](https://github.com/ucb-substrate/substrate2/commit/ce3724e9e907f3eb3653dbf39f763865914235e3))
* **impl-dispatch:** remove impl dispatch in favor of trait bounds ([#283](https://github.com/ucb-substrate/substrate2/issues/283)) ([d954115](https://github.com/ucb-substrate/substrate2/commit/d9541152db52aebde928e41c0d800453e906d62b))
* **netlister:** reduce duplicate code between spectre and SPICE netlisters ([#261](https://github.com/ucb-substrate/substrate2/issues/261)) ([5ba3623](https://github.com/ucb-substrate/substrate2/commit/5ba36230e653e4dc77819c5c50b527311768cd83))
* **netlists:** use consistent ordering via indexmap ([#266](https://github.com/ucb-substrate/substrate2/issues/266)) ([f275c19](https://github.com/ucb-substrate/substrate2/commit/f275c19396ed4f7d255836822ff72b808f89cde7)), closes [#265](https://github.com/ucb-substrate/substrate2/issues/265)
* **ngspice:** create ngspice simulator ([#274](https://github.com/ucb-substrate/substrate2/issues/274)) ([0205300](https://github.com/ucb-substrate/substrate2/commit/02053006bc26d0b3d9e1d380def89836d7921857))
* **polygon:** polygon implemented in geometry ([#263](https://github.com/ucb-substrate/substrate2/issues/263)) ([4508570](https://github.com/ucb-substrate/substrate2/commit/45085706a30a12f4af6c5e3f642ca55b4c32dd24))
* **primitives:** add 2-terminal capacitor primitive ([#262](https://github.com/ucb-substrate/substrate2/issues/262)) ([bc622b9](https://github.com/ucb-substrate/substrate2/commit/bc622b936a77719dbf92f76fdc3cbfbae61e9021))
* **primitives:** add built-in resistor and capacitor schematic blocks ([bc622b9](https://github.com/ucb-substrate/substrate2/commit/bc622b936a77719dbf92f76fdc3cbfbae61e9021))
* **primitives:** revamp schematic primitives ([#291](https://github.com/ucb-substrate/substrate2/issues/291)) ([e5ba06a](https://github.com/ucb-substrate/substrate2/commit/e5ba06ab10008b72e78397ad70781caa6bc61791))
* **refactor:** rename Has_ and Has_Data ([#282](https://github.com/ucb-substrate/substrate2/issues/282)) ([2018153](https://github.com/ucb-substrate/substrate2/commit/2018153686dd7ef3df0e10874db3c656ca245026))
* **tracks:** uniform and enumerated track manager ([#295](https://github.com/ucb-substrate/substrate2/issues/295)) ([ed5cceb](https://github.com/ucb-substrate/substrate2/commit/ed5cceb27bb1fa2525c88c32e766312880390dcc))


### Bug Fixes

* **deps:** remove opacity from substrate and deps ([#288](https://github.com/ucb-substrate/substrate2/issues/288)) ([a8c97b3](https://github.com/ucb-substrate/substrate2/commit/a8c97b30b4d075343903fa580437e9a099a745a2))
* **scir:** remove use of opacity from SCIR ([#286](https://github.com/ucb-substrate/substrate2/issues/286)) ([5e38b28](https://github.com/ucb-substrate/substrate2/commit/5e38b288629b5f2d6d3ca372418a331b6bd98e5e))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * config bumped from 0.2.3 to 0.2.4
    * examples bumped from 0.3.1 to 0.4.0
    * cache bumped from 0.3.1 to 0.4.0
    * codegen bumped from 0.6.1 to 0.7.0
    * geometry bumped from 0.4.0 to 0.5.0
    * enumify bumped from 0.0.0 to 0.1.0
    * scir bumped from 0.5.0 to 0.6.0
  * dev-dependencies
    * sky130pdk bumped from <=0.6.1 to <=0.7.0
    * spectre bumped from <=0.6.1 to <=0.7.0
    * spice bumped from <=0.4.0 to <=0.5.0

## [0.6.1](https://github.com/substrate-labs/substrate2/compare/substrate-v0.6.0...substrate-v0.6.1) (2023-08-08)


### Bug Fixes

* **generics:** change `Deserialize&lt;'static&gt;` bounds to `DeserializeOwned` ([#259](https://github.com/substrate-labs/substrate2/issues/259)) ([8015063](https://github.com/substrate-labs/substrate2/commit/80150630b094a04a75cfc5b681255b80caf4f895))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * examples bumped from 0.3.0 to 0.3.1
    * codegen bumped from 0.6.0 to 0.6.1
  * dev-dependencies
    * sky130pdk bumped from <=0.6.0 to <=0.6.1
    * spectre bumped from <=0.6.0 to <=0.6.1

## [0.6.0](https://github.com/substrate-labs/substrate2/compare/substrate-v0.5.0...substrate-v0.6.0) (2023-08-08)


### Features

* **codegen:** derive Block macro adds required trait bounds by default ([#249](https://github.com/substrate-labs/substrate2/issues/249)) ([892bef5](https://github.com/substrate-labs/substrate2/commit/892bef585548264e3fcdcc2e6523a2321c6c6897))
* **codegen:** insert appropriate bounds in Io, SchematicType, LayoutType proc macros ([#251](https://github.com/substrate-labs/substrate2/issues/251)) ([33dcc79](https://github.com/substrate-labs/substrate2/commit/33dcc797fdbeb21ad046093e655acf965fd99321))
* **macros:** refactor macro reexports ([#250](https://github.com/substrate-labs/substrate2/issues/250)) ([a332717](https://github.com/substrate-labs/substrate2/commit/a332717e549fdea50306067e1c92dc60293aed4c))
* **slices:** use `SliceOne` instead of `Slice` where possible ([#253](https://github.com/substrate-labs/substrate2/issues/253)) ([8eba8ed](https://github.com/substrate-labs/substrate2/commit/8eba8ed5aad0aa4911ae31f4521d297487256087))
* **testing:** add test for terminal path API ([#245](https://github.com/substrate-labs/substrate2/issues/245)) ([de55691](https://github.com/substrate-labs/substrate2/commit/de556912ba4460a26d2b89510070976b8d8afcfe))
* **validation:** create type for unvalidated SCIR library ([#252](https://github.com/substrate-labs/substrate2/issues/252)) ([1550a22](https://github.com/substrate-labs/substrate2/commit/1550a22b9a1c9f7cd9717feaa45d00487cc8848e))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * config bumped from 0.2.2 to 0.2.3
    * examples bumped from 0.2.0 to 0.3.0
    * cache bumped from 0.3.0 to 0.3.1
    * codegen bumped from 0.5.0 to 0.6.0
    * geometry bumped from 0.3.0 to 0.4.0
    * scir bumped from 0.4.0 to 0.5.0
    * spice bumped from 0.3.0 to 0.4.0
    * type_dispatch bumped from 0.2.0 to 0.3.0
  * dev-dependencies
    * sky130pdk bumped from <=0.5.0 to <=0.6.0
    * spectre bumped from <=0.5.0 to <=0.6.0

## [0.5.0](https://github.com/substrate-labs/substrate2/compare/substrate-v0.4.0...substrate-v0.5.0) (2023-08-05)


### Features

* **codegen:** derive macro for implementing FromSaved ([#243](https://github.com/substrate-labs/substrate2/issues/243)) ([48acae0](https://github.com/substrate-labs/substrate2/commit/48acae0fb8915c4f968223268c92077f2deda979))
* **terminals:** add support for terminal paths ([#236](https://github.com/substrate-labs/substrate2/issues/236)) ([3fba7f6](https://github.com/substrate-labs/substrate2/commit/3fba7f6227bbf2efcaf79d849c79175e44d783a4))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * config bumped from 0.2.1 to 0.2.2
    * examples bumped from 0.1.0 to 0.2.0
    * cache bumped from 0.2.1 to 0.3.0
    * codegen bumped from 0.4.0 to 0.5.0
    * scir bumped from 0.3.0 to 0.4.0
    * pathtree bumped from 0.1.0 to 0.2.0
    * spice bumped from 0.2.0 to 0.3.0
    * type_dispatch bumped from 0.1.0 to 0.2.0
  * dev-dependencies
    * sky130pdk bumped from <=0.4.0 to <=0.5.0
    * spectre bumped from <=0.4.0 to <=0.5.0

## [0.4.0](https://github.com/substrate-labs/substrate2/compare/substrate-v0.3.0...substrate-v0.4.0) (2023-08-04)


### Features

* **corners:** require specifying corner by default ([#221](https://github.com/substrate-labs/substrate2/issues/221)) ([4c2c3e4](https://github.com/substrate-labs/substrate2/commit/4c2c3e4a3cd8b7e68921baf3af8b87f1da048936))
* **docs:** reorganize docs and add code snippets ([#216](https://github.com/substrate-labs/substrate2/issues/216)) ([d7c457d](https://github.com/substrate-labs/substrate2/commit/d7c457d4e5c1d4846549a0e6df958243042285db))
* **io:** composable port directions and runtime connection checking ([#231](https://github.com/substrate-labs/substrate2/issues/231)) ([e1e367a](https://github.com/substrate-labs/substrate2/commit/e1e367a2b8940319cb4f804888746a094f06e161))
* **ios:** panic when shorting IOs ([#234](https://github.com/substrate-labs/substrate2/issues/234)) ([62ff08c](https://github.com/substrate-labs/substrate2/commit/62ff08cfce531a4a7446813868f9c40e15c1c120))
* **layout:** rename `HasLayout` and `HasLayoutImpl` ([#227](https://github.com/substrate-labs/substrate2/issues/227)) ([2cf1f7d](https://github.com/substrate-labs/substrate2/commit/2cf1f7d435549df26ff15370e7324e9df76e0e4f))
* **parameters:** substrate schematic primitives support parameters ([#233](https://github.com/substrate-labs/substrate2/issues/233)) ([5dabcb2](https://github.com/substrate-labs/substrate2/commit/5dabcb270cab0d259b7301d67f77de6d55261092))
* **pdk:** remove `PdkData` object to clean up interface ([#218](https://github.com/substrate-labs/substrate2/issues/218)) ([1dd166a](https://github.com/substrate-labs/substrate2/commit/1dd166a8f23e7b3c011c01b5c8527b8c5494ddea))
* **primitives:** support parameters in SCIR primitive devices ([#232](https://github.com/substrate-labs/substrate2/issues/232)) ([a8f5b45](https://github.com/substrate-labs/substrate2/commit/a8f5b45a00b77d050f6a812c469e19da3305e064))
* **repo:** reorganize repo ([#207](https://github.com/substrate-labs/substrate2/issues/207)) ([54a6b43](https://github.com/substrate-labs/substrate2/commit/54a6b43079d283a29bc0aa9e18dc6230b56fa385))
* **save-api:** add typed API for saving arbitrary signals ([#228](https://github.com/substrate-labs/substrate2/issues/228)) ([046be02](https://github.com/substrate-labs/substrate2/commit/046be02acbedc7fa2bb4896b92ec17babd80eee5))
* **schematics:** blackboxes can reference nodes ([#208](https://github.com/substrate-labs/substrate2/issues/208)) ([d998b4a](https://github.com/substrate-labs/substrate2/commit/d998b4a133d47d0123768dfb3c27f8ee32ed9db9))
* **schematics:** rename `HasSchematic` and `HasSchematicImpl` ([#226](https://github.com/substrate-labs/substrate2/issues/226)) ([a2b9c78](https://github.com/substrate-labs/substrate2/commit/a2b9c78ea6ff56983e9a02aeafe655e92852c264))
* **schematics:** user-specified schematic hierarchy flattening ([#222](https://github.com/substrate-labs/substrate2/issues/222)) ([251f377](https://github.com/substrate-labs/substrate2/commit/251f37778526d2f1c08a2b3c66f72ffe273021fa))
* **spectre:** vsource uses primitives instead of being blackboxed ([5dabcb2](https://github.com/substrate-labs/substrate2/commit/5dabcb270cab0d259b7301d67f77de6d55261092))
* **type-dispatch:** add helper crate for dispatching types ([#225](https://github.com/substrate-labs/substrate2/issues/225)) ([13ee1aa](https://github.com/substrate-labs/substrate2/commit/13ee1aa1b287ed0c147549003c0af815b849577b))
* **validation:** SCIR driver analysis and validation ([#239](https://github.com/substrate-labs/substrate2/issues/239)) ([5a91448](https://github.com/substrate-labs/substrate2/commit/5a914489294bed06be1bd34aaa1036e4357d9a52))


### Bug Fixes

* **schematics:** clean up SCIR export code ([#224](https://github.com/substrate-labs/substrate2/issues/224)) ([79d6501](https://github.com/substrate-labs/substrate2/commit/79d6501f855fc3410f63c2355596c535584e5922))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * config bumped from 0.2.0 to 0.2.1
    * examples bumped from 0.0.0 to 0.1.0
    * cache bumped from 0.2.0 to 0.2.1
    * codegen bumped from 0.3.0 to 0.4.0
    * gds bumped from 0.2.0 to 0.3.0
    * scir bumped from 0.2.0 to 0.3.0
    * spice bumped from 0.1.0 to 0.2.0
    * type_dispatch bumped from 0.0.0 to 0.1.0
  * dev-dependencies
    * sky130pdk bumped from <=0.3.0 to <=0.4.0

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
