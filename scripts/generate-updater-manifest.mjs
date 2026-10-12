#!/usr/bin/env node

import { readdirSync, readFileSync, writeFileSync, existsSync } from "node:fs";
import { resolve, basename } from "node:path";

function parseArgs() {
    const args = process.argv.slice(2);
    const options = {
        version: "",
        dir: "release-assets",
        repo: process.env.GITHUB_REPOSITORY || "johanruizb/my-lan",
        notes: "",
        output: "",
    };

    for (let i = 0; i < args.length; i++) {
        const arg = args[i];
        if (arg === "--version" && i + 1 < args.length) {
            options.version = args[++i];
        } else if (arg === "--dir" && i + 1 < args.length) {
            options.dir = args[++i];
        } else if (arg === "--repo" && i + 1 < args.length) {
            options.repo = args[++i];
        } else if (arg === "--notes" && i + 1 < args.length) {
            options.notes = args[++i];
        } else if (arg === "--output" && i + 1 < args.length) {
            options.output = args[++i];
        }
    }

    if (!options.version) {
        console.error("Error: --version es requerido (ej. 0.5.9 o v0.5.9)");
        process.exit(1);
    }

    return options;
}

export function generateManifest({ version, dir, repo, notes, output }) {
    const cleanVersion = version.replace(/^v/, "");
    const tag = `v${cleanVersion}`;
    const assetsDir = resolve(dir);

    if (!existsSync(assetsDir)) {
        throw new Error(`El directorio de assets no existe: ${assetsDir}`);
    }

    const files = readdirSync(assetsDir);
    const platforms = {};

    // 1. Linux x86_64: busca artefacto updater (.AppImage.tar.gz y .sig)
    const linuxSigFile = files.find(
        (f) =>
            (f.endsWith(".AppImage.tar.gz.sig") || f.endsWith(".tar.gz.sig")) &&
            !f.includes("cli"),
    );

    if (linuxSigFile) {
        const payloadFile = linuxSigFile.replace(/\.sig$/, "");
        if (files.includes(payloadFile)) {
            const sigContent = readFileSync(
                resolve(assetsDir, linuxSigFile),
                "utf8",
            ).trim();
            platforms["linux-x86_64"] = {
                signature: sigContent,
                url: `https://github.com/${repo}/releases/download/${tag}/${payloadFile}`,
            };
            console.log(`✓ Plataforma linux-x86_64 vinculada: ${payloadFile}`);
        } else {
            console.warn(
                `⚠ Firma encontrada (${linuxSigFile}) pero falta el binario ${payloadFile}`,
            );
        }
    }

    // 2. Windows x86_64: busca instalador updater (NSIS .exe o .msi.zip)
    // Se prefiere NSIS (-setup.exe o .exe.sig) sobre MSI
    const windowsSigFile =
        files.find(
            (f) =>
                (f.endsWith("-setup.exe.sig") ||
                    f.endsWith(".nsis.zip.sig") ||
                    (f.endsWith(".exe.sig") && !f.includes("cli"))) &&
                !f.includes("cli"),
        ) ||
        files.find((f) => f.endsWith(".msi.zip.sig"));

    if (windowsSigFile) {
        const payloadFile = windowsSigFile.replace(/\.sig$/, "");
        if (files.includes(payloadFile)) {
            const sigContent = readFileSync(
                resolve(assetsDir, windowsSigFile),
                "utf8",
            ).trim();
            platforms["windows-x86_64"] = {
                signature: sigContent,
                url: `https://github.com/${repo}/releases/download/${tag}/${payloadFile}`,
            };
            console.log(`✓ Plataforma windows-x86_64 vinculada: ${payloadFile}`);
        } else {
            console.warn(
                `⚠ Firma encontrada (${windowsSigFile}) pero falta el binario ${payloadFile}`,
            );
        }
    }

    const platformCount = Object.keys(platforms).length;
    if (platformCount === 0) {
        console.warn(
            "⚠ No se encontraron archivos .sig para ninguna plataforma en " +
                assetsDir,
        );
    }

    const manifest = {
        version: cleanVersion,
        notes: notes || `MyLAN ${tag}`,
        pub_date: new Date().toISOString(),
        platforms,
    };

    const outputPath = output || resolve(assetsDir, "latest.json");
    writeFileSync(outputPath, JSON.stringify(manifest, null, 2) + "\n");
    console.log(`✓ latest.json generado exitosamente en: ${outputPath}`);
    return manifest;
}

// Ejecutar si es invocado directamente vía CLI
const isDirectCall =
    process.argv[1] &&
    resolve(process.argv[1]) === resolve(new URL(import.meta.url).pathname);

if (isDirectCall) {
    const opts = parseArgs();
    generateManifest(opts);
}
