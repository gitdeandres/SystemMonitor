# System Monitor

Una aplicación de escritorio desarrollada con Tauri que se ejecuta en segundo plano para recopilar información del sistema y transmitirla automáticamente a una API externa. Diseñada para entornos empresariales y gestión de activos IT.

## 🎯 Objetivo

Sistema de monitoreo silencioso que recopila y envía información crítica del sistema una vez al día:
- Información del sistema operativo (nombre y versión)
- Hostname de la máquina
- Número de serie del hardware
- Estado de activación de Windows
- Verificación de conectividad a internet

## 🏗️ Arquitectura Técnica

### Stack Tecnológico
- **Backend**: Rust + Tauri v2
- **Frontend**: React 19.1 + TypeScript 5 + Vite 7.0
- **Gestor de paquetes**: pnpm v10.12
- **Sistema de build**: Tauri CLI
- **Logging**: tauri-plugin-log con rotación automática

### Componentes Principales

```
├── src/                          # Frontend (React/TypeScript)
│   ├── system-monitor.ts        # Lógica principal y único de monitoreo
│   ├── main.tsx                 # Entrypoint mínimo
├── src-tauri/                   # Backend (Rust)
│   ├── src/
│   │   └── lib.rs              # Comandos Rust y lógica del sistema
│   ├── Cargo.toml              # Dependencias Rust
│   └── tauri.conf.json         # Configuración de Tauri
└── .env                        # Variables de entorno
```

## 🚀 Funcionalidades

### Características Principales
- ✅ **Ejecución en background**: Sin interfaz visible al usuario
- ✅ **Recolección automática**: Datos del sistema una vez al día
- ✅ **Verificación de conectividad**: Ping a múltiples servidores DNS
- ✅ **Logging detallado**: Archivos .log con rotación automática (5MB)
- ✅ **Reintentos inteligentes**: 3 intentos con delay configurable
- ✅ **Multiplataforma**: Windows (principal), Linux
- ✅ **WebView2 embebido**: No requiere instalación previa

### Datos Recopilados
- **Sistema operativo**: Nombre y versión
- **Identificación**: Hostname y número de serie
- **Licenciamiento**: Estado de activación de Windows
- **Conectividad**: Verificación previa de internet
- **Metadatos**: Timestamp y versión del cliente

## 📋 Requisitos del Sistema

### Desarrollo
- **Sistema operativo**: Ubuntu 25.04+ (desarrollo), Windows 10+ (testing)
- **Node.js**: 18+ 
- **pnpm**: 10.12+
- **Rust**: 1.60+ con toolchain stable
- **Tauri CLI**: 2.0+

### Producción (Windows)
- **Windows**: 10/11 (x64)
- **WebView2**: Se instala automáticamente
- **Permisos**: Ejecución de PowerShell y comandos de sistema

## 🛠️ Configuración e Instalación

### 1. Clonar el repositorio
```bash
git clone <repository-url>
cd system-monitor
```

### 2. Instalar dependencias
```bash
# Dependencias frontend
pnpm install

# Dependencias Rust (automático con Tauri)
cd src-tauri
cargo check
```

### 3. Configurar variables de entorno
```bash
# Crear archivo .env en la raíz
cp .env.example .env
```

Editar `.env`:
```env
VITE_API_ENDPOINT=https://tu-api.com/api/system-data
VITE_API_TOKEN=tu-token-secreto
```

### 4. Generar íconos (opcional)
```bash
pnpm tauri icon path/to/icon.png
```

## 🧪 Desarrollo

### Comandos principales
```bash
# Ejecutar en modo desarrollo
pnpm tauri dev

# Build para producción (Linux)
pnpm tauri build

# Build para Windows desde Linux
pnpm tauri build --target x86_64-pc-windows-gnu

# Limpiar cache de compilación
cd src-tauri && cargo clean

# Verificar sintaxis Rust sin compilar
cd src-tauri && cargo check
```

## 📦 Despliegue

### Build para Windows
```bash
# Desde Linux (recomendado para cross-compilation)
sudo apt install nsis
rustup target add x86_64-pc-windows-gnu
pnpm tauri build --target x86_64-pc-windows-gnu
```

### Artefactos generados
```
src-tauri/target/x86_64-pc-windows-gnu/release/bundle/
├── nsis/
│   └── system-monitor_0.1.0_x64-setup.exe      # Instalador NSIS
└── system-monitor.exe                           # Ejecutable portable
```

### Instalación en Windows
1. **Recomendado**: Usar el instalador NSIS
2. **Alternativo**: Ejecutable portable `.exe`

## ⚙️ Configuración

### Variables de entorno

| Variable | Descripción | Valor por defecto |
|----------|-------------|------------------|
| `VITE_API_ENDPOINT` | URL del endpoint de la API | `https://httpbin.org/post` |
| `VITE_API_TOKEN` | Token de autenticación Bearer | `""` |

### Configuración de logging (src-tauri/src/lib.rs)
```rust
.rotation_strategy(RotationStrategy::KeepOne)     // Mantener solo uno
.max_file_size(5_000_000)                        // 5 MB por archivo
```