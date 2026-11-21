# 0001: AGPL-3.0-or-later with a Contributor License Agreement

Date: 2026-11-20. Status: accepted.

## Context

Stepanova is open source and welcomes contributions, and a possible future is
acquisition by reMarkable. An acquirer needs to be able to obtain clean,
complete rights to the code. Stepanova also includes a network server, and the
community app stack it may integrate with (AppLoad) is GPL-3.0.

## Decision

- License all code under **AGPL-3.0-or-later**. Copyleft keeps third parties
  from shipping closed forks; the Affero clause extends that to modified
  server deployments; and it is compatible with GPL-3.0 components.
- Require every contributor to agree to the [CLA](../../CLA.md). It grants the
  maintainer the right to relicense contributions under any terms, including
  proprietary ones, and to transfer those rights to a successor owner.

Together these keep a single party able to relicense or sell the project,
while the public version stays free software.

## Consequences

- A CLA discourages some contributors. This is accepted in exchange for clean
  ownership.
- Third-party code may only be vendored if its license is compatible with
  AGPL-3.0 and does not prevent relicensing of Stepanova's own code. Code
  under GPL-3.0 (such as client code from AppLoad) is compatible but
  cannot be relicensed; keep it isolated so it can be replaced.
- The CLA is enforced by a pull request checkbox. Switch to an automated tool
  (such as CLA Assistant) once external contributions become regular. Have a
  lawyer review the CLA before accepting significant outside contributions.
