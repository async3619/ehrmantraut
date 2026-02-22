# [1.0.0-dev.3](https://github.com/async3619/ehrmantraut/compare/v1.0.0-dev.2...v1.0.0-dev.3) (2026-02-22)


### Features

* **napi:** add multi-threaded parallel processing APIs ([#63](https://github.com/async3619/ehrmantraut/issues/63)) ([019dcb9](https://github.com/async3619/ehrmantraut/commit/019dcb9e3eb24d9c8711ace3164c752fd7cc2859))

# [1.0.0-dev.2](https://github.com/async3619/ehrmantraut/compare/v1.0.0-dev.1...v1.0.0-dev.2) (2026-02-22)


### Bug Fixes

* **lowering:** add defensive bounds checks for string and byte slicing ([#51](https://github.com/async3619/ehrmantraut/issues/51)) ([940b4ae](https://github.com/async3619/ehrmantraut/commit/940b4ae4497371649ded64b35dcfb79798602696))
* **lowering:** harden for..of detection against grammar changes ([#55](https://github.com/async3619/ehrmantraut/issues/55)) ([3f4d8d1](https://github.com/async3619/ehrmantraut/commit/3f4d8d16640e1afa92d5d7d62a5bd00908ffc2f0))
* **lowering:** remove incorrect DeclKind::Import fallback for exported variables ([#50](https://github.com/async3619/ehrmantraut/issues/50)) ([4f19787](https://github.com/async3619/ehrmantraut/commit/4f197877fb075cce48bb2621f787ba22899be4a5))
* **lowering:** replace silent fallbacks with opaque variants for unknown patterns and object properties ([#53](https://github.com/async3619/ehrmantraut/issues/53)) ([1a2b264](https://github.com/async3619/ehrmantraut/commit/1a2b264d60d467328bce7b4ee984c12aaac18694))


### Features

* **lowering:** add await/yield and template literal lowering ([#41](https://github.com/async3619/ehrmantraut/issues/41)) ([163c8f6](https://github.com/async3619/ehrmantraut/commit/163c8f6b6b0b075529c3a364f223918f344b8659)), closes [#31](https://github.com/async3619/ehrmantraut/issues/31) [#32](https://github.com/async3619/ehrmantraut/issues/32)
* **lowering:** add destructuring pattern and spread expression lowering ([#39](https://github.com/async3619/ehrmantraut/issues/39)) ([3e1b2ab](https://github.com/async3619/ehrmantraut/commit/3e1b2ab3e334eca27026834b199987f78d3dd55f)), closes [#27](https://github.com/async3619/ehrmantraut/issues/27) [#23](https://github.com/async3619/ehrmantraut/issues/23)
* **lowering:** add labeled statement, class body member, and advanced parameter lowering ([#42](https://github.com/async3619/ehrmantraut/issues/42)) ([4f68df6](https://github.com/async3619/ehrmantraut/commit/4f68df6fcb472318d0ecb501f2774b0f464b356a))
* **lowering:** add new, this/super, and optional chaining expression lowering ([#40](https://github.com/async3619/ehrmantraut/issues/40)) ([f0cc50c](https://github.com/async3619/ehrmantraut/commit/f0cc50c366ec662263b574f2769be2ab47b7b295)), closes [#29](https://github.com/async3619/ehrmantraut/issues/29) [#28](https://github.com/async3619/ehrmantraut/issues/28) [#30](https://github.com/async3619/ehrmantraut/issues/30)
* **lowering:** add throw, arrow function, and import/export lowering ([#38](https://github.com/async3619/ehrmantraut/issues/38)) ([1c6817a](https://github.com/async3619/ehrmantraut/commit/1c6817ad51d1671d5453b8bcfb0b6d5688254e5b)), closes [#24](https://github.com/async3619/ehrmantraut/issues/24) [#22](https://github.com/async3619/ehrmantraut/issues/22) [#21](https://github.com/async3619/ehrmantraut/issues/21)
* **lowering:** add unary, update, conditional, array, and object expression lowering ([#37](https://github.com/async3619/ehrmantraut/issues/37)) ([35f8774](https://github.com/async3619/ehrmantraut/commit/35f877449d2ccbf9d585db256ac58308bb7e1b0a))
* **lowering:** implement tagged template literal lowering ([#52](https://github.com/async3619/ehrmantraut/issues/52)) ([0604814](https://github.com/async3619/ehrmantraut/commit/060481410d747c440538fe74fb610774a8593706))

# 1.0.0-dev.1 (2026-02-21)


### Features

* implement tree-sitter CST parsing and IR lowering for JS/TS ([#11](https://github.com/async3619/ehrmantraut/issues/11)) ([9ce9054](https://github.com/async3619/ehrmantraut/commit/9ce9054e9418b5784fabb94071accae3f1a60240))
