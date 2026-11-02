# Code signing policy

Maintainer and proposed release approver: [chill498](https://github.com/chill498). No additional reviewers or team members are claimed.

Status: unsigned. A free SignPath Foundation application was submitted on 2 November 2026 and is pending review. No signing approval, certificate, endorsement or signing access has been obtained for this fork. Upstream's signing approval applies to upstream only.

Before signing access is used, the maintainer must enable MFA on repository and signing accounts and establish the roles required by the provider. Completion of those steps is not claimed here. The provider decides whether this fork, its source/build provenance and project reputation qualify. The maintainer must personally review and approve releases; no automatic approval is assumed.

Only this fork's reviewed source builds may be submitted under its own approved signing identity. The launcher, portable wrapper, generated uninstaller and installer need signing coverage. Valid signatures on Microsoft runtime DLLs must be preserved. Returned signatures, trusted chains and timestamps must be checked before a release is described as signed. Files must not be modified after signing.

The manual CI currently creates **unsigned review artifacts only**. Its output cannot be represented as a signed release. Production signing integration is pending project approval and configuration. Any provider credit will be added only once signing is actually provided.

A trusted signature establishes publisher/integrity information; it does not guarantee antivirus safety or immediate SmartScreen reputation. Windows protection must remain enabled.

References: https://signpath.org/terms.html and https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation
