# Esquema de Workspaces y Generador de Informes PDF con Modo Censura

Type: grilling
Status: resolved
Blocked by: none

## Question

¿Cómo se modela la partición por Workspaces en el esquema relacional de SQLite (`mylan-db`) y cómo se estructura el generador de reportes ejecutivos en PDF garantizando el Modo Censura?

Se debe definir:
1. Migración de base de datos para Workspaces: tabla `workspaces` (`id`, `name`, `created_at`, `color`, `icon`, `is_active`) y clave foránea `workspace_id` en `networks`, `scans`, `devices` y `events`. Estrategia para migrar datos preexistentes a un "Workspace por defecto" sin romper compatibilidad brownfield.
2. Selección y conmutación de Workspace en la UI: selector en la barra superior o lateral, aislamiento de historial e inventario entre workspaces.
3. Generador de informes PDF:
   - ¿Dónde se compila el PDF? (Backend Rust con crates como `printpdf`/`genpdf` vs Frontend React con `@react-pdf/renderer` o canvas/impresión web).
   - Estructura del informe: carátula con identidad visual de MyLAN, resumen ejecutivo de inventario, tabla de dispositivos con tipos y certeza, semáforo de riesgos de seguridad, métricas de salud de red.
   - Respeto estricto del Modo Censura: si la opción de censura está activa, el PDF generado debe enmascarar automáticamente todas las IPs, MACs, SSIDs y hostnames.

## Answer

### 1. Migración Relacional para Workspaces en SQLite (`mylan-db`)
- **Nueva Tabla `workspaces`**:
  ```sql
  CREATE TABLE IF NOT EXISTS workspaces (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    icon TEXT NOT NULL DEFAULT 'network',
    color TEXT NOT NULL DEFAULT '#06b6d4',
    created_at TEXT NOT NULL
  );
  ```
- **Claves Foráneas y Migración Brownfield**:
  - Se añade la columna `workspace_id TEXT REFERENCES workspaces(id)` a las tablas `networks`, `scans`, `devices`, `events`, `security_findings` y `network_health_samples`.
  - Para migraciones desde versiones anteriores sin romper compatibilidad:
    ```sql
    INSERT OR IGNORE INTO workspaces (id, name, description, icon, color, created_at)
    VALUES ('default', 'Mi Red Local', 'Workspace principal por defecto', 'home', '#06b6d4', datetime('now'));

    UPDATE networks SET workspace_id = 'default' WHERE workspace_id IS NULL;
    UPDATE scans SET workspace_id = 'default' WHERE workspace_id IS NULL;
    UPDATE devices SET workspace_id = 'default' WHERE workspace_id IS NULL;
    UPDATE events SET workspace_id = 'default' WHERE workspace_id IS NULL;
    ```

### 2. Detección Automática y Conmutación en la Interfaz
- **Detección Automática por Red**:
  - Cada Workspace almacena las redes conocidas asociadas a él (mediante SSID o CIDR/Gateway).
  - Al iniciar o detectar una conexión, MyLAN identifica automáticamente si la red actual pertenece a un Workspace existente (ej. "Oficina" vs "Casa") y conmuta la vista activa sin fricción.
- **Selector de Workspace en la UI**:
  - Componente de selector desplegable en la barra de navegación superior izquierda con indicador de color e icono.
  - Permite crear nuevos Workspaces, renombrar, cambiar color/icono y alternar manualmente entre ellos.
  - Todas las consultas de inventario, dispositivos, historial y métricas filtran automáticamente por el `workspace_id` activo.

### 3. Motor de Compilación de Informes PDF en Frontend React
- **Tecnología**: Generación en cliente mediante **`@react-pdf/renderer`** (o plantilla web de alta fidelidad exportada a PDF).
  - Permite reutilizar la paleta de colores de Tailwind, tipografías y componentes visuales del sistema de diseño (badges de `Certeza`, `Confiable`, `Online` e isotipos hexagonales de la marca).
  - Cero dependencias nativas complejas en Rust y compilación instantánea sin consumo de recursos del servidor.

### 4. Estructura del Informe Ejecutivo en PDF
1. **Encabezado / Portada**: Logotipo oficial de MyLAN (`final-horizontal-color.svg` o monocromo), nombre del Workspace, fecha y hora de emisión, CIDR/SSID de la red activa e indicador visible de estado del Modo Censura.
2. **Resumen Ejecutivo**:
   - Total de dispositivos descubiertos (con desglose Online vs Offline y Confiables vs Desconocidos).
   - **Score de Seguridad de Red** (0 a 100) con semáforo y conteo de vulnerabilidades abiertas.
   - Métricas de salud de red (latencia RTT promedio al Gateway, servidores DNS activos y último speed test registrado).
3. **Matriz de Riesgos de Seguridad**:
   - Tabla con severidad (`Critical`, `High`, `Medium`), código de regla, dispositivo afectado, evidencia técnica y recomendación práctica de mitigación.
4. **Inventario Técnico Detallado**:
   - Tabla de dispositivos con IP, MAC, Fabricante/Vendor (OUI), Tipo de dispositivo con porcentaje de Certeza, etiqueta Confiable y catálogo de puertos/servicios abiertos identificados.
5. **Pie de Página**: Numeración de páginas ("Página X de Y"), versión de MyLAN y disclaimer legal/auditoría defensiva.

### 5. Respeto Estricto del Modo Censura en Exportaciones
- El modal de exportación a PDF incluye un toggle explícito: `Exportar con Modo Censura (enmascarar IPs, MACs, Hostnames y SSIDs)`.
- El toggle viene pre-marcado según el estado actual del Modo Censura en la aplicación.
- Si la censura está activada:
  - Todas las IPs se exportan con el octeto final enmascarado (ej. `192.168.1.***`).
  - Todas las MACs se enmascaran en sus bytes de dispositivo (ej. `a4:83:e7:**:**:**`).
  - Los SSIDs y hostnames sensibles se ofuscan.
  - Se añade un banner discreto en el pie del documento: `Documento generado con Modo Censura activo para distribución y auditoría segura`.
