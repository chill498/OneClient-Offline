# OneClient Offline

Unofficial GPL-3.0 community fork of [Polyfrost/OneLauncher](https://github.com/Polyfrost/OneLauncher), maintained by [chill498](https://github.com/chill498). This fork is independent of Polyfrost.

Username-only accounts are **offline and unauthenticated**. They can be used for singleplayer and servers that permit offline identities. Microsoft authentication and ownership requirements for authenticated services remain in place.

The fork includes optional local PNG skins with Normal/Slim models, the reproduced setup/avatar freeze fix, persistent selection shared by Home/Versions/Launch, seven embedded version backgrounds, and a Minecraft fullscreen cursor compatibility option. Local in-game skins require a compatible mod; cross-launcher visibility requires server-side skin support.

See [offline-mode details and validation](OFFLINE_MODE.md), [privacy/network disclosure](PRIVACY.md), and [code signing policy](CODE_SIGNING.md).

## Build

Windows builds require Rust 1.99.1, Visual Studio C++ build tools and a Windows SDK. The manual `Offline Windows build` workflow runs focused tests and builds unsigned review artifacts from this repository's source. It never publishes a release or claims signing approval.

```text
cargo --config profile.release.package.oneclient_app.codegen-units=16 rustc -p oneclient_app --bin oneclient_app --release --locked --target x86_64-pc-windows-msvc -- -C lto=thin
```

The source is based on upstream commit `290578ef1d4182e50d015c3f284a1c31b17f46d5`. Every file in the supplied base ZIP was checked against upstream Git blob hashes. [Provenance](distribution/upstream-provenance.json) records the source and patch hashes. Original upstream workflows are retained under `distribution/upstream-workflows` for reference, without being activated for this fork.

## Releases and code signing policy

The current revision 4 setup is unsigned. A free SignPath Foundation application was submitted on 2 November 2026 and is pending review. **No signing approval or certificate has been issued, and this fork is not signed or endorsed by SignPath or the upstream publisher.** Windows protection should remain enabled. This repository does not offer a signed release yet.

After project approval, signed executables and installers must come from reviewed source and verifiable builds using this fork's own approved signing project. No upstream signing identity or token may be reused. See [CODE_SIGNING.md](CODE_SIGNING.md).

## License and artwork

The code is [GPL-3.0-only](LICENSE). Preserve [upstream attribution](ATTRIBUTION.md). New version artwork was generated using the built-in image generation tool; [prompts and asset paths](distribution/art-prompts.json) are included.
