import { mkdtempSync, writeFileSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import assert from "node:assert";
import { generateManifest } from "./generate-updater-manifest.mjs";

const tempDir = mkdtempSync(join(tmpdir(), "mylan-updater-test-"));

try {
    const appimageSig = "dW50cnVzdGVkLWFwcGltYWdlLXNpZw==";
    const debSig = "dW50cnVzdGVkLWRlYi1zaWc==";
    const windowsSig = "dW50cnVzdGVkLXdpbmRvd3Mtc2ln==";

    // Crear archivos simulados de release según nombres reales de Tauri
    writeFileSync(
        join(tempDir, "MyLAN_0.6.0_amd64.AppImage"),
        "fake-appimage",
    );
    writeFileSync(
        join(tempDir, "MyLAN_0.6.0_amd64.AppImage.sig"),
        appimageSig,
    );

    writeFileSync(join(tempDir, "MyLAN_0.6.0_amd64.deb"), "fake-deb");
    writeFileSync(join(tempDir, "MyLAN_0.6.0_amd64.deb.sig"), debSig);

    writeFileSync(join(tempDir, "MyLAN_0.6.0_x64-setup.exe"), "fake-exe");
    writeFileSync(join(tempDir, "MyLAN_0.6.0_x64-setup.exe.sig"), windowsSig);

    // Archivos adicionales que no deben interferir
    writeFileSync(
        join(tempDir, "mylan-cli-x86_64-unknown-linux-gnu.tar.gz"),
        "fake-cli",
    );

    const manifest = generateManifest({
        version: "0.6.0",
        dir: tempDir,
        repo: "johanruizb/my-lan",
        notes: "Test release notes",
    });

    assert.strictEqual(manifest.version, "0.6.0");
    assert.strictEqual(manifest.notes, "Test release notes");
    assert.ok(manifest.pub_date);

    // linux-x86_64 y linux-x86_64-appimage
    assert.ok(manifest.platforms["linux-x86_64"]);
    assert.strictEqual(
        manifest.platforms["linux-x86_64"].signature,
        appimageSig,
    );
    assert.strictEqual(
        manifest.platforms["linux-x86_64"].url,
        "https://github.com/johanruizb/my-lan/releases/download/v0.6.0/MyLAN_0.6.0_amd64.AppImage",
    );
    assert.ok(manifest.platforms["linux-x86_64-appimage"]);
    assert.strictEqual(
        manifest.platforms["linux-x86_64-appimage"].signature,
        appimageSig,
    );

    // linux-x86_64-deb
    assert.ok(manifest.platforms["linux-x86_64-deb"]);
    assert.strictEqual(manifest.platforms["linux-x86_64-deb"].signature, debSig);
    assert.strictEqual(
        manifest.platforms["linux-x86_64-deb"].url,
        "https://github.com/johanruizb/my-lan/releases/download/v0.6.0/MyLAN_0.6.0_amd64.deb",
    );

    // windows-x86_64
    assert.ok(manifest.platforms["windows-x86_64"]);
    assert.strictEqual(
        manifest.platforms["windows-x86_64"].signature,
        windowsSig,
    );
    assert.strictEqual(
        manifest.platforms["windows-x86_64"].url,
        "https://github.com/johanruizb/my-lan/releases/download/v0.6.0/MyLAN_0.6.0_x64-setup.exe",
    );

    const written = JSON.parse(
        readFileSync(join(tempDir, "latest.json"), "utf8"),
    );
    assert.deepStrictEqual(written, manifest);

    // Caso 2: Sin AppImage (únicamente deb + tar.gz + windows setup)
    const tempDirReduced = mkdtempSync(join(tmpdir(), "mylan-reduced-test-"));
    try {
        writeFileSync(join(tempDirReduced, "MyLAN_0.6.1_amd64.deb"), "fake-deb");
        writeFileSync(join(tempDirReduced, "MyLAN_0.6.1_amd64.deb.sig"), debSig);
        writeFileSync(join(tempDirReduced, "MyLAN_0.6.1_amd64.tar.gz"), "fake-tar");
        writeFileSync(join(tempDirReduced, "MyLAN_0.6.1_x64-setup.exe"), "fake-exe");
        writeFileSync(join(tempDirReduced, "MyLAN_0.6.1_x64-setup.exe.sig"), windowsSig);

        const manifestReduced = generateManifest({
            version: "0.6.1",
            dir: tempDirReduced,
            repo: "johanruizb/my-lan",
        });

        assert.strictEqual(manifestReduced.platforms["linux-x86_64"].signature, debSig);
        assert.strictEqual(manifestReduced.platforms["linux-x86_64-deb"].signature, debSig);
        assert.strictEqual(manifestReduced.platforms["windows-x86_64"].signature, windowsSig);
        assert.strictEqual(manifestReduced.platforms["linux-x86_64-appimage"], undefined);
    } finally {
        rmSync(tempDirReduced, { recursive: true, force: true });
    }

    console.log("✓ Todos los tests de generate-updater-manifest pasaron!");
} finally {
    rmSync(tempDir, { recursive: true, force: true });
}
