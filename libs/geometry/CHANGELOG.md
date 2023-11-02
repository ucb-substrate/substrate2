# Changelog

## [0.5.0](https://github.com/ucb-substrate/substrate2/compare/geometry-v0.4.0...geometry-v0.5.0) (2023-11-02)


### Features

* **geometry:** implemented contains for polygon ([#292](https://github.com/ucb-substrate/substrate2/issues/292)) ([708053a](https://github.com/ucb-substrate/substrate2/commit/708053adfb9f3783fc03895ede7348ace51730f0))
* **netlists:** use consistent ordering via indexmap ([#266](https://github.com/ucb-substrate/substrate2/issues/266)) ([f275c19](https://github.com/ucb-substrate/substrate2/commit/f275c19396ed4f7d255836822ff72b808f89cde7)), closes [#265](https://github.com/ucb-substrate/substrate2/issues/265)
* **polygon:** polygon implemented in geometry ([#263](https://github.com/ucb-substrate/substrate2/issues/263)) ([4508570](https://github.com/ucb-substrate/substrate2/commit/45085706a30a12f4af6c5e3f642ca55b4c32dd24))


### Bug Fixes

* **deps:** update rust crate num-rational to 0.4 ([#294](https://github.com/ucb-substrate/substrate2/issues/294)) ([fc8f5ce](https://github.com/ucb-substrate/substrate2/commit/fc8f5ce9f35eb074acff45115e44ffbd37e0d237))

## [0.4.0](https://github.com/substrate-labs/substrate2/compare/geometry-v0.3.0...geometry-v0.4.0) (2023-08-08)


### Features

* **macros:** refactor macro reexports ([#250](https://github.com/substrate-labs/substrate2/issues/250)) ([a332717](https://github.com/substrate-labs/substrate2/commit/a332717e549fdea50306067e1c92dc60293aed4c))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * geometry_macros bumped from 0.0.0 to 0.0.1

## [0.3.0](https://github.com/substrate-labs/substrate2/compare/geometry-v0.2.0...geometry-v0.3.0) (2023-07-23)


### Features

* **gds-import:** implement GDS to RawCell importer ([#196](https://github.com/substrate-labs/substrate2/issues/196)) ([fc37eeb](https://github.com/substrate-labs/substrate2/commit/fc37eeb6bac10779491b98bcadcc0eeaeb7d8ec5))
* **gds:** gds reexport test ([#199](https://github.com/substrate-labs/substrate2/issues/199)) ([93d3cd5](https://github.com/substrate-labs/substrate2/commit/93d3cd555c1cb4a76a8845f4401e98d327b5d674))
* **proc-macros:** derive macros for geometry traits ([#164](https://github.com/substrate-labs/substrate2/issues/164)) ([a86074a](https://github.com/substrate-labs/substrate2/commit/a86074a69b714b1be551ae00c775beb04c13f776))
* **tiling:** array and grid tiling API ([#201](https://github.com/substrate-labs/substrate2/issues/201)) ([b3b7c2b](https://github.com/substrate-labs/substrate2/commit/b3b7c2bfb7ba72198872d0f08ded3e0bc757479d))

## [0.2.0](https://github.com/substrate-labs/substrate2/compare/geometry-v0.1.0...geometry-v0.2.0) (2023-07-07)


### Features

* **blackboxing:** support Substrate and SCIR blackboxes ([#135](https://github.com/substrate-labs/substrate2/issues/135)) ([049a598](https://github.com/substrate-labs/substrate2/commit/049a598e2b8d11228c63f03dc878fc4c56e036a6))
* **tests:** add SCIR and SPICE netlister blackbox tests ([049a598](https://github.com/substrate-labs/substrate2/commit/049a598e2b8d11228c63f03dc878fc4c56e036a6))

## 0.1.0 (2023-06-13)


### Features

* **geometry:** initial geometry implementation ([#23](https://github.com/substrate-labs/substrate2/issues/23)) ([0361062](https://github.com/substrate-labs/substrate2/commit/036106213afa965c245acbd41874148f99fabdbb))
