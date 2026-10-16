#!/usr/bin/env node

import assert from "node:assert";
import { extractReleaseNotes } from "./extract-changelog.mjs";

const sampleChangelog = `
# Changelog

## [Unreleased]

## [1.2.0] - 2026-10-16 (abc1234)

### Added
- Feature A
- Feature B

### Fixed
- Bug fix C

**Full changelog:** https://github.com/foo/bar/compare/v1.1.0...v1.2.0

## [1.1.0] — 2026-09-01

### Added
- Feature old
`;

// Test 1: Extract standard version
const notes120 = extractReleaseNotes(sampleChangelog, "1.2.0");
assert.ok(notes120.includes("### Added\n- Feature A"));
assert.ok(notes120.includes("- Bug fix C"));
assert.ok(notes120.includes("**Full changelog:**"));
assert.strictEqual(notes120.includes("Feature old"), false);

// Test 2: Leading 'v' normalization
const notesWithV = extractReleaseNotes(sampleChangelog, "v1.2.0");
assert.strictEqual(notesWithV, notes120);

// Test 3: Last version in file (no following header)
const notes110 = extractReleaseNotes(sampleChangelog, "1.1.0");
assert.ok(notes110.includes("Feature old"));
assert.strictEqual(notes110.includes("Feature A"), false);

// Test 4: Missing version throws error
assert.throws(() => {
  extractReleaseNotes(sampleChangelog, "9.9.9");
}, /not found/i);

console.log("✓ Todos los tests de extract-changelog pasaron!");
