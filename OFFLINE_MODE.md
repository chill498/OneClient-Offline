# Username-only offline accounts â€” OneClient 2.9.0

This is an unofficial source modification based on the supplied `OneLauncher-main.zip`, whose workspace version is 2.9.0. The source ZIP includes source and regression tests; the compiled Windows client and new offline installer are supplied separately. The supplied ZIP has no Git commit metadata, so this is not a claim that it exactly reproduces the signed 2.9.0 installer.

The companion `OneClient-Offline-Windows-x64.zip` contains the built Windows client. Extract the entire ZIP and open **Start OneClient.exe**. That launcher keeps its profile in a `data` folder beside the program and needs no Rust or PowerShell for daily use. Personal profile data is excluded from the redistributable ZIP. The portable launcher's corresponding source is in `distribution/windows/portable-launcher.cpp` and `portable-launcher.rc`.

## Using the modified application after it is built

During first-time setup, enter a username, optionally choose a skin PNG and its **Normal** or **Slim** model, then press **Next**. Next saves the account/skin and advances in one operation. Pressing Enter in the username field does the same. Invalid names and save errors stay visible on this page. Microsoft sign-in is also available. Existing installations can add usernames through **Settings â†’ Accounts â†’ Add offline**.

Usernames must contain 3â€“16 ASCII letters, digits or underscores. Duplicate names are rejected without regard to letter case. The first offline account becomes the default when no default is already selected. Adding another offline account keeps the existing selection.

Offline accounts are explicitly labelled **Offline (unauthenticated)**. They are usable for singleplayer and servers that allow offline players. Online-mode servers and Realms still require a valid Microsoft/Minecraft session. No server settings, session services or authentication checks are bypassed. Creating an offline account does not verify ownership of a Minecraft identity.

Offline account mode does not provide a fully disconnected installation: Java, game files, assets and mods may still require downloads.

## Windows installer

`OneClient-Offline-2.9.0.3-x64-setup.exe` installs for the current Windows user under `%LOCALAPPDATA%\Programs\OneClient Offline`, creates desktop/Start-menu shortcuts, and provides an uninstaller. It does not request administrator privileges, add startup tasks/services, or alter antivirus settings. Program files and the uninstall entry are separate from the official OneClient installation. Accounts, skins and game files in the installed `data` folder are retained on upgrade and uninstall. Personal profile data is excluded from the installer and ZIP.

This remains an unsigned community build. SmartScreen/Smart App Control can warn or block it. Trusted signing and publisher/file reputation are required for wider distribution; no warning-free guarantee is possible. There is no code-signing certificate available in this workspace. A clean antivirus scan is not a security guarantee. The installer script is `distribution/windows/offline-installer.nsi` and uses unmodified NSIS 3.12 tooling verified against the MacPorts SHA-256.

The managed `oneclient-offline-build.txt` file beside the application prevents the official launcher updater from replacing this custom build. Use this build's installer for updates. Minecraft/mod downloads still use the normal launcher paths.

For isolated installer tests, `/S /PORTABLE /D=<absolute workspace folder>` installs the identical payload without creating shortcuts or registry entries; this mode is recorded in `installer-portable.txt` so its uninstaller also avoids changing shell integration.

## Local skins

At the username step, **Choose skin from folder** opens a PNG file picker. Choose **Normal** or **Slim** and check the 3D preview. You can continue without a skin. Later, open **Settings â†’ Accounts â†’ Profile** (or the pencil beside an offline account) to choose a different skin, change the arm model, or choose **Use default skin**, then **Save skin**. The same controls are available in the Add offline dialog.

This accepts decoded 64 Ã— 64 PNGs up to 1 MiB. Invalid, damaged, oversized and other-size images are rejected without replacing the existing skin. Each account's image and model are saved atomically under `offline-skins/<offline UUID>.json` in the launcher's profile. The selected image is copied, so moving the original does not lose it. This does not alter the authentication file or upload a skin to Microsoft.

Launcher previews and avatars use the local image immediately after saving. In-game use **requires a compatible CustomSkinLoader mod installed in that Minecraft instance**. The launcher does not install that mod automatically. Before launch it exports a username-scoped `CustomSkinAPI` profile under `<game folder>/CustomSkinLoader/OneClientOffline/`, and supplies a managed `ExtraList/OneClientOffline.json` provider. The explicit `default` or `slim` texture key carries the arm model. Other loader providers/settings are preserved. Removing the selected skin removes that username's managed game profile at the next launch. Relaunch Minecraft after changes. Local skins are visible only on this computer; other players do not receive them. Vanilla alone does not apply this local cosmetic.

The integration follows [CustomSkinLoader's CustomSkinAPI parser](https://github.com/xfl03/MCCustomSkinLoader/blob/15-develop/Common/src/main/java/customskinloader/loader/jsonapi/CustomSkinAPI.java) and [ExtraList loader](https://github.com/xfl03/MCCustomSkinLoader/blob/15-develop/Common/src/main/java/customskinloader/config/Config.java). It uses local file paths, without session replacement or authentication claims. Microsoft accounts continue to use the existing authenticated wardrobe. The in-game result has not been exercised.

## Source changes

- `packages/oneclient_auth/src/store.rs`: allow adding and resolving offline accounts without a Microsoft account, and select the first offline account by default. Keep validation, duplicate checking and persisted account kinds.
- `packages/oneclient_auth/src/service.rs`: allow launching an offline account without a separate Microsoft account. Offline accounts continue to skip Microsoft token renewal.
- `packages/oneclient_auth/src/error.rs`: remove the obsolete Microsoft prerequisite error.
- `packages/oneclient_app/src/view/onboarding/account.rs`: add a username field and offline-account action to setup, with validation, pending state and error display.
- `packages/oneclient_app/src/view/app/settings/accounts.rs`: enable offline creation independently of Microsoft sign-in and explain its limits.
- Account switcher, control center, account-query exports and English translations: remove obsolete prerequisite wording and label offline identities as unauthenticated.
- `packages/oneclient_core/src/game/launch.rs` and `packages/oneclient_mc/src/arguments.rs`: pass a non-authenticating `0` token placeholder and vanilla's `legacy` user type for offline accounts, while Microsoft accounts retain their token and `msa` user type. Both modern argument arrays and legacy argument strings receive the selected user type.
- `packages/oneclient_core/src/local_skin.rs`, app `components/offline_skin.rs` and skin/profile queries: local PNG validation, per-account copying/model persistence, file picker, preview, profile editing and CustomSkinLoader export.
- `packages/oneclient_app/src/ipc.rs`: separate Windows instance endpoints for explicit portable profiles, so opening the modified portable client does not forward to a running installed client.
- `packages/oneclient_app/src/main.rs`: support an optional `ONECLIENT_HOME` folder for portable use and isolated launch checks. With an explicit folder, startup skips legacy-folder adoption and URL-handler registration, keeping normal profile files and associations intact. With no override, the normal profile behavior is unchanged.

The existing offline UUID algorithm, `AccountKind::Offline` representation, empty stored credentials, Microsoft/Xbox/Minecraft handshake and ownership checks, and Poly+ restrictions are unchanged. There is no new dependency or database schema change. The existing behavior for signed-out Microsoft accounts is unchanged; it does not grant online authentication.

## Installer/setup revision validation

The username step now saves and advances through **Next** (or Enter) rather than requiring a separate account-creation button. A clean isolated profile was opened with no accounts; user interaction advanced beyond this page and completed setup. The resulting default account was offline, with empty access/refresh tokens. The skin file-picker interaction was not automated while user input was active. In-game appearance is still untested.

The final NSIS installer was executed in `/S /PORTABLE` mode into a new workspace folder. All 18 top-level payload files matched the built bundle byte for byte; no personal data was included. Reinstall and uninstall preserved test account, skin, settings and game-data files. Portable test mode left the product registry keys unchanged. The ordinary desktop/Start-menu registration branch was compiled and reviewed, but was not run against the real user profile. The embedded installer manifest was checked for `asInvoker` (no administrator-elevation request). A separate local Defender scan result is supplied with the installer.

## Validation status

Completed here:

- Parsed all 16 Cargo manifests and checked their local dependency paths and required embedded Java/icon assets.
- Checked that all 2,135 English translation keys exactly match source references, using the same matching rule as the project's translation test.
- Compared the result against the supplied ZIP and confirmed that Microsoft authentication/entitlement implementation, account kinds, offline UUID algorithm, Poly+ authentication, dependency lockfile and build configuration are unchanged.
- Checked patch application against a fresh copy of the supplied ZIP and compared the patched files with the delivered source archive.
- Parsed and formatted all 31 modified or added Rust files with rustfmt 1.99.1, without changing their child modules.
- Ran `cargo test -p oneclient_auth --locked --target x86_64-pc-windows-msvc`: 20 tests passed, 0 failed.
- Ran `cargo test -p oneclient_mc --locked --target x86_64-pc-windows-msvc`: 61 tests passed, 0 failed.
- Ran `cargo test -p oneclient_core --lib local_skin::tests --locked --target x86_64-pc-windows-msvc`: 3 tests passed, 0 failed. These cover PNG size/decoding, copied-image persistence, arm-model persistence, invalid-image preservation, account separation, Microsoft exclusion, username-scoped CustomSkinLoader export, model keys and reset/cleanup.
- Executed the exact Windows `endpoint_for_profile` production function and its included unit test in a small native Rust test harness: 1 passed. The full GUI crate test suite was not run.
- Verified the task-local Rust, Microsoft C++ compiler and Windows SDK by compiling and running small C and Rust programs.

The Windows x64 release build uses the locked dependencies. This skin update uses build-command-only overrides of 16 code-generation units for `oneclient_app` and `-C lto=thin` for the final executable to reduce compilation time, retaining release optimization level 3. Command: `cargo --config profile.release.package.oneclient_app.codegen-units=16 rustc -p oneclient_app --bin oneclient_app --release --locked --target x86_64-pc-windows-msvc -- -C lto=thin`. Cargo manifests and the lockfile are unchanged. The client window opened on the desktop and its account screen showed an offline account labelled **Offline (unauthenticated)**. The original OneClient `settings.json` and `auth.json` hashes were unchanged after the isolated launch check. The executable's x64 architecture, direct DLL dependencies and packaged ZIP were checked. Required app-local Microsoft runtime DLLs are included.

Full Minecraft gameplay, Microsoft login, Realms, server connections and Clippy have not been exercised. Installer extraction, reinstall and uninstall were exercised as described above. The original ZIP and installer were left unchanged. No authenticated identity was created or claimed for an offline username. This is an unsigned custom build.

The build uses task-local Rust 1.99.1, MSVC 14.44.35207 and Windows SDK 10.0.26100.0. The normal installation directories were unavailable, so no global Rust PATH change was made. The workspace-wide formatting check reports pre-existing formatting differences in untouched upstream files; the modified Rust files pass their formatting check.

Regression tests were added or updated for:

- First offline account creation without Microsoft, default selection and empty credentials.
- Retaining an existing default, and retaining an offline default after removing the last Microsoft account.
- Invalid and duplicate usernames, and saving/reloading an offline account without tokens.
- Launching and refreshing an offline identity without Microsoft credentials, even after its stored expiry passes.
- Separate offline/Microsoft launch arguments in both modern and legacy argument formats.

## Checks for a build machine

For future rebuilds, use a machine that has the project's prerequisites, including Rust 1.97+ and, for Windows, the MSVC C++ tools and Windows SDK:

```text
cargo fmt --all --check
cargo test -p oneclient_auth
cargo test -p oneclient_mc
cargo check -p oneclient_app
cargo clippy -p oneclient_auth -p oneclient_mc -p oneclient_app --all-targets
cargo build -p oneclient_app --release
```

Check setup and account management at the minimum supported window size. Test adding a username on a clean app profile, restarting, switching accounts, removing Microsoft accounts, and launching both modern and legacy Minecraft versions. Confirm that offline identities remain visibly unauthenticated, online-mode servers/Realms reject them, valid Microsoft sign-in still works, and Microsoft account failures do not turn into authenticated offline sessions. Use the upstream packaging instructions only after those checks pass.

The supplied installer has not been patched. A new offline installer and rebuilt portable ZIP are supplied; the original signed installer remains unchanged.

Original source ZIP SHA-256: `f8ddff62e01cc4d6bccaf7cbf5468daddef23febf346b7c21612a97eb6acf9b9`.

Original installer SHA-256: `7c6f2c6f152141640380fa80bf8eb1968dd5eecc1ea1179547eeea6566b7ca6a`.


## Revision 3: setup stalls and skin support

Finish no longer awaits broad query invalidation or remote artwork. It writes the setup settings locally before routing to Home; write failures remain visible with retry. Database/settings saves have a 15-second budget, optional catalog sync a 30-second budget, and cached metadata/art requests a 12-second budget covering both headers and body. A stalled or broken response falls back to the existing cache without replacing it with partial content or prematurely committing its ETag. Startup no longer waits for the optional catalog before entering onboarding. If the package query is unavailable, the final page explicitly offers Finish with defaults; players can select mods later.

Profile now includes Set up in-game skins. Choose an instance to open the official CustomSkinLoader Modrinth project in the built-in mod browser, then install a release compatible with that Minecraft version and loader, together with its required dependencies. The browser retains its normal compatibility and install checks. No mod, loader, or third-party skin service is silently installed or selected by the launcher. Vanilla without a compatible mod cannot display a chosen local skin. Installation/download needs internet; already installed game files, Java, mod and exported skin can be used in singleplayer without it.

For skins visible to other players on different launchers, the multiplayer server must apply the texture. Profile offers Export skin PNG for server and a link to the official SkinsRestorer upload guide. Export copies the selected PNG using a Save dialog; the suggested filename identifies Normal or Slim. The player or server administrator must upload/apply it using the server's supported skin workflow, choose Classic for Normal or Slim, and have permission to use it. The launcher does not upload files, hold server credentials, install a server plugin, generate a trusted identity, or automatically change arbitrary servers. Without server-side support, local skins are visible only to the local client with the required mod.

Primary documentation: https://skinsrestorer.net/docs/features/skin-upload and https://github.com/xfl03/MCCustomSkinLoader/wiki/How-to-Use .

Revision 3 transport regression tests: `cargo test -p oneclient_net --lib --locked --offline --target x86_64-pc-windows-msvc`: all 16 passed, including six new tests using a local TCP server for stalled headers/body, no-cache first run, truncated responses, 304/503 fallback, successful refresh and committed/deferred ETags. No external service was needed for these tests. Previously recorded auth/MC/skin test results above remain from the earlier revision; their implementations are unchanged in this revision.


### Reproduced selected-skin freeze

The isolated GUI test reproduced a persistent freeze at the final setup page after choosing a local PNG, despite setup being saved locally. Native thread samples located repeated PNG decoding and avatar rendering on the UI thread. The local-skin hook had cloned the saved PNG Vec into a new Bytes allocation on every render, while Avatar keyed its reactive cache by allocation address. Every cache write scheduled another render with another allocation, preventing Home from settling and starving UI input. This also explains why merely waiting did not help.

The corrected hook memoizes the local skin Bytes until the query changes. Avatar now uses a nonreactive image cache keyed by byte content, caches decode failures, and never writes reactive state from its render cache. Three regression tests execute the exact production cache implementation in a small native Rust harness: 100 identical images from separate allocations decode once, changed images refresh, and invalid images do not cause repeated decoding. All three passed. Full GUI crate tests were not run. The runtime retest result is supplied in the external validation report.


Revision 3 runtime retest: the final build was opened with a new isolated profile containing no accounts or database. The native chooser selected a 64x64 PNG; Next saved the offline account and PNG. With the metadata endpoint deliberately stalled, Finish opened Home in the first post-click UI snapshot, displayed the avatar, and remained responsive. The user then navigated to Versions and started 26.2; the launcher began downloading game files. The saved PNG matched byte-for-byte, setup completion persisted, and the sole account was offline with empty credentials. The saved model in this UI test was Normal. In-game and cross-client appearance validation remains pending; later runtime details are in the delivered external report.

## Revision 4: selected version, artwork and fullscreen cursor compatibility

Versions, Home and launch actions now share one selected cluster. Clicking a release card or choosing an instance updates Home immediately. The selected cluster ID is saved in launcher settings and restored on startup; if the cluster was removed, Home chooses an existing cluster. Settings writes from normal UI edits are serialized and take the latest snapshot so rapid selections cannot leave an older queued snapshot on disk.

Seven original voxel landscapes are embedded for 1.8.9, 1.21.1, 1.21.10, 1.21.11, 26.1.x, 26.2 and 26.3. Cards, sidebars and the dynamic Home background use the same release artwork, including when remote artwork is unavailable. User-selected covers retain priority. Other versions retain the existing remote/fallback behavior. Assets were generated with the built-in image_gen tool; exact prompts and paths are recorded in distribution/art-prompts.json.

On Windows, Settings > Minecraft includes Fullscreen cursor compatibility, enabled by default. At the next launch it sets existing supported Minecraft options exclusiveFullscreen and allowCursorChanges to false, using borderless fullscreen and the standard menu cursor. It preserves fullscreen on/off, raw mouse input and all unrelated options. It does not create a new options file or invent settings absent from an older game's file. For a first-ever game launch, close Minecraft once to let it save options, then relaunch to apply the workaround. Disabling the launcher toggle stops reapplying it; choose the desired fullscreen/cursor settings inside Minecraft afterward. Windows cursor, driver, display and protection settings are unchanged. This is a compatibility workaround, not a guarantee against every game/mod/driver cursor bug.

Mojang documents the Exclusive Fullscreen option in the 26.1 pre-release notes: https://www.minecraft.net/nl-nl/article/minecraft-26-1-pre-release-2 . GLFW cursor/input behavior is documented at https://www.glfw.org/docs/latest/input.html .

New automated checks passed: two core cursor-option preservation/idempotence tests, two settings backward-compatibility/selection-roundtrip tests, and three release-art mapping tests executing the exact production source in a native Rust harness. Final runtime and installer validation is recorded in the delivered validation report.
