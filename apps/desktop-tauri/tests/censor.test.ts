// AC-15: Tests unit de `lib/censor.ts` — primer test del pipeline, cero mock.
// Valida el catálogo de enmascaramiento (maskIp/maskMac/isSensitive) y el
// dispatch por nombre de campo. Sin dependencias de React ni IPC.

import {
    isMacField,
    isSensitive,
    maskCidr,
    maskDns,
    maskHostname,
    maskIp,
    maskMac,
    maskValue,
} from "@/lib/censor";

describe("maskMac", () => {
    it("retorna el placeholder constante ••••", () => {
        expect(maskMac()).toBe("••••");
    });
});

describe("maskIp", () => {
    it("enmascara los dos últimos octetos de una IPv4", () => {
        expect(maskIp("192.168.1.42")).toBe("192.168.*.*");
    });

    it("enmascara los últimos 3 grupos de una IPv6 larga", () => {
        // 6 grupos → conserva los 3 primeros
        expect(maskIp("2001:db8::1:2:3")).toBe("2001:db8::*:*:*");
    });

    it("IPv6 corta (<=3 grupos) retorna *:*:*", () => {
        expect(maskIp("fe80::1")).toBe("*:*:*");
    });

    it("cadena sin formato reconocido retorna *", () => {
        expect(maskIp("no-ip")).toBe("*");
    });
});

describe("maskHostname", () => {
    it("conserva la primera etiqueta y enmascara el resto", () => {
        expect(maskHostname("router.lan")).toBe("router.*");
    });

    it("sin punto retorna *", () => {
        expect(maskHostname("localhost")).toBe("*");
    });
});

describe("maskCidr", () => {
    it("enmascara la dirección pero conserva el prefix-len", () => {
        // maskIp siempre cero los dos últimos octetos → "192.168.*.*/24".
        expect(maskCidr("192.168.1.0/24")).toBe("192.168.*.*/24");
    });

    it("sin slash delega a maskIp", () => {
        expect(maskCidr("10.0.0.1")).toBe("10.0.*.*");
    });
});

describe("maskDns", () => {
    it("enmascara cada servidor como IP y los une con coma", () => {
        expect(maskDns(["8.8.8.8", "1.1.1.1"])).toBe("8.8.*.*, 1.1.*.*");
    });
});

describe("isSensitive", () => {
    const sensitiveFields = [
        "primary_ip",
        "primary_mac",
        "hostname",
        "display_name",
        "gateway_ip",
        "gateway_mac",
        "dns_servers",
        "cidr",
        "ip",
        "mac",
        "device_ip",
    ];

    it.each(sensitiveFields)("marca '%s' como sensible", (field) => {
        expect(isSensitive(field)).toBe(true);
    });

    const nonSensitiveFields = [
        "vendor",
        "manufacturer",
        "banner",
        "product",
        "version",
        "port",
        "notes",
        "device_type",
        "os_family",
        "confidence",
    ];

    it.each(nonSensitiveFields)("marca '%s' como NO sensible", (field) => {
        expect(isSensitive(field)).toBe(false);
    });
});

describe("isMacField", () => {
    it.each(["primary_mac", "mac", "gateway_mac"])(
        "marca '%s' como campo MAC",
        (field) => {
            expect(isMacField(field)).toBe(true);
        },
    );

    it("marca primary_ip como NO MAC", () => {
        expect(isMacField("primary_ip")).toBe(false);
    });
});

describe("maskValue — dispatch por nombre de campo", () => {
    it("MAC-family retorna el placeholder constante", () => {
        expect(maskValue("primary_mac", "aa:bb:cc:dd:ee:ff")).toBe("••••");
        expect(maskValue("mac", "aa:bb:cc:dd:ee:ff")).toBe("••••");
        expect(maskValue("gateway_mac", "aa:bb:cc:dd:ee:ff")).toBe("••••");
    });

    it("ip/primary_ip/gateway_ip/device_ip aplican maskIp", () => {
        expect(maskValue("ip", "192.168.1.42")).toBe("192.168.*.*");
        expect(maskValue("primary_ip", "10.0.0.1")).toBe("10.0.*.*");
        expect(maskValue("gateway_ip", "192.168.1.1")).toBe("192.168.*.*");
        expect(maskValue("device_ip", "172.16.0.5")).toBe("172.16.*.*");
    });

    it("hostname/display_name aplican maskHostname", () => {
        expect(maskValue("hostname", "router.lan")).toBe("router.*");
        expect(maskValue("display_name", "my-pc.local")).toBe("my-pc.*");
    });

    it("cidr aplica maskCidr conservando el prefix-len", () => {
        expect(maskValue("cidr", "192.168.1.0/24")).toBe("192.168.*.*/24");
    });

    it("dns_servers enmascara cada entrada separada por coma", () => {
        expect(maskValue("dns_servers", "8.8.8.8, 1.1.1.1")).toBe(
            "8.8.*.*, 1.1.*.*",
        );
    });

    it("campo no sensible pasa el valor sin cambios", () => {
        expect(maskValue("vendor", "ExampleVendor")).toBe("ExampleVendor");
        expect(maskValue("notes", "texto libre")).toBe("texto libre");
    });
});