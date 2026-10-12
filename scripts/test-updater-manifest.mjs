import { mkdtempSync, writeFileSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import assert from "node:assert";
import { generateManifest } from "./generate-updater-manifest.mjs";

const tempDir = mkdtempSync(join(tmpdir(), "mylan-updater-test-"));

try {
    const linuxSig = "dW50cnVzdGVkLWxpbnV4LXNpZw==";
    const windowsSig = "dW50cnVzdGVkLXdpbmRvd3Mtc2ln==";

    // Crear archivos simulados de release
    writeFileSync(
        join(tempDir, "MyLAN_0.5.9_amd64.AppImage.tar.gz"),
        "fake-tar-gz",
    );
    writeFileSync(
        join(tempDir, "MyLAN_0.5.9_amd64.AppImage.tar.gz.sig"),
        linuxSig,
    );

    writeFileSync(join(tempDir, "MyLAN_0.5.9_x64-setup.exe"), "fake-exe");
    writeFileSync(join(tempDir, "MyLAN_0.5.9_x64-setup.exe.sig"), windowsSig);

    // Archivos adicionales que no deben interferir
    writeFileSync(
        join(tempDir, "mylan-cli-x86_64-unknown-linux-gnu.tar.gz"),
        "fake-cli",
    );

    const manifest = generateManifest({
        version: "0.5.9",
        dir: tempDir,
        repo: "johanruizb/my-lan",
        notes: "Test release notes",
    });

    assert.strictEqual(manifest.version, "0.5.9");
    assert.strictEqual(manifest.notes, "Test release notes");
    assert.ok(manifest.pub_date);

    assert.ok(manifest.platforms["linux-x86_64"]);
    assert.strictEqual(manifest.platforms["linux-x86_64"].signature, linuxSig);
    assert.strictEqual(
        manifest.platforms["linux-x86_64"].url,
        "https://github.com/johanruizb/my-lan/releases/download/v0.5.9/MyLAN_0.5.9_amd64.AppImage.tar.gz",
    );

    assert.ok(manifest.platforms["windows-x86_64"]);
    assert.strictEqual(
        manifest.platforms["windows-x86_64"].signature,
        windowsSig,
    );
    assert.strictEqual(
        manifest.platforms["windows-x86_64"].url,
        "https://github.com/johanruizb/my-lan/releases/download/v0.5.9/MyLAN_0.5.9_x64-setup.exe",
    );

    const written = JSON.parse(
        readFileSync(join(tempDir, "latest.json"), "utf8"),
    );
    assert.deepStrictEqual(written, manifest);

    console.log("✓ Todos los tests de generate-updater-manifest pasaron!");
} finally {
    rmSync(tempDir, { recursive: true, force: true });
}
