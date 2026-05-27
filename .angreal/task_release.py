import os
import re
import sys
import angreal

# .angreal lives at <repo>/.angreal, so the workspace root is one level up.
ROOT = os.path.abspath(os.path.join(angreal.get_root(), '..'))

SEMVER_RE = re.compile(r'^\d+\.\d+\.\d+$')

# Every manifest that must carry the same version for a release. The desktop app
# and the Claude Code plugin are versioned in separate files; bumping them by hand
# lets them drift (the plugin shipped at an old version more than once), so this
# task moves all of them together.
#
# Each entry: (relative path, regex, expected number of replacements). The regex
# has two capture groups wrapping the version so the replacement only rewrites the
# digits and never touches surrounding punctuation.
TARGETS = [
    # Rust workspace version — only the [workspace.package] entry, not dep versions.
    ('Cargo.toml',
     r'(\[workspace\.package\]\s*\nversion\s*=\s*")[^"]+(")', 1),
    # GUI npm package.
    ('crates/metis-docs-gui/package.json',
     r'("version":\s*")[^"]+(")', 1),
    # npm lockfile — both metis-docs-gui entries (root + packages[""]).
    ('crates/metis-docs-gui/package-lock.json',
     r'("name": "metis-docs-gui",\s*\n\s*"version": ")[^"]+(")', 2),
    # Tauri bundle config.
    ('crates/metis-docs-gui/src-tauri/tauri.conf.json',
     r'("version":\s*")[^"]+(")', 1),
    # Claude Code plugin manifest.
    ('plugins/metis/.claude-plugin/plugin.json',
     r'("version":\s*")[^"]+(")', 1),
    # Marketplace entry for the plugin.
    ('.claude-plugin/marketplace.json',
     r'("version":\s*")[^"]+(")', 1),
]


def _bump_file(rel_path, pattern, new_version, expected):
    path = os.path.join(ROOT, rel_path)
    try:
        with open(path, 'r') as f:
            content = f.read()
    except FileNotFoundError:
        print(f"✗ {rel_path}: file not found — has it moved? Update TARGETS in task_release.py.")
        sys.exit(1)

    new_content, count = re.subn(
        pattern,
        lambda m: m.group(1) + new_version + m.group(2),
        content,
    )

    if count != expected:
        print(f"✗ {rel_path}: expected {expected} replacement(s) but made {count}. "
              f"The file format likely changed — update the pattern in task_release.py.")
        sys.exit(1)

    if new_content != content:
        with open(path, 'w') as f:
            f.write(new_content)

    field_word = 'field' if count == 1 else 'fields'
    print(f"  ✓ {rel_path} ({count} {field_word})")


@angreal.command(
    name='bump-version',
    about='Set the project version across every release manifest (Cargo workspace, '
          'GUI npm package + lockfile, Tauri config, Claude plugin, marketplace)',
    when_to_use=['Cutting a new release', 'Immediately before tagging a vX.Y.Z release'],
    when_not_to_use=['Bumping a single component in isolation',
                     'Changes that will not ship as a tagged release'],
)
@angreal.argument(
    name='version',
    help='New semantic version, e.g. 2.1.0',
    required=True,
)
def bump_version(version):
    """Rewrite the version string in every manifest that must move together for a release.

    Fails loudly if any file is missing or a pattern matches the wrong number of
    times, so a silent partial bump (the bug this task exists to prevent) can't
    slip through.
    """
    if not SEMVER_RE.match(version):
        print(f"Error: '{version}' is not a valid semantic version "
              f"(expected MAJOR.MINOR.PATCH, e.g. 2.1.0)")
        sys.exit(1)

    print(f"Bumping all release manifests to {version}:\n")
    for rel_path, pattern, expected in TARGETS:
        _bump_file(rel_path, pattern, version, expected)

    print(f"\n✓ All manifests set to {version}.")
    print("Next steps:")
    print(f"  git commit -am 'release: v{version}'")
    print(f"  git tag -a v{version} -m 'Metis v{version}'")
    print(f"  git push origin main && git push origin v{version}")
    return 0
