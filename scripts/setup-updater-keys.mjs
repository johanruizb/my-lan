#!/usr/bin/env node

import { existsSync, readFileSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";
import { execSync } from "node:child_process";

const keyPath = join(homedir(), ".tauri", "mylan.key");
const pubKeyPath = `${keyPath}.pub`;

console.log("=== Asistente de Claves de Firma para MyLAN Auto-Updater ===\n");

if (!existsSync(keyPath) || !existsSync(pubKeyPath)) {
    console.log("Generando un nuevo par de claves con cargo tauri signer generate...");
    try {
        execSync(`cargo tauri signer generate -w "${keyPath}" --ci -p ""`, {
            stdio: "inherit",
        });
    } catch (e) {
        console.error("Error generando claves con cargo tauri:", e.message);
        process.exit(1);
    }
} else {
    console.log(`✓ Claves encontradas en: ${keyPath}`);
}

const pubKey = readFileSync(pubKeyPath, "utf8").trim();
const privateKey = readFileSync(keyPath, "utf8").trim();

console.log("\n------------------------------------------------------------");
console.log("1. CLAVE PÚBLICA (Ya configurada en tauri.conf.json):");
console.log("------------------------------------------------------------");
console.log(pubKey);

console.log("\n------------------------------------------------------------");
console.log("2. CLAVE PRIVADA PARA GITHUB ACTIONS (SECRETS):");
console.log("------------------------------------------------------------");
console.log("Ve a tu repositorio en GitHub:");
console.log("Settings -> Secrets and variables -> Actions -> New repository secret\n");
console.log("Nombre del secret: TAURI_SIGNING_PRIVATE_KEY");
console.log("Valor (copia el bloque completo a continuación):\n");
console.log(privateKey);

console.log("\n------------------------------------------------------------");
console.log("Nombre del secret opcional: TAURI_SIGNING_PRIVATE_KEY_PASSWORD");
console.log("Valor: (dejar vacío si no usaste contraseña)\n");
console.log("------------------------------------------------------------\n");
