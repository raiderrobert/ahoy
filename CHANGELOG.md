# Changelog

## [0.1.2](https://github.com/raiderrobert/ahoy/compare/v0.1.1...v0.1.2) (2026-03-15)


### Bug Fixes

* rename install/install.rs to install/run.rs to fix module inception lint ([3a37ac7](https://github.com/raiderrobert/ahoy/commit/3a37ac752eb3582cde7c571e7679133852412f1a))


### Miscellaneous

* added in justfile ([a85073b](https://github.com/raiderrobert/ahoy/commit/a85073b43861f2e15bff6f00afc259d61b8ce2eb))

## [0.1.1](https://github.com/raiderrobert/ahoy/compare/v0.1.0...v0.1.1) (2026-03-15)


### Features

* add Claude Code plugin for hook installation ([c8d1e28](https://github.com/raiderrobert/ahoy/commit/c8d1e2822b751bb538140177ae771a25b5b35197))
* add install and uninstall scripts ([4055e62](https://github.com/raiderrobert/ahoy/commit/4055e6232ac0005684dcaebb13b9469921654205))
* add marketplace.json for plugin discovery ([3d41b56](https://github.com/raiderrobert/ahoy/commit/3d41b562969ef31bb42cabe7e9214eddd87d2416))
* implement ahoy notification daemon for LLM agents ([abeaa08](https://github.com/raiderrobert/ahoy/commit/abeaa08c4154316ff298b728df32c12605170334))
* **macos:** add native Swift notification helper ([34992a6](https://github.com/raiderrobert/ahoy/commit/34992a64c7cc54666cbd433f4fc5723305394812))
* **macos:** native Swift notifications with app icon ([9ae19c2](https://github.com/raiderrobert/ahoy/commit/9ae19c2a1da264c5d0ac264ceb04db1e00186ca8))
* remove retry notification behavior ([b072b61](https://github.com/raiderrobert/ahoy/commit/b072b61b85f57849ead1ab331a8469a079aec272))
* suppress notifications when user is active ([e9989ab](https://github.com/raiderrobert/ahoy/commit/e9989abe2bfa8c5f1bd673bd42a274c314c187b7))


### Bug Fixes

* add email to author/owner fields ([ace76b7](https://github.com/raiderrobert/ahoy/commit/ace76b70ffd64701e05204bb6f4a7aac57230bf7))
* **ci:** use correct dtolnay/rust-toolchain action ([9fce2af](https://github.com/raiderrobert/ahoy/commit/9fce2aff8003390c8fa96027f7bea33d6fe67acb))
* **ci:** use Makefile to build Swift app bundle ([9f0ac9f](https://github.com/raiderrobert/ahoy/commit/9f0ac9f489fa93ba76f4823834fd7ac522767268))
* correct notification suppression logic ([d0fc9f9](https://github.com/raiderrobert/ahoy/commit/d0fc9f97467657745efd247fb446d826531b0dc8))
* **macos:** add sender flag to show notifications as Ahoy ([3b3ade3](https://github.com/raiderrobert/ahoy/commit/3b3ade33af00363b794ff06f42a0395811c43d4c))
* **macos:** use osascript for reliable notifications with sound ([6977ed7](https://github.com/raiderrobert/ahoy/commit/6977ed72138c37800b329f661dca0a527ab0716e))
* **macos:** use terminal-notifier for reliable banner notifications ([c9f67c7](https://github.com/raiderrobert/ahoy/commit/c9f67c7d5c4aedaee103e1cfe9bd4703ebed39c4))
* only suppress notifications when terminal is focused ([c48a184](https://github.com/raiderrobert/ahoy/commit/c48a184f2aaf2c206934c7bc62ba3d88ac629a31))
* register Ahoy.app with Launch Services for reliable notifications ([6f9d161](https://github.com/raiderrobert/ahoy/commit/6f9d161b264db46a239b7aa10a3f69cc1056cfaa))
* remove email from author/owner fields ([835f551](https://github.com/raiderrobert/ahoy/commit/835f5518d2a9d7b549bd0b068672c52c9b5809f1))


### Miscellaneous

* add in graft + pr-tutle + release-please ([d5ac77d](https://github.com/raiderrobert/ahoy/commit/d5ac77d572c1435ab8668f33b342056049cef395))
* add lefthook for shareable git hooks ([9308469](https://github.com/raiderrobert/ahoy/commit/93084699f90b29aec1874c113ad794d1e51f61dd))
* add release-please config files ([be45bf3](https://github.com/raiderrobert/ahoy/commit/be45bf3af56932225e15f738da0d5bde521638e3))
* corrected description ([377a76d](https://github.com/raiderrobert/ahoy/commit/377a76ddabae455beaa80a221bd813ef252df318))
* exclude Ahoy.app build artifact from git ([0772c80](https://github.com/raiderrobert/ahoy/commit/0772c804392374fb36770eb713e460edab234b2b))
* ignore CLAUDE.md and .beads files ([f3ad76f](https://github.com/raiderrobert/ahoy/commit/f3ad76f694895422d12c8d74e0bbf4a2be8501b7))
* pin Rust toolchain to 1.92.0 for consistent formatting ([cf697c4](https://github.com/raiderrobert/ahoy/commit/cf697c46ba59f175ea548c3806fa8489e10176a0))
* removed AGENTS.md ([be26fa8](https://github.com/raiderrobert/ahoy/commit/be26fa8024ebe182586652df9ca7eee92e8bfad2))


### Documentation

* add brief README ([89139a9](https://github.com/raiderrobert/ahoy/commit/89139a94e74b5e5bfda31bfca7e608a84f649bb2))
* add brief README ([81c6ad4](https://github.com/raiderrobert/ahoy/commit/81c6ad48094f2d6583eb694c52337c311a9b8297))
* fix plugin install syntax ([a787c60](https://github.com/raiderrobert/ahoy/commit/a787c60b68579a9af26220e2b859ea8eeb74000c))
* recommend plugin install over CLI hooks ([8a9f1f4](https://github.com/raiderrobert/ahoy/commit/8a9f1f4445cfed1cd62df1343a193f4aa4686775))
* rewrite README to match actual implementation ([f6f07e1](https://github.com/raiderrobert/ahoy/commit/f6f07e101739ca3e7e1d3c684a2127a405318b78))
* rewrite README with clearer tagline and trimmed sections ([9f1a692](https://github.com/raiderrobert/ahoy/commit/9f1a692aed53f3056eb9f6b2491bbc895e59679e))


### Code Refactoring

* consolidate Info.plist files to single source of truth ([fa1bd9e](https://github.com/raiderrobert/ahoy/commit/fa1bd9e37ad5615fef2f783e3e6d297ff92cb6b5))
* remove unnecessary tokio and thiserror dependencies ([445023b](https://github.com/raiderrobert/ahoy/commit/445023bf1672fcce4ac37e67deb5c7a9b4af925e))
* removed obvious comments ([06273a3](https://github.com/raiderrobert/ahoy/commit/06273a36957b13570496894e72ab5fcd78e70811))
* simplify ahoy from daemon to direct CLI ([027d948](https://github.com/raiderrobert/ahoy/commit/027d94875928ef28ca4a7ceae7617502156f9650))
