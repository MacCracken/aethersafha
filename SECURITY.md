# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

1. **Do not** open a public issue
2. Email security details to the maintainers
3. Include steps to reproduce if possible
4. Allow reasonable time for a fix before disclosure

## Security Model

Aethersafha enforces:
- Per-agent screen capture permissions with rate limiting
- Sandboxed plugin execution via Landlock + seccomp
- Agent permission dialogs for sensitive operations
- Audit logging of all security-relevant events
