# Security Policy

## Supported versions

Only the latest `main` branch of this repository is supported with security
fixes.

## Reporting a vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Use GitHub's private vulnerability reporting: **Security → Report a
vulnerability** on this repository. Include a description, reproduction steps,
and your assessment of impact if possible.

Corridor moves real money and personal-eligibility data. Reports touching the
following are especially valuable:

* the fail-closed guarantees of the verifier (`ultrahonk_verifier`) and the
  mock's replacement path
* nullifier uniqueness / linkage across corridors
* policy binding (`verifier`, `vk_hash`) enforcement in `enter()`
* the `PublicInputs` canonical-word decoding
* secret handling in the SDK (`holder_secret`, issuer keys)

We will acknowledge reports within 7 days and aim to ship a fix within 90
days. We credit reporters in the release notes unless you prefer anonymity.

## Threat model (contract-specific)

| Risk | Impact | Status / mitigation |
|------|--------|---------------------|
| `verifier_mock` deployed to a real corridor | Any proof accepted | Clearly named; a policy's `verifier` is the operator's choice. M3 shipped the real UltraHonk verifier — point new corridors at it and pin its `vk_hash`. |
| Verifier soundness bug | Fake passes | Use the audited reference verifier; pin `vk_hash`; every policy has a `paused` kill switch. |
| Compromised issuer key | Bad statements signed until noticed | Short `expiry` caps the window; the issuer bumps `cred_epoch` and operators raise `min_cred_epoch` (`set_min_cred_epoch`, monotonic) to bulk-revoke; a corridor can drop the issuer from `accepted_issuers` instantly. |
| Malformed public-input words | Decoding confusion | `PublicInputs::decode` rejects non-canonical numeric words (`BadPublicInputs`); the circuit range-constrains the same fields. |
