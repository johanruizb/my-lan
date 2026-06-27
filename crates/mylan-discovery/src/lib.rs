//! `mylan-discovery` — descubrimiento de hosts en la LAN.
//!
//! Funciones async concretas por técnica (sin trait de estrategia, principio P3) que
//! producen `Observation`s: detección de interfaz/gateway/CIDR (`netdev`), lectura de
//! `/proc/net/arp`, barrido TCP-connect, mDNS, SSDP, ICMP no-root (best-effort) y, con
//! privilegios, ARP sweep (`pnet_datalink`) + ICMP raw. Pipeline de dos fases:
//! liveness → enrichment.
//!
//! Estado: esqueleto (Paso 1). Implementación en Paso 4.
