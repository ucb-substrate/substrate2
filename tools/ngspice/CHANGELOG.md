# Changelog

## [0.3.1](https://github.com/ucb-substrate/substrate2/compare/ngspice-v0.3.0...ngspice-v0.3.1) (2023-11-26)


### Bug Fixes

* **simulation:** standardize ngspice and spectre transient data formats ([#327](https://github.com/ucb-substrate/substrate2/issues/327)) ([0aa42d6](https://github.com/ucb-substrate/substrate2/commit/0aa42d6000d28a8aecb655e06330f4545e155b9b))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * substrate bumped from 0.8.0 to 0.8.1
    * spice bumped from 0.7.0 to 0.7.1

## [0.3.0](https://github.com/ucb-substrate/substrate2/compare/ngspice-v0.2.0...ngspice-v0.3.0) (2023-11-25)


### Features

* **simulation:** improve simulation APIs ([#320](https://github.com/ucb-substrate/substrate2/issues/320)) ([4ed59a1](https://github.com/ucb-substrate/substrate2/commit/4ed59a1283f9546e8336cc96015bd87c55682777))
* **stdcells:** add standard cell support to Sky130 PDK ([#323](https://github.com/ucb-substrate/substrate2/issues/323)) ([0b2048e](https://github.com/ucb-substrate/substrate2/commit/0b2048ed44d89c5de87380cac48a4bbff2b4c20a))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * cache bumped from 0.4.0 to 0.5.0
    * substrate bumped from 0.7.1 to 0.8.0
    * spice bumped from 0.6.0 to 0.7.0

## [0.2.0](https://github.com/ucb-substrate/substrate2/compare/ngspice-v0.1.0...ngspice-v0.2.0) (2023-11-04)


### Features

* **spice:** refactor netlisting and fix voltage source netlist ([#316](https://github.com/ucb-substrate/substrate2/issues/316)) ([7a3df69](https://github.com/ucb-substrate/substrate2/commit/7a3df695cf9b38c837ff86d5a5da2417c4db7aa2))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * scir bumped from 0.6.0 to 0.7.0
    * substrate bumped from 0.7.0 to 0.7.1
    * spice bumped from 0.5.0 to 0.6.0

## 0.1.0 (2023-11-02)


### Features

* **ngspice:** create ngspice simulator ([#274](https://github.com/ucb-substrate/substrate2/issues/274)) ([0205300](https://github.com/ucb-substrate/substrate2/commit/02053006bc26d0b3d9e1d380def89836d7921857))
* **parser:** use nutmeg format for spectre output ([#289](https://github.com/ucb-substrate/substrate2/issues/289)) ([034f58f](https://github.com/ucb-substrate/substrate2/commit/034f58f99c587c61003761971e76c26038de9b3b))
* **primitives:** revamp schematic primitives ([#291](https://github.com/ucb-substrate/substrate2/issues/291)) ([e5ba06a](https://github.com/ucb-substrate/substrate2/commit/e5ba06ab10008b72e78397ad70781caa6bc61791))
* **refactor:** rename Has_ and Has_Data ([#282](https://github.com/ucb-substrate/substrate2/issues/282)) ([2018153](https://github.com/ucb-substrate/substrate2/commit/2018153686dd7ef3df0e10874db3c656ca245026))


### Bug Fixes

* **deps:** remove opacity from substrate and deps ([#288](https://github.com/ucb-substrate/substrate2/issues/288)) ([a8c97b3](https://github.com/ucb-substrate/substrate2/commit/a8c97b30b4d075343903fa580437e9a099a745a2))
* **deps:** update rust crate rust_decimal to 1.32 ([#296](https://github.com/ucb-substrate/substrate2/issues/296)) ([a2fe877](https://github.com/ucb-substrate/substrate2/commit/a2fe877d03d3f907f348d7711a2132194ae91034))
* **deps:** update rust crate rust_decimal_macros to 1.32 ([#297](https://github.com/ucb-substrate/substrate2/issues/297)) ([5474cc8](https://github.com/ucb-substrate/substrate2/commit/5474cc8778b81c30b34fc7d146eec6e5e2532a26))
* ngspice tests ([#310](https://github.com/ucb-substrate/substrate2/issues/310)) ([62e16bd](https://github.com/ucb-substrate/substrate2/commit/62e16bdf296a6150066369f6465f49d299d86842))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * cache bumped from 0.3.1 to 0.4.0
    * scir bumped from 0.5.0 to 0.6.0
    * substrate bumped from 0.6.1 to 0.7.0
    * nutlex bumped from 0.1.0 to 0.2.0
    * spice bumped from 0.4.0 to 0.5.0
