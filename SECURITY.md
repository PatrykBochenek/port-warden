# Security Policy

portly manages processes and can terminate them, so security is taken
seriously.

## Reporting a Vulnerability

**Please do not open a public issue for security vulnerabilities.**

Instead, report vulnerabilities privately via GitHub's private vulnerability
reporting:

<https://github.com/PatrykBochenek/portly/security/advisories/new>

Or by emailing the maintainer directly:
[contact@patrykbochenek.com](mailto:contact@patrykbochenek.com).

You will receive a response within 48 hours. Please include:

- The affected version(s)
- A description of the vulnerability and its impact
- Steps to reproduce, or a minimal proof of concept

## Scope

Things we care most about:

- Privilege/security issues in the process lookup or `kill()` paths
  (e.g. killing the wrong process, TOCTOU abuse of `find_free()`)
- Unsafe code in the Rust extension (`src/lib.rs`)
- Supply-chain issues (dependencies, build/publish pipeline)

## Disclosure

Once a fix is ready, we coordinate disclosure via a GitHub Security Advisory.
We aim to credit reporters in the advisory unless they prefer to stay
anonymous.

## Supported versions

| Version | Supported          |
| ------- | ------------------ |
| latest  | :white_check_mark: |

Only the latest release receives security fixes; users are encouraged to stay
current.