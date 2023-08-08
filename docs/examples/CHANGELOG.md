# Changelog

* The following workspace dependencies were updated
  * dev-dependencies
    * substrate bumped from <=0.6.0 to <=0.6.1
    * sky130pdk bumped from <=0.6.0 to <=0.6.1
    * spectre bumped from <=0.6.0 to <=0.6.1

## [0.3.0](https://github.com/substrate-labs/substrate2/compare/examples-v0.2.0...examples-v0.3.0) (2023-08-08)


### Features

* **codegen:** insert appropriate bounds in Io, SchematicType, LayoutType proc macros ([#251](https://github.com/substrate-labs/substrate2/issues/251)) ([33dcc79](https://github.com/substrate-labs/substrate2/commit/33dcc797fdbeb21ad046093e655acf965fd99321))
* **macros:** refactor macro reexports ([#250](https://github.com/substrate-labs/substrate2/issues/250)) ([a332717](https://github.com/substrate-labs/substrate2/commit/a332717e549fdea50306067e1c92dc60293aed4c))


### Dependencies

* The following workspace dependencies were updated
  * dev-dependencies
    * substrate bumped from <=0.5.0 to <=0.6.0
    * sky130pdk bumped from <=0.5.0 to <=0.6.0
    * spectre bumped from <=0.5.0 to <=0.6.0

## [0.2.0](https://github.com/substrate-labs/substrate2/compare/examples-v0.1.0...examples-v0.2.0) (2023-08-05)


### Features

* **codegen:** derive macro for implementing FromSaved ([#243](https://github.com/substrate-labs/substrate2/issues/243)) ([48acae0](https://github.com/substrate-labs/substrate2/commit/48acae0fb8915c4f968223268c92077f2deda979))


### Dependencies

* The following workspace dependencies were updated
  * dev-dependencies
    * substrate bumped from <=0.4.0 to <=0.5.0
    * sky130pdk bumped from <=0.4.0 to <=0.5.0
    * spectre bumped from <=0.4.0 to <=0.5.0

## 0.1.0 (2023-08-04)


### Features

* **corners:** require specifying corner by default ([#221](https://github.com/substrate-labs/substrate2/issues/221)) ([4c2c3e4](https://github.com/substrate-labs/substrate2/commit/4c2c3e4a3cd8b7e68921baf3af8b87f1da048936))
* **docs:** reorganize docs and add code snippets ([#216](https://github.com/substrate-labs/substrate2/issues/216)) ([d7c457d](https://github.com/substrate-labs/substrate2/commit/d7c457d4e5c1d4846549a0e6df958243042285db))
* **layout:** rename `HasLayout` and `HasLayoutImpl` ([#227](https://github.com/substrate-labs/substrate2/issues/227)) ([2cf1f7d](https://github.com/substrate-labs/substrate2/commit/2cf1f7d435549df26ff15370e7324e9df76e0e4f))
* **pdk:** remove `PdkData` object to clean up interface ([#218](https://github.com/substrate-labs/substrate2/issues/218)) ([1dd166a](https://github.com/substrate-labs/substrate2/commit/1dd166a8f23e7b3c011c01b5c8527b8c5494ddea))
* **schematics:** rename `HasSchematic` and `HasSchematicImpl` ([#226](https://github.com/substrate-labs/substrate2/issues/226)) ([a2b9c78](https://github.com/substrate-labs/substrate2/commit/a2b9c78ea6ff56983e9a02aeafe655e92852c264))
* **type-dispatch:** add helper crate for dispatching types ([#225](https://github.com/substrate-labs/substrate2/issues/225)) ([13ee1aa](https://github.com/substrate-labs/substrate2/commit/13ee1aa1b287ed0c147549003c0af815b849577b))


### Dependencies

* The following workspace dependencies were updated
  * dev-dependencies
    * substrate bumped from <=0.3.0 to <=0.4.0
    * sky130pdk bumped from <=0.3.0 to <=0.4.0
