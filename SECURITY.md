# Security Policy

## Reporting a vulnerability

If you find a security vulnerability, **do not open a public issue**. That gives everyone the exploit before we can fix it.

Instead, email `security@getmosaic.run` with:
- What the vulnerability is
- Where it is (file, function, etc)
- How to reproduce it
- Any proof of concept (if you have one)

We'll acknowledge it within 48 hours and work on a fix. Once it's patched and released, you can publicly disclose it if you want credit.

## What counts as a vulnerability

- Authentication/authorization bypass
- Credential exposure (tokens, passwords, API keys)
- SQL injection or similar code execution
- Data exposure or corruption
- Denial of service

**Not vulnerabilities:**
- Typos in docs
- Performance issues
- Feature requests
- Code style complaints

## Our process

1. You report it privately
2. We verify and assess severity
3. We fix it
4. We release a patch
5. You can disclose it publicly if you want

We'll give you credit in the release notes unless you want to stay anonymous.

## Scope

- The CLI tool: in scope
- The registry API: in scope
- The website: in scope
- Polytoria's platform itself: not our scope, report to Polytoria
- User scripts running on Polytoria: not our scope

Thanks for helping keep Mosaic safe.