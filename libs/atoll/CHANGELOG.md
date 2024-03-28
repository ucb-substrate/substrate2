# Changelog

* The following workspace dependencies were updated
  * dependencies
    * substrate bumped from 0.7.0 to 0.7.1

* The following workspace dependencies were updated
  * dependencies
    * substrate bumped from 0.7.1 to 0.8.0

* The following workspace dependencies were updated
  * dependencies
    * substrate bumped from 0.8.0 to 0.8.1

## [0.2.0](https://github.com/ucb-substrate/substrate2/compare/atoll-v0.1.3...atoll-v0.2.0) (2024-03-28)


### Features

* **atoll:** additional routing and strapping APIs ([#392](https://github.com/ucb-substrate/substrate2/issues/392)) ([6544675](https://github.com/ucb-substrate/substrate2/commit/6544675fc739ba34e840823c0057fa9cf18221bc))
* **atoll:** Hierarchical ATOLL and configurable via spacing ([#374](https://github.com/ucb-substrate/substrate2/issues/374)) ([542b9a9](https://github.com/ucb-substrate/substrate2/commit/542b9a956d5c993908e33d3e707fc6bdb97d2c84))
* **atoll:** implement first cut ATOLL implementation ([#357](https://github.com/ucb-substrate/substrate2/issues/357)) ([372b927](https://github.com/ucb-substrate/substrate2/commit/372b9275c9d9c5cd58603f5a462a5e4b66b64cf7))
* **atoll:** require edge-centered tracks ([#368](https://github.com/ucb-substrate/substrate2/issues/368)) ([cad8c96](https://github.com/ucb-substrate/substrate2/commit/cad8c96f47409f564e820bdd775e307094ee1f12))
* **atoll:** SKY130 ATOLL plugin NMOS tile generator ([#350](https://github.com/ucb-substrate/substrate2/issues/350)) ([264d028](https://github.com/ucb-substrate/substrate2/commit/264d0286ca1f4f23defdee54a56db016c71697dc))
* **atoll:** strap routing and enable overlapping instances ([#391](https://github.com/ucb-substrate/substrate2/issues/391)) ([9dddae7](https://github.com/ucb-substrate/substrate2/commit/9dddae76681a58b9a00ff490f88be0b335c56847))
* **grid:** add ATOLL LCM routing grid and layer stack definition ([#338](https://github.com/ucb-substrate/substrate2/issues/338)) ([1e1ad90](https://github.com/ucb-substrate/substrate2/commit/1e1ad90d02b50dd0dd15516c306971241bf30b7c))
* **sky130:** Fix ATOLL plugin implementation ([#376](https://github.com/ucb-substrate/substrate2/issues/376)) ([aef1ed1](https://github.com/ucb-substrate/substrate2/commit/aef1ed10e6104d55a5fdf755ae4c26955d647a42))
* **validation:** add function to validate layers in stack alternate track directions ([#340](https://github.com/ucb-substrate/substrate2/issues/340)) ([3533e74](https://github.com/ucb-substrate/substrate2/commit/3533e7433777c0faf03ec2cc1536fba9fd148f00))


### Bug Fixes

* **atoll:** `GreedyRouter` and transformation fixes ([#385](https://github.com/ucb-substrate/substrate2/issues/385)) ([41e6e31](https://github.com/ucb-substrate/substrate2/commit/41e6e31cb1070f7b0ce2a2db61e885a6f53fa7eb))
* **atoll:** fix how ATOLL creates/uses abstracts ([#383](https://github.com/ucb-substrate/substrate2/issues/383)) ([cd44695](https://github.com/ucb-substrate/substrate2/commit/cd44695ff08fc31d6963f8936ad8092a5f9f7cac))
* **atoll:** store via information in blocked grid points ([#393](https://github.com/ucb-substrate/substrate2/issues/393)) ([0ec0877](https://github.com/ucb-substrate/substrate2/commit/0ec0877d5ab321398e9674353983482f8e8a6d9f))
* **atoll:** Use ATOLL virtual layer for abstract bounding box ([#389](https://github.com/ucb-substrate/substrate2/issues/389)) ([d1060af](https://github.com/ucb-substrate/substrate2/commit/d1060af4c116351f0e55adc341f72b12b57b631f))
* **layout:** fix issues in GDS export and ATOLL API ([#341](https://github.com/ucb-substrate/substrate2/issues/341)) ([08930b1](https://github.com/ucb-substrate/substrate2/commit/08930b1b25d018c20758986e206dc8882df782af))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * substrate bumped from 0.8.1 to 0.9.0
    * cache bumped from 0.5.0 to 0.6.0

## 0.1.0 (2023-11-02)


### Features

* add BFS router for ATOLL ([#313](https://github.com/ucb-substrate/substrate2/issues/313)) ([eaf4cc4](https://github.com/ucb-substrate/substrate2/commit/eaf4cc4336d34f256f36a8564725fb313527f959))
* **docs:** add atoll design docs ([#293](https://github.com/ucb-substrate/substrate2/issues/293)) ([996f1bc](https://github.com/ucb-substrate/substrate2/commit/996f1bcd0f071ec845fa60ff45f404cd71d42632))


### Bug Fixes

* **deps:** update rust crate grid to 0.11 ([#311](https://github.com/ucb-substrate/substrate2/issues/311)) ([2b5a093](https://github.com/ucb-substrate/substrate2/commit/2b5a09346c879c66f46c5de7e7bb4c5210757a6a))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * substrate bumped from 0.6.1 to 0.7.0
