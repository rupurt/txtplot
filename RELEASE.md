# Release Process

This repository currently uses a straightforward manual release process for the published `txtplot` crate.

## Release Contract

- Versioning follows [Semantic Versioning](https://semver.org/).
- `Cargo.toml` is the canonical version source.
- Release tags should use the `vX.Y.Z` form.

## Release Checklist

### 1. Finalize the version

Update the crate version in `Cargo.toml`.

```toml
[package]
version = "0.1.0"
```

### 2. Update user-facing surfaces

If the release changes public behavior, review and update:

- `README.md`
- `GUIDE.md`
- examples in `examples/`
- any release notes or changelog material you keep externally

### 3. Run the project checks

Use the existing workflow commands before tagging a release:

```bash
just check
just flake-check
```

If the flake inputs changed as part of the release, refresh the lockfile:

```bash
nix flake lock
```

### 4. Commit the release

Create a release commit after the version and docs are in place.

```bash
git add Cargo.toml README.md flake.nix flake.lock
git commit -m "chore: release v0.1.0"
```

### 5. Tag the release

```bash
git tag v0.1.0
git push origin main --tags
```

### 6. Publish the crate

If this release is intended for crates.io:

```bash
cargo publish
```

## Post-release Checks

After publication:

1. Verify the new version appears on crates.io
2. Verify the git tag matches the published version
3. Confirm the README and examples still describe the released API accurately

## Notes

- Keep release commits atomic.
- Prefer doc updates in the same release slice instead of fixing documentation drift later.
- If automated release infrastructure is introduced later, update this document rather than letting the manual checklist silently rot.
