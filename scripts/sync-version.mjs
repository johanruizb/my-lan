#!/usr/bin/env node

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const checkOnly = process.argv.includes("--check");

const targets = [
  "package.json",
  "apps/desktop-tauri/package.json",
  "apps/desktop-tauri/package-lock.json",
  "apps/desktop-tauri/src-tauri/Cargo.toml",
  "apps/desktop-tauri/src-tauri/tauri.conf.json",
];

function readText(relativePath) {
  return readFileSync(resolve(repoRoot, relativePath), "utf8");
}

function writeText(relativePath, content) {
  writeFileSync(resolve(repoRoot, relativePath), content);
}

function tomlSection(text, sectionName) {
  const lines = text.split(/\r?\n/);
  const header = `[${sectionName}]`;
  const start = lines.findIndex((line) => line.trim() === header);
  if (start === -1) {
    throw new Error(`Missing [${sectionName}] section`);
  }

  const end = lines.findIndex(
    (line, index) => index > start && /^\[[^\]]+\]\s*$/.test(line),
  );

  return lines.slice(start + 1, end === -1 ? undefined : end).join("\n");
}

function workspaceVersion() {
  const section = tomlSection(readText("Cargo.toml"), "workspace.package");
  const match = section.match(/^version\s*=\s*"([^"]+)"\s*$/m);
  if (!match) {
    throw new Error('Missing version = "..." in [workspace.package]');
  }
  return match[1];
}

function replacePackageVersionToml(text, version) {
  const section = tomlSection(text, "package");
  if (!/^version\s*=\s*"[^"]+"\s*$/m.test(section)) {
    throw new Error('Missing version = "..." in [package]');
  }
  return text.replace(
    /^(\[package\][\s\S]*?^version\s*=\s*)"[^"]+"/m,
    `$1"${version}"`,
  );
}

function updatePackageJson(relativePath, version) {
  const original = readText(relativePath);
  const json = JSON.parse(original);
  json.version = version;
  return JSON.stringify(json, null, 2) + "\n";
}

function packageJsonVersions(relativePath) {
  return [JSON.parse(readText(relativePath)).version];
}

function updatePackageLock(relativePath, version) {
  const original = readText(relativePath);
  const json = JSON.parse(original);
  json.version = version;
  if (!json.packages?.[""]) {
    throw new Error(`Missing root package entry in ${relativePath}`);
  }
  json.packages[""].version = version;
  return JSON.stringify(json, null, 2) + "\n";
}

function packageLockVersions(relativePath) {
  const json = JSON.parse(readText(relativePath));
  if (!json.packages?.[""]) {
    throw new Error(`Missing root package entry in ${relativePath}`);
  }
  return [json.version, json.packages[""].version];
}

function updateTauriConfig(relativePath, version) {
  const original = readText(relativePath);
  const json = JSON.parse(original);
  json.version = version;
  return JSON.stringify(json, null, 2) + "\n";
}

function tauriConfigVersions(relativePath) {
  return [JSON.parse(readText(relativePath)).version];
}

function cargoPackageVersions(relativePath) {
  const section = tomlSection(readText(relativePath), "package");
  const match = section.match(/^version\s*=\s*"([^"]+)"\s*$/m);
  if (!match) {
    throw new Error(`Missing package version in ${relativePath}`);
  }
  return [match[1]];
}

const readers = new Map([
  ["package.json", packageJsonVersions],
  ["apps/desktop-tauri/package.json", packageJsonVersions],
  ["apps/desktop-tauri/package-lock.json", packageLockVersions],
  ["apps/desktop-tauri/src-tauri/Cargo.toml", cargoPackageVersions],
  ["apps/desktop-tauri/src-tauri/tauri.conf.json", tauriConfigVersions],
]);

const version = workspaceVersion();
const updates = new Map([
  ["package.json", updatePackageJson("package.json", version)],
  [
    "apps/desktop-tauri/package.json",
    updatePackageJson("apps/desktop-tauri/package.json", version),
  ],
  [
    "apps/desktop-tauri/package-lock.json",
    updatePackageLock("apps/desktop-tauri/package-lock.json", version),
  ],
  [
    "apps/desktop-tauri/src-tauri/Cargo.toml",
    replacePackageVersionToml(
      readText("apps/desktop-tauri/src-tauri/Cargo.toml"),
      version,
    ),
  ],
  [
    "apps/desktop-tauri/src-tauri/tauri.conf.json",
    updateTauriConfig("apps/desktop-tauri/src-tauri/tauri.conf.json", version),
  ],
]);

const stale = targets.filter((relativePath) =>
  readers.get(relativePath)(relativePath).some((targetVersion) => targetVersion !== version),
);

if (checkOnly) {
  if (stale.length > 0) {
    console.error(
      `Version mismatch: expected ${version} from Cargo.toml in:\n` +
        stale.map((path) => `- ${path}`).join("\n"),
    );
    console.error("Run `npm run version:sync` from the repository root.");
    process.exit(1);
  }
  console.log(`All app versions match ${version}.`);
  process.exit(0);
}

for (const relativePath of stale) {
  writeText(relativePath, updates.get(relativePath));
}

console.log(
  stale.length === 0
    ? `All app versions already match ${version}.`
    : `Synced ${stale.length} file(s) to ${version}.`,
);
