# Network and local-data disclosure

This fork retains upstream network integrations; it is not a network-silent application. Upstream's terms/privacy notices shown by the launcher continue to apply to upstream services. Review settings and provider notices before enabling optional integrations.

The launcher can request version/catalog/art metadata, game and Java/runtime downloads, libraries and assets, mod metadata/downloads from providers such as Modrinth and CurseForge, and upstream service/status information. Selected games and mods have their own independent network behavior and policies.

Microsoft login is optional for offline accounts. If selected, it contacts Microsoft/Minecraft services and stores the resulting account/session data locally. Offline accounts have no authenticated Microsoft credentials. Premium/online services still require valid authentication and applicable ownership.

Upstream settings include Discord integration and crash reporting. Their enabled state is controlled by the corresponding settings; this fork does not claim they are disabled by default. Review the source and upstream notices for the data used by those integrations. The custom launcher automatic updater is disabled for this unofficial build; use its own releases for launcher updates. Game/mod metadata and update operations remain available.

User accounts, settings, database, logs and optional local skin PNGs are stored in the launcher's data directory. The portable wrapper selects a data directory beside the application. The installer packages program files, not test profiles or game data. Local PNG skin selection does not upload that PNG or change arbitrary servers. Exporting a PNG writes a local file; server upload/application is a separate player/server operation.

No claim is made that third-party services or mods never collect data. Do not commit profiles, account files, logs, skins, credentials or downloaded game data to this public repository. Report issues with sensitive details removed.
