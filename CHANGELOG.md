# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

### Bug Fixes

- Multiple alter column cmds shall use "," to separate ([2bc7e6e](2bc7e6e307928155a8b50c17e3be67281283c977) - 2023-01-08 by Tyr Chen)

## [0.2.12] - 2023-01-08

[e2afc2c](e2afc2c882a86dc78928aad1a1226c9f9bd2e862)...[c82e14e](c82e14e3595d92762867747052abc2d23d957d5e)

### Features

- Support array type ([c82e14e](c82e14e3595d92762867747052abc2d23d957d5e) - 2023-01-08 by Tyr Chen)

## [0.2.11] - 2023-01-08

[4b6bba7](4b6bba767aa9339e8a173e05000fe6653c1f0b5a)...[e2afc2c](e2afc2c882a86dc78928aad1a1226c9f9bd2e862)

### Features

- Make typemod work and fix type related issues if using varchar() ([e2afc2c](e2afc2c882a86dc78928aad1a1226c9f9bd2e862) - 2023-01-08 by Tyr Chen)

## [0.2.10] - 2023-01-08

[77043a7](77043a72c3b3d0fe439937525a343ad5eb095bda)...[4b6bba7](4b6bba767aa9339e8a173e05000fe6653c1f0b5a)

### Bug Fixes

- 1. normalize failed due to lack of `create schema`; 2. plan coloring 3. table sequence sql missing ([8dde39d](8dde39dcc58dec0740a175b4383343032f9f3c7e) - 2023-01-07 by Tyr Chen)
- Fix issue on complex check constraint ([08a8bcf](08a8bcfa97248b31532bd68f7517645a081d14fa) - 2023-01-08 by Tyr Chen)

### Features

- Refactor schema diff code and support rls/owner ([8a66e53](8a66e533283fb724380627f0f907a094814cacdc) - 2023-01-07 by Tyr Chen)
- Support alter table add/drop constraints ([bbe1da0](bbe1da0347b2d6be4e5d78832ceafd6d47ccc1c8) - 2023-01-08 by Tyr Chen)
- Support enum migration and fix constraint expr issue ([4b6bba7](4b6bba767aa9339e8a173e05000fe6653c1f0b5a) - 2023-01-08 by Tyr Chen)

## [0.2.9] - 2023-01-07

[31a68c5](31a68c575ee30010b83cbfae362112e98484d34a)...[77043a7](77043a72c3b3d0fe439937525a343ad5eb095bda)

### Features

- Group sql when saving ([77043a7](77043a72c3b3d0fe439937525a343ad5eb095bda) - 2023-01-07 by Tyr Chen)

## [0.2.8] - 2023-01-07

[4658050](4658050defc8020003f74da8ff90bb878d84bd15)...[31a68c5](31a68c575ee30010b83cbfae362112e98484d34a)

### Features

- Add `renovate schema normalize` ([0c00600](0c006000c6b23e02bc8619f8c1ab60b017ed86c5) - 2023-01-07 by Tyr Chen)
- Make sequence diff work ([44c7695](44c769594b6525bb6c3770bb3b61dbaad39f02a5) - 2023-01-07 by Tyr Chen)
- Support table level objects ([2a01a82](2a01a8274dbd23e6d34def39c1aa29aa8bae8d8e) - 2023-01-07 by Tyr Chen)
- Support table sequence and fix loading order by adding prefix ([85bb819](85bb819befd58cad549b347076cbcc7aec8629f3) - 2023-01-07 by Tyr Chen)
- Improve normalize CLI to remove all sql files and retrieve them from server ([31a68c5](31a68c575ee30010b83cbfae362112e98484d34a) - 2023-01-07 by Tyr Chen)

### Refactor

- Replace most of the macros with functions ([fb2a7f2](fb2a7f2fcc0c14120d48940ce455a14a88261854) - 2023-01-07 by Tyr Chen)
- Deprecate SchemaSaver. Use DatabaseSchema directly. ([b8d2564](b8d2564d4423d3b058c8fac3a579485e7814ef7c) - 2023-01-07 by Tyr Chen)

## [0.2.7] - 2023-01-07

[9a36034](9a360344c11fe77eb0c0a07337bc8656b24c5526)...[4658050](4658050defc8020003f74da8ff90bb878d84bd15)

### Features

- Support default constraint ([4658050](4658050defc8020003f74da8ff90bb878d84bd15) - 2023-01-07 by Tyr Chen)

## [0.2.6] - 2023-01-06

[67bb303](67bb303518436845dbac8ab4ad321665e14501d5)...[9a36034](9a360344c11fe77eb0c0a07337bc8656b24c5526)

### Miscellaneous Tasks

- Improve help ([9a36034](9a360344c11fe77eb0c0a07337bc8656b24c5526) - 2023-01-06 by Tyr Chen)

## [0.2.5] - 2023-01-06

[e124a66](e124a6613894449aa9ccb1e7d9d94812925458f8)...[67bb303](67bb303518436845dbac8ab4ad321665e14501d5)

### Bug Fixes

- Add test for schema_diff and fix schema_id issue ([f3a9754](f3a9754399f22b1b402fb4b35e6c6aefabac7585) - 2023-01-06 by Tyr Chen)

### Refactor

- Rename subcommand `pg` to `schema`. ([67bb303](67bb303518436845dbac8ab4ad321665e14501d5) - 2023-01-06 by Tyr Chen)

## [0.2.4] - 2023-01-06

[31b4ffd](31b4ffd386ccdaa32e73bc636c6381ad7223e5e2)...[e124a66](e124a6613894449aa9ccb1e7d9d94812925458f8)

### Refactor

- Deprecate schema_diff macro, use function instead ([e124a66](e124a6613894449aa9ccb1e7d9d94812925458f8) - 2023-01-06 by Tyr Chen)

## [0.2.3] - 2023-01-06

[c81aa65](c81aa65505e7b54bc2a022b9d83a8342e6d4dd22)...[31b4ffd](31b4ffd386ccdaa32e73bc636c6381ad7223e5e2)

### Features

- Add `renovate pg fetch` and `renovate generate completion`. ([31b4ffd](31b4ffd386ccdaa32e73bc636c6381ad7223e5e2) - 2023-01-06 by Tyr Chen)

## [0.2.2] - 2023-01-06

[65d2e13](65d2e132651a89d262895a9dd6da20c5e9b08550)...[c81aa65](c81aa65505e7b54bc2a022b9d83a8342e6d4dd22)

### Features

- Support `renovate pg apply` command ([c81aa65](c81aa65505e7b54bc2a022b9d83a8342e6d4dd22) - 2023-01-06 by Tyr Chen)

## [0.2.1] - 2023-01-06

[257f82f](257f82f497f117946372fb5af432b3ab7e347c86)...[65d2e13](65d2e132651a89d262895a9dd6da20c5e9b08550)

### Bug Fixes

- Improve error message for `renovate pg init` ([65d2e13](65d2e132651a89d262895a9dd6da20c5e9b08550) - 2023-01-06 by Tyr Chen)

## [0.2.0] - 2023-01-06

### Features

- Working on column migration (not finished) ([c7ca57e](c7ca57e95d27e12c7b649a32d057e4af6f0633c9) - 2022-12-04 by Tyr Chen)
- Add init cli for pg ([acbae1a](acbae1a7601f2775a4918a3ec1094cb6c66b0553) - 2023-01-05 by Tyr Chen)
- Support `renovate pg plan` ([257f82f](257f82f497f117946372fb5af432b3ab7e347c86) - 2023-01-06 by Tyr Chen)

### Miscellaneous Tasks

- Improve column migration ([9864470](98644701788d42f65b34088c2cc1406d76188028) - 2023-01-05 by Tyr Chen)
- Add makefile, changelog and update deps ([1bc5e51](1bc5e51b4d89f373962294f248f8a95c3b82c9e5) - 2023-01-05 by Tyr Chen)

<!-- generated by git-cliff -->
