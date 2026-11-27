# pon//kan.

A free, independent, open-source Windows desktop application for managing VALORANT settings profiles.

## Features

- Save, organize and restore settings profiles.
- Import and export profiles, and optionally share portable settings using short codes.
- Manage saved profiles, favorites and the recycle bin.

## Download and use

The 0.3.0 beta candidate targets Windows 11 x64 with WebView2. The installer is unsigned; Windows may show a SmartScreen warning. Obtain the installer only from the official repository's Releases page, and compare its SHA-256 against SHA256SUMS.txt.

Sign in to the Riot Client before saving or applying profiles. Close VALORANT before applying a profile. Back up existing settings. This candidate intentionally blocks installation over detected legacy data; automatic migration is not supported. Do not remove legacy profiles to bypass the installer guard.

## Status and limitations

Experimental prerelease candidate, not a stable release. Clean installation and automated tests were checked; complete native end-to-end validation, recovery, legacy migration, CI and Riot review remain pending. Ponkan uses undocumented Riot Client/player preference integrations. Compatibility and protection from account restrictions are not guaranteed. Distribution is pending applicable review and release approval. No automatic updater is provided.

## Privacy

Full profile exports and recovery backups may contain private account and game settings. Optional short codes use is.gd and are not confidential or encrypted. Do not share real backups or credentials in public issues. See [PRIVACY.md](PRIVACY.md) and [SECURITY.md](SECURITY.md).

## Build from source

Tested toolchain: Node.js 24.22.3, Rust 1.100.0, MSVC C++ build tools and WebView2.

Run `npm ci` and `npm run desktop:build`. Development builds use separate identity and storage via `npm run desktop:dev`.

## License

[MIT](LICENSE). Third-party notices and attributions are included in [LICENSE-THIRD-PARTY.txt](LICENSE-THIRD-PARTY.txt) and [third-party-notices](third-party-notices/). Integration adapted from ValoUtils v1.0.17. Development directed by the project maintainer with AI assistance.

Ponkan isn't endorsed by Riot Games and doesn't reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties. Riot Games, and all associated properties are trademarks or registered trademarks of Riot Games, Inc.
