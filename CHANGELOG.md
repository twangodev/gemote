# Changelog

## [0.4.0](https://github.com/twangodev/gemote/compare/gemote-v0.3.1...gemote-v0.4.0) (2026-04-17)


### Features

* implement recursive flag handling for sync and save commands ([2f0f7c9](https://github.com/twangodev/gemote/commit/2f0f7c9b79f304c9b4be5cd82daf4ef082b00b08))

## [0.3.1](https://github.com/twangodev/gemote/compare/gemote-v0.3.0...gemote-v0.3.1) (2026-02-08)


### Miscellaneous Chores

* bump gemote version to 0.3.1 ([c3bd4ac](https://github.com/twangodev/gemote/commit/c3bd4ac786dd3e865dcdcc10fdddc1998b6ee642))

## [0.3.0](https://github.com/twangodev/gemote/compare/gemote-v0.2.0...gemote-v0.3.0) (2026-02-06)


### Features

* add shell completion support for gemote CLI ([6af86b1](https://github.com/twangodev/gemote/commit/6af86b1b501c4d45cb3d0b6fda7100405e42d10e))
* add version information to gemote command in CLI ([56007b5](https://github.com/twangodev/gemote/commit/56007b5e4b412799e112ece89852a4cde2876e6b))
* improve config file serialization with additional metadata ([2352d94](https://github.com/twangodev/gemote/commit/2352d948cc1876c69dec2e4da96f6923bacf1b49))

## [0.2.0](https://github.com/twangodev/gemote/compare/gemote-v0.1.0...gemote-v0.2.0) (2026-02-06)


### Features

* add path-slash dependency and update path handling in git module ([420acb3](https://github.com/twangodev/gemote/commit/420acb3763d297eab20c7fac7c2dc1ff21738072))
* add recursive option for sync and save commands, support submodules ([bbc953f](https://github.com/twangodev/gemote/commit/bbc953f1789ded91d46b02fc43b34bea61f22ba9))
* add setup job and targets configuration for CI workflow ([bf31fe7](https://github.com/twangodev/gemote/commit/bf31fe7e2bf37b9303b90af1bec62da29ea526e8))
* add tests to support recursive submodules ([df0ddeb](https://github.com/twangodev/gemote/commit/df0ddebec70fc27639195584e1a3cd0f7edf1fd6))
* bump gemote version to 0.2.0 in Cargo files ([a007307](https://github.com/twangodev/gemote/commit/a00730724251a7a8c525e001fc0b78b4ed736450))
* enhance CI workflow with artifact upload and release steps ([659a675](https://github.com/twangodev/gemote/commit/659a675ea1bac522c694b41bc25ed4310f064f79))
* replace overwrite flag with force in save command ([8af52c6](https://github.com/twangodev/gemote/commit/8af52c63a5b0eecd3639d1c07055b0f4d88ee279))
* update README with installation instructions and recursive submodule support ([ade48fa](https://github.com/twangodev/gemote/commit/ade48faadcd2294e48f363916a617593bacb8eaa))


### Bug Fixes

* normalize path separators in submodule handling ([6787092](https://github.com/twangodev/gemote/commit/6787092a98b4a843966b8bcbbf1343ec200503af))
* update CI workflow to ensure Docker job depends on build step ([3a1b218](https://github.com/twangodev/gemote/commit/3a1b2187eb4885c2442ac2ec5fd4052394f0be91))

## 0.1.0 (2026-02-06)


### Features

* add authentication step for crates.io in CI workflow ([f9d76e1](https://github.com/twangodev/gemote/commit/f9d76e1d8011490492fac30dc22c656168ed6364))
* add Dockerfile and .dockerignore for containerization ([c49580f](https://github.com/twangodev/gemote/commit/c49580f2b9ecfd5cfd372a89d6e67456719fe5dd))
* add initial configuration for gemote with default settings ([259f031](https://github.com/twangodev/gemote/commit/259f031b41560beb19afa49c626d79daf579aa02))
* add TOML mode settings to configuration serialization ([df77fdc](https://github.com/twangodev/gemote/commit/df77fdce12f1f6dbfe88b93d6e50d55ab8a2032d))
* implement gemote CLI for managing git remotes ([d40a9be](https://github.com/twangodev/gemote/commit/d40a9be02835280c922507787b31959fa95f1e9f))
* implement unit and integration tests ([cc5e1bf](https://github.com/twangodev/gemote/commit/cc5e1bf8ab82f1d5078001c952febbc1009ad165))
* init project ([eb54be2](https://github.com/twangodev/gemote/commit/eb54be29b71b45317b2561924e498b70301e620e))
* update Cargo.toml with project metadata and add release configuration ([a6bd293](https://github.com/twangodev/gemote/commit/a6bd293dfa838daebcde9f23fc22680dafe561ba))
