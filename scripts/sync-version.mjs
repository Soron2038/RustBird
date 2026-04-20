#!/usr/bin/env node
// Called by `npm version` as the "version" lifecycle script.
// Reads package.json's (already-bumped) version and propagates it to
// src-tauri/Cargo.toml, src-tauri/tauri.conf.json, and Cargo.lock.

import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const version = JSON.parse(
  readFileSync(join(root, "package.json"), "utf8"),
).version;

const updates = [
  {
    file: "src-tauri/tauri.conf.json",
    // top-level version, anchored to productName to avoid plugin sections
    pattern: /("productName":\s*"[^"]+",\s*\n\s*"version":\s*")[^"]+(")/,
  },
  {
    file: "src-tauri/Cargo.toml",
    // [package] version, lazy-matched to skip other fields in between
    pattern: /(\[package\][\s\S]*?\nversion\s*=\s*")[^"]+(")/,
  },
  {
    file: "src-tauri/Cargo.lock",
    pattern: /(name = "rustbird"\nversion = ")[^"]+(")/,
  },
];

for (const { file, pattern } of updates) {
  const path = join(root, file);
  const before = readFileSync(path, "utf8");
  if (!pattern.test(before)) {
    console.error(`sync-version: pattern not found in ${file}`);
    process.exit(1);
  }
  writeFileSync(path, before.replace(pattern, `$1${version}$2`));
  console.log(`sync-version: ${file} → ${version}`);
}
