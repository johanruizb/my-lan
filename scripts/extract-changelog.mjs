#!/usr/bin/env node

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");

function parseArgs(args) {
  const options = {
    version: null,
    file: resolve(repoRoot, "CHANGELOG.md"),
    output: null,
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--version" || arg === "-v") {
      options.version = args[++i];
    } else if (arg === "--file" || arg === "-f") {
      options.file = resolve(process.cwd(), args[++i]);
    } else if (arg === "--output" || arg === "-o") {
      options.output = resolve(process.cwd(), args[++i]);
    } else if (arg === "--help" || arg === "-h") {
      console.log(`Usage: extract-changelog.mjs --version <version> [--file <path>] [--output <path>]`);
      process.exit(0);
    }
  }

  if (!options.version) {
    console.error("Error: --version is required.");
    process.exit(1);
  }

  return options;
}

function escapeRegex(string) {
  return string.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

export function extractReleaseNotes(changelogContent, version) {
  const cleanVersion = version.replace(/^v/, "").trim();
  const headerRegex = new RegExp(`^##\\s*\\[v?${escapeRegex(cleanVersion)}\\][^\r\n]*$`, "m");
  const match = headerRegex.exec(changelogContent);

  if (!match) {
    throw new Error(`Version ${cleanVersion} not found in changelog.`);
  }

  const startIndex = match.index + match[0].length;
  const remaining = changelogContent.slice(startIndex);

  // Match the next version header starting with "## " or "## ["
  const nextHeaderMatch = /\r?\n##\s+/m.exec(remaining);
  const sectionContent = nextHeaderMatch
    ? remaining.slice(0, nextHeaderMatch.index)
    : remaining;

  const trimmed = sectionContent.trim();
  if (!trimmed) {
    throw new Error(`Changelog section for version ${cleanVersion} is empty.`);
  }

  return trimmed;
}

function main() {
  const options = parseArgs(process.argv.slice(2));

  let changelogContent;
  try {
    changelogContent = readFileSync(options.file, "utf8");
  } catch (err) {
    console.error(`Error reading changelog file at ${options.file}:`, err.message);
    process.exit(1);
  }

  try {
    const notes = extractReleaseNotes(changelogContent, options.version);
    if (options.output) {
      writeFileSync(options.output, notes + "\n", "utf8");
    } else {
      process.stdout.write(notes + "\n");
    }
  } catch (err) {
    console.error(`Error extracting changelog: ${err.message}`);
    process.exit(1);
  }
}

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  main();
}
