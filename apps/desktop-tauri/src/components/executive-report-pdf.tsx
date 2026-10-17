import {
    Document,
    Page,
    Text,
    View,
    StyleSheet,
    pdf,
} from "@react-pdf/renderer";
import type {
    Device,
    NetworkHealthSample,
    SecurityFinding,
    SpeedTestResult,
    Workspace,
} from "@/lib/tauri";
import { maskCidr, maskHostname, maskIp, maskMac } from "@/lib/censor";

const styles = StyleSheet.create({
    page: {
        paddingTop: 36,
        paddingBottom: 48,
        paddingHorizontal: 40,
        fontSize: 9,
        color: "#1e293b",
        fontFamily: "Helvetica",
        backgroundColor: "#ffffff",
    },
    header: {
        flexDirection: "row",
        justifyContent: "space-between",
        alignItems: "flex-start",
        paddingBottom: 16,
        marginBottom: 16,
        borderBottomWidth: 1,
        borderBottomColor: "#b5bfce",
    },
    brandTitle: {
        fontSize: 20,
        fontWeight: "bold",
        color: "#0f172a",
        marginBottom: 4,
    },
    subtitle: {
        fontSize: 10,
        color: "#64748b",
    },
    metaBlock: {
        alignItems: "flex-end",
    },
    metaText: {
        fontSize: 8,
        color: "#64748b",
        marginBottom: 2,
    },
    badgeCensorship: {
        marginTop: 4,
        backgroundColor: "#fef3c7",
        color: "#b45309",
        paddingVertical: 2,
        paddingHorizontal: 6,
        borderRadius: 4,
        fontSize: 7,
        fontWeight: "bold",
    },
    sectionTitle: {
        fontSize: 12,
        fontWeight: "bold",
        color: "#0f172a",
        marginBottom: 8,
        marginTop: 12,
    },
    kpiRow: {
        flexDirection: "row",
        justifyContent: "space-between",
        marginBottom: 16,
    },
    kpiCard: {
        flex: 1,
        marginHorizontal: 3,
        padding: 8,
        backgroundColor: "#f8fafc",
        borderRadius: 4,
        borderWidth: 1,
        borderColor: "#b5bfce",
    },
    kpiValue: {
        fontSize: 14,
        fontWeight: "bold",
        color: "#0f172a",
        marginTop: 2,
    },
    kpiLabel: {
        fontSize: 7,
        color: "#64748b",
        textTransform: "uppercase",
        fontWeight: "bold",
    },
    kpiSub: {
        fontSize: 7,
        color: "#94a3b8",
        marginTop: 2,
    },
    table: {
        width: "100%",
        borderWidth: 1,
        borderColor: "#b5bfce",
        borderRadius: 4,
        overflow: "hidden",
        marginBottom: 12,
    },
    tableHeaderRow: {
        flexDirection: "row",
        backgroundColor: "#f1f5f9",
        borderBottomWidth: 1,
        borderBottomColor: "#b5bfce",
        paddingVertical: 6,
        paddingHorizontal: 8,
    },
    tableRow: {
        flexDirection: "row",
        borderBottomWidth: 1,
        borderBottomColor: "#f1f5f9",
        paddingVertical: 5,
        paddingHorizontal: 8,
        alignItems: "center",
    },
    tableColHeader: {
        fontSize: 8,
        fontWeight: "bold",
        color: "#475569",
    },
    tableCol: {
        fontSize: 8,
        color: "#334155",
    },
    severityCritical: {
        color: "#dc2626",
        fontWeight: "bold",
    },
    severityHigh: {
        color: "#ea580c",
        fontWeight: "bold",
    },
    severityMedium: {
        color: "#d97706",
        fontWeight: "bold",
    },
    severityLow: {
        color: "#65a30d",
        fontWeight: "bold",
    },
    footer: {
        position: "absolute",
        bottom: 24,
        left: 40,
        right: 40,
        flexDirection: "row",
        justifyContent: "space-between",
        alignItems: "center",
        borderTopWidth: 1,
        borderTopColor: "#b5bfce",
        paddingTop: 8,
    },
    footerText: {
        fontSize: 7,
        color: "#94a3b8",
    },
    footerDisclaimer: {
        fontSize: 7,
        color: "#64748b",
        fontStyle: "italic",
    },
});

export interface ExecutiveReportProps {
    workspace: Workspace;
    networkName: string;
    networkCidr: string;
    generatedAt: string;
    censorshipEnabled: boolean;
    devices: Device[];
    securityScore: number;
    findings: SecurityFinding[];
    speedTest?: SpeedTestResult | null;
    healthSample?: NetworkHealthSample | null;
    version?: string;
}

export function ExecutiveReportDocument({
    workspace,
    networkName,
    networkCidr,
    generatedAt,
    censorshipEnabled,
    devices,
    securityScore,
    findings,
    speedTest,
    healthSample,
    version = "1.0",
}: ExecutiveReportProps) {
    const displayedCidr = censorshipEnabled
        ? maskCidr(networkCidr)
        : networkCidr;
    const displayedNetworkName = censorshipEnabled
        ? maskHostname(networkName)
        : networkName;

    // Resumen de dispositivos
    const onlineCount = devices.filter((d) => d.is_online).length;
    const offlineCount = devices.length - onlineCount;
    const trustedCount = devices.filter((d) => d.is_trusted).length;
    const untrustedCount = devices.length - trustedCount;
    const openFindings = findings.filter((f) => f.status === "open");

    return (
        <Document
            title={`Informe Ejecutivo MyLAN - ${workspace.name}`}
            author="MyLAN Network Inspector"
            subject="Auditoría y Diagnóstico de Red Local"
        >
            {/* Página 1: Portada y Resumen Ejecutivo */}
            <Page size="A4" style={styles.page}>
                {/* Header */}
                <View style={styles.header}>
                    <View>
                        <Text style={styles.brandTitle}>MyLAN</Text>
                        <Text style={styles.subtitle}>
                            Informe Ejecutivo de Auditoría y Diagnóstico de Red
                        </Text>
                    </View>
                    <View style={styles.metaBlock}>
                        <Text style={styles.metaText}>
                            Workspace: {workspace.name}
                        </Text>
                        <Text style={styles.metaText}>
                            Red: {displayedNetworkName} ({displayedCidr})
                        </Text>
                        <Text style={styles.metaText}>
                            Emitido: {generatedAt}
                        </Text>
                        {censorshipEnabled && (
                            <Text style={styles.badgeCensorship}>
                                MODO CENSURA ACTIVO
                            </Text>
                        )}
                    </View>
                </View>

                {/* Resumen Ejecutivo KPI Cards */}
                <Text style={styles.sectionTitle}>1. Resumen Ejecutivo</Text>
                <View style={styles.kpiRow}>
                    <View style={styles.kpiCard}>
                        <Text style={styles.kpiLabel}>Total Dispositivos</Text>
                        <Text style={styles.kpiValue}>{devices.length}</Text>
                        <Text style={styles.kpiSub}>
                            {onlineCount} Online • {offlineCount} Offline
                        </Text>
                    </View>
                    <View style={styles.kpiCard}>
                        <Text style={styles.kpiLabel}>Confianza</Text>
                        <Text style={styles.kpiValue}>
                            {devices.length > 0
                                ? `${Math.round((trustedCount / devices.length) * 100)}%`
                                : "0%"}
                        </Text>
                        <Text style={styles.kpiSub}>
                            {trustedCount} Confiables • {untrustedCount}{" "}
                            Desconocidos
                        </Text>
                    </View>
                    <View style={styles.kpiCard}>
                        <Text style={styles.kpiLabel}>Score de Seguridad</Text>
                        <Text
                            style={[
                                styles.kpiValue,
                                securityScore >= 85
                                    ? { color: "#16a34a" }
                                    : securityScore >= 60
                                      ? { color: "#ea580c" }
                                      : { color: "#dc2626" },
                            ]}
                        >
                            {securityScore} / 100
                        </Text>
                        <Text style={styles.kpiSub}>
                            {openFindings.length} Riesgos Abiertos
                        </Text>
                    </View>
                    <View style={styles.kpiCard}>
                        <Text style={styles.kpiLabel}>Salud & Velocidad</Text>
                        <Text style={styles.kpiValue}>
                            {speedTest
                                ? `${Math.round(speedTest.download_mbps)} Mbps`
                                : healthSample?.rtt_ms
                                  ? `${healthSample.rtt_ms} ms RTT`
                                  : "N/A"}
                        </Text>
                        <Text style={styles.kpiSub}>
                            {speedTest
                                ? `Subida: ${Math.round(speedTest.upload_mbps)} Mbps`
                                : healthSample
                                  ? `Jitter: ${healthSample.jitter_ms.toFixed(1)} ms`
                                  : "Sin sondeo reciente"}
                        </Text>
                    </View>
                </View>

                {/* Sección 2: Matriz de Riesgos de Seguridad */}
                <Text style={styles.sectionTitle}>
                    2. Matriz de Riesgos de Seguridad ({openFindings.length}{" "}
                    abiertos)
                </Text>
                {openFindings.length === 0 ? (
                    <View
                        style={[
                            styles.kpiCard,
                            { padding: 12, marginBottom: 12 },
                        ]}
                    >
                        <Text
                            style={{
                                color: "#16a34a",
                                fontWeight: "bold",
                                fontSize: 9,
                            }}
                        >
                            ✓ No se detectaron vulnerabilidades activas ni puertos
                            críticos de alto riesgo en este espacio de trabajo.
                        </Text>
                    </View>
                ) : (
                    <View style={styles.table}>
                        <View style={styles.tableHeaderRow}>
                            <Text style={[styles.tableColHeader, { width: "15%" }]}>
                                Severidad
                            </Text>
                            <Text style={[styles.tableColHeader, { width: "22%" }]}>
                                Regla / Código
                            </Text>
                            <Text style={[styles.tableColHeader, { width: "35%" }]}>
                                Hallazgo & Evidencia
                            </Text>
                            <Text style={[styles.tableColHeader, { width: "28%" }]}>
                                Mitigación Sugerida
                            </Text>
                        </View>
                        {openFindings.slice(0, 5).map((f) => {
                            const sevStyle =
                                f.severity === "Critical"
                                    ? styles.severityCritical
                                    : f.severity === "High"
                                      ? styles.severityHigh
                                      : f.severity === "Medium"
                                        ? styles.severityMedium
                                        : styles.severityLow;
                            return (
                                <View key={f.id} style={styles.tableRow}>
                                    <Text style={[styles.tableCol, sevStyle, { width: "15%" }]}>
                                        {f.severity}
                                    </Text>
                                    <Text
                                        style={[
                                            styles.tableCol,
                                            { width: "22%", fontWeight: "bold" },
                                        ]}
                                    >
                                        {f.rule_id}
                                    </Text>
                                    <View style={{ width: "35%", paddingRight: 4 }}>
                                        <Text
                                            style={[
                                                styles.tableCol,
                                                { fontWeight: "bold" },
                                            ]}
                                        >
                                            {f.title}
                                        </Text>
                                        {f.evidence && (
                                            <Text
                                                style={[
                                                    styles.tableCol,
                                                    {
                                                        color: "#64748b",
                                                        fontSize: 7,
                                                        marginTop: 1,
                                                    },
                                                ]}
                                            >
                                                {censorshipEnabled
                                                    ? maskIp(f.evidence)
                                                    : f.evidence}
                                            </Text>
                                        )}
                                    </View>
                                    <Text
                                        style={[
                                            styles.tableCol,
                                            { width: "28%", color: "#475569" },
                                        ]}
                                    >
                                        {f.remediation_advice ??
                                            "Revisar configuración de red."}
                                    </Text>
                                </View>
                            );
                        })}
                    </View>
                )}

                {/* Footer fijo */}
                <View style={styles.footer} fixed>
                    <Text style={styles.footerDisclaimer}>
                        {censorshipEnabled
                            ? "Documento generado con Modo Censura activo para distribución y auditoría segura."
                            : "Uso confidencial. Auditoría interna defensiva MyLAN."}
                    </Text>
                    <Text
                        style={styles.footerText}
                        render={({ pageNumber, totalPages }) =>
                            `Página ${pageNumber} de ${totalPages} • MyLAN v${version}`
                        }
                    />
                </View>
            </Page>

            {/* Página 2: Inventario Técnico Detallado */}
            <Page size="A4" style={styles.page}>
                {/* Header secundario */}
                <View style={styles.header}>
                    <View>
                        <Text style={[styles.brandTitle, { fontSize: 14 }]}>
                            MyLAN — Inventario Técnico de Dispositivos
                        </Text>
                        <Text style={styles.subtitle}>
                            Workspace: {workspace.name} ({displayedCidr})
                        </Text>
                    </View>
                    <View style={styles.metaBlock}>
                        <Text style={styles.metaText}>
                            Total: {devices.length} hosts
                        </Text>
                    </View>
                </View>

                <Text style={styles.sectionTitle}>
                    3. Catálogo de Dispositivos y Certeza
                </Text>

                <View style={styles.table}>
                    <View style={styles.tableHeaderRow}>
                        <Text style={[styles.tableColHeader, { width: "20%" }]}>
                            Dirección IP
                        </Text>
                        <Text style={[styles.tableColHeader, { width: "20%" }]}>
                            Dirección MAC
                        </Text>
                        <Text style={[styles.tableColHeader, { width: "25%" }]}>
                            Fabricante / Nombre
                        </Text>
                        <Text style={[styles.tableColHeader, { width: "20%" }]}>
                            Tipo & Certeza
                        </Text>
                        <Text style={[styles.tableColHeader, { width: "15%" }]}>
                            Estado
                        </Text>
                    </View>

                    {devices.map((d) => {
                        const rawIp = d.primary_ip ?? "N/A";
                        const rawMac = d.primary_mac ?? "N/A";
                        const ipText = censorshipEnabled
                            ? maskIp(rawIp)
                            : rawIp;
                        const macText = censorshipEnabled
                            ? maskMac()
                            : rawMac;
                        const nameText =
                            d.display_name ?? d.hostname ?? d.vendor ?? "Dispositivo";

                        return (
                            <View key={d.id} style={styles.tableRow}>
                                <Text
                                    style={[
                                        styles.tableCol,
                                        { width: "20%", fontWeight: "bold" },
                                    ]}
                                >
                                    {ipText}
                                </Text>
                                <Text
                                    style={[
                                        styles.tableCol,
                                        {
                                            width: "20%",
                                            fontFamily: "Courier",
                                            fontSize: 7,
                                        },
                                    ]}
                                >
                                    {macText}
                                </Text>
                                <View style={{ width: "25%", paddingRight: 2 }}>
                                    <Text
                                        style={[
                                            styles.tableCol,
                                            { fontWeight: "bold" },
                                        ]}
                                    >
                                        {censorshipEnabled
                                            ? maskHostname(nameText)
                                            : nameText}
                                    </Text>
                                    {d.vendor && (
                                        <Text
                                            style={{
                                                fontSize: 7,
                                                color: "#64748b",
                                            }}
                                        >
                                            {d.vendor}
                                        </Text>
                                    )}
                                </View>
                                <Text style={[styles.tableCol, { width: "20%" }]}>
                                    {d.device_type} ({d.confidence}%)
                                </Text>
                                <Text
                                    style={[
                                        styles.tableCol,
                                        {
                                            width: "15%",
                                            color: d.is_online
                                                ? "#16a34a"
                                                : "#94a3b8",
                                            fontWeight: "bold",
                                        },
                                    ]}
                                >
                                    {d.is_online ? "Online" : "Offline"}
                                    {d.is_trusted ? " • Confiable" : ""}
                                </Text>
                            </View>
                        );
                    })}
                </View>

                {/* Footer fijo */}
                <View style={styles.footer} fixed>
                    <Text style={styles.footerDisclaimer}>
                        {censorshipEnabled
                            ? "Documento generado con Modo Censura activo para distribución y auditoría segura."
                            : "Uso confidencial. Auditoría interna defensiva MyLAN."}
                    </Text>
                    <Text
                        style={styles.footerText}
                        render={({ pageNumber, totalPages }) =>
                            `Página ${pageNumber} de ${totalPages} • MyLAN v${version}`
                        }
                    />
                </View>
            </Page>
        </Document>
    );
}

/** Genera el blob PDF del informe ejecutivo. */
export async function generateExecutiveReportBlob(
    props: ExecutiveReportProps,
): Promise<Blob> {
    return await pdf(<ExecutiveReportDocument {...props} />).toBlob();
}

/** Descarga el informe ejecutivo en el navegador/desktop. */
export async function downloadExecutiveReport(
    props: ExecutiveReportProps,
    fileName?: string,
): Promise<void> {
    const blob = await generateExecutiveReportBlob(props);
    const url = URL.createObjectURL(blob);
    const name =
        fileName ??
        `mylan-reporte-${props.workspace.name.toLowerCase().replace(/\s+/g, "-")}-${new Date().toISOString().slice(0, 10)}.pdf`;

    const a = document.createElement("a");
    a.href = url;
    a.download = name;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
}
