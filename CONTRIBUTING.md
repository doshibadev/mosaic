# Contributing to Mosaic

Thanks for wanting to help. We need it.

## How to contribute

### Report a bug
Found something broken? Open an issue. Tell us:
- What you were doing when it broke
- What you expected to happen
- What actually happened
- Your OS and Mosaic version

### Suggest a feature
Have an idea? Open an issue first before you start coding. We can discuss whether it fits the project. No point building something we won't merge.

### Write code
1. Fork the repo
2. Create a branch: `git checkout -b fix/your-fix` or `git checkout -b feature/your-feature`
3. Make your changes
4. Test locally (`cargo build`, `cargo test`, etc)
5. Commit with a clear message: `Fix XML injection bug` not `stuff`
6. Push and open a PR

We'll review it. Might take a bit, but we'll get to it.

## Code style

- Rust: Use `cargo fmt` before committing. We're not picky, but consistent is better.
- TypeScript: Existing code uses 2-space tabs. Match that.
- Lua: Same as TypeScript, 2 spaces.

## Project structure

- `cli/` - The command-line tool. This is what users run.
- `registry/` - The backend API. Handles package metadata, uploads, auth.
- `website/` - The landing page and package browser. Next.js.

Pick whichever part interests you most.

## Testing

- If you're fixing a bug, add a test that would fail without your fix.
- If you're adding a feature, test it works before submitting the PR.
- We don't have comprehensive test coverage yet, but new code should have *some*.

## Commits

Keep commits clean and focused. One fix per commit. Bad commits make history hard to read later.

Good:
```
Fix panic when package name contains slash
Add validation for semantic versions
```

Bad:
```
stuff
wip
fix things and also add feature and also cleanup
```

## Questions?

Open an issue and ask. No such thing as a dumb question. We all started somewhere.

Thanks for the help.