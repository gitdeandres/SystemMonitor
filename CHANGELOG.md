# Changelog

## [1.1.0] - 2024-10-29

### Added

- Filtrado específico para evitar confusión con licencias de Office

### Changed
- Versión del cliente ahora se obtiene dinámicamente desde package.json

### Fixed

- Estado de activación devolvía "Unknown" en algunos equipos Windows 10 Enterprise LTSC

## [1.0.0] - 2024-10-27

### Added

- **Sistema de monitoreo automatizado**: Recopilación y transmisión diaria de datos del sistema
- **Información básica del sistema**: Obtención de nombre y versión del SO, hostname
- **Información específica de Windows**: Número de serie del BIOS y estado de activación de Windows
- **Verificación de conectividad**: Ping automático a múltiples DNS confiables (Google, Cloudflare, OpenDNS)
- **Transmisión a API externa**: Envío seguro de datos con autenticación Bearer token
- **Scheduler inteligente**: Envío automático diario con verificación horaria
- **Sistema de reintentos**: Hasta 3 intentos con delay configurable para robustez
- **Logging completo**: Sistema de logs detallado con rotación y múltiples niveles
- **Múltiples métodos de obtención de datos**: Fallbacks robustos para información del BIOS
- **Manejo de errores resiliente**: Continuidad del servicio ante fallos temporales
- **Configuración por variables de entorno**: Endpoint y token de API configurables
- **Almacenamiento local**: Tracking de último envío para evitar duplicados
