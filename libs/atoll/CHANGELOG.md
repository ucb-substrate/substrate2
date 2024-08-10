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

## [0.2.0](https://github.com/ucb-substrate/substrate2/compare/atoll-v0.1.3...atoll-v0.2.0) (2024-08-10)


### Features

* **atoll:** add functions for creating named instances ([#446](https://github.com/ucb-substrate/substrate2/issues/446)) ([081f8f5](https://github.com/ucb-substrate/substrate2/commit/081f8f55137f3b2834368e7225955d500d6841b5))
* **atoll:** additional routing and strapping APIs ([#392](https://github.com/ucb-substrate/substrate2/issues/392)) ([6544675](https://github.com/ucb-substrate/substrate2/commit/6544675fc739ba34e840823c0057fa9cf18221bc))
* **atoll:** assign nets only if available ([#416](https://github.com/ucb-substrate/substrate2/issues/416)) ([3c38c84](https://github.com/ucb-substrate/substrate2/commit/3c38c841ce7e3a2728a8a56013f194d9f60b91bb))
* **atoll:** Hierarchical ATOLL and configurable via spacing ([#374](https://github.com/ucb-substrate/substrate2/issues/374)) ([542b9a9](https://github.com/ucb-substrate/substrate2/commit/542b9a956d5c993908e33d3e707fc6bdb97d2c84))
* **atoll:** implement first cut ATOLL implementation ([#357](https://github.com/ucb-substrate/substrate2/issues/357)) ([372b927](https://github.com/ucb-substrate/substrate2/commit/372b9275c9d9c5cd58603f5a462a5e4b66b64cf7))
* **atoll:** require edge-centered tracks ([#368](https://github.com/ucb-substrate/substrate2/issues/368)) ([cad8c96](https://github.com/ucb-substrate/substrate2/commit/cad8c96f47409f564e820bdd775e307094ee1f12))
* **atoll:** SKY130 ATOLL plugin NMOS tile generator ([#350](https://github.com/ucb-substrate/substrate2/issues/350)) ([264d028](https://github.com/ucb-substrate/substrate2/commit/264d0286ca1f4f23defdee54a56db016c71697dc))
* **atoll:** strap routing and enable overlapping instances ([#391](https://github.com/ucb-substrate/substrate2/issues/391)) ([9dddae7](https://github.com/ucb-substrate/substrate2/commit/9dddae76681a58b9a00ff490f88be0b335c56847))
* **grid:** add ATOLL LCM routing grid and layer stack definition ([#338](https://github.com/ucb-substrate/substrate2/issues/338)) ([1e1ad90](https://github.com/ucb-substrate/substrate2/commit/1e1ad90d02b50dd0dd15516c306971241bf30b7c))
* **sky130:** Fix ATOLL plugin implementation ([#376](https://github.com/ucb-substrate/substrate2/issues/376)) ([aef1ed1](https://github.com/ucb-substrate/substrate2/commit/aef1ed10e6104d55a5fdf755ae4c26955d647a42))
* **straps:** cut power straps to vias ([#430](https://github.com/ucb-substrate/substrate2/issues/430)) ([0ff8636](https://github.com/ucb-substrate/substrate2/commit/0ff863607e53dea7f057f973179750cbe689752e))
* **validation:** add function to validate layers in stack alternate track directions ([#340](https://github.com/ucb-substrate/substrate2/issues/340)) ([3533e74](https://github.com/ucb-substrate/substrate2/commit/3533e7433777c0faf03ec2cc1536fba9fd148f00))


### Bug Fixes

* **atoll:** `GreedyRouter` and transformation fixes ([#385](https://github.com/ucb-substrate/substrate2/issues/385)) ([41e6e31](https://github.com/ucb-substrate/substrate2/commit/41e6e31cb1070f7b0ce2a2db61e885a6f53fa7eb))
* **atoll:** abstract/autorouter fixes and APIs ([#398](https://github.com/ucb-substrate/substrate2/issues/398)) ([4dfac76](https://github.com/ucb-substrate/substrate2/commit/4dfac76647347ca8fc0131adb7ec5b066a1685de))
* **atoll:** allow tiles with top layer below 0 ([#417](https://github.com/ucb-substrate/substrate2/issues/417)) ([7ea4a43](https://github.com/ucb-substrate/substrate2/commit/7ea4a439b4457ce6c3eae5a197c5b89277bf13f4))
* **atoll:** fix how ATOLL creates/uses abstracts ([#383](https://github.com/ucb-substrate/substrate2/issues/383)) ([cd44695](https://github.com/ucb-substrate/substrate2/commit/cd44695ff08fc31d6963f8936ad8092a5f9f7cac))
* **atoll:** make router work regardless of shuffling ([#403](https://github.com/ucb-substrate/substrate2/issues/403)) ([303bec2](https://github.com/ucb-substrate/substrate2/commit/303bec2a541b236e0c1ebd4d6eb4c642d68d5574))
* **atoll:** store via information in blocked grid points ([#393](https://github.com/ucb-substrate/substrate2/issues/393)) ([0ec0877](https://github.com/ucb-substrate/substrate2/commit/0ec0877d5ab321398e9674353983482f8e8a6d9f))
* **atoll:** Use ATOLL virtual layer for abstract bounding box ([#389](https://github.com/ucb-substrate/substrate2/issues/389)) ([d1060af](https://github.com/ucb-substrate/substrate2/commit/d1060af4c116351f0e55adc341f72b12b57b631f))
* **atoll:** use checked operations in abstract generator ([#412](https://github.com/ucb-substrate/substrate2/issues/412)) ([2dad96a](https://github.com/ucb-substrate/substrate2/commit/2dad96a3fb3b0af8c09c0a2b279cf25e565056cf))
* **layout:** fix issues in GDS export and ATOLL API ([#341](https://github.com/ucb-substrate/substrate2/issues/341)) ([08930b1](https://github.com/ucb-substrate/substrate2/commit/08930b1b25d018c20758986e206dc8882df782af))
* **schematics:** fix bugs with instance naming, cell ID allocation ([#445](https://github.com/ucb-substrate/substrate2/issues/445)) ([e7da085](https://github.com/ucb-substrate/substrate2/commit/e7da08583fefe96625017d32c03fc3cdd39aa9b4))
* **straps:** mark vias on strap ends to prevent drc issues ([#415](https://github.com/ucb-substrate/substrate2/issues/415)) ([f85e4db](https://github.com/ucb-substrate/substrate2/commit/f85e4dbb2102cbeda58c48dd5393a8367dd27c9e))


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
