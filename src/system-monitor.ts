import { invoke } from '@tauri-apps/api/core'
import { 
  info as logInfo, 
  warn as logWarn, 
  error as logError, 
  debug as logDebug 
} from '@tauri-apps/plugin-log'

interface BasicSystemInfo {
  os_name: string;
  os_version: string;
  hostname: string;
}

interface WindowsSpecificInfo {
  serial_number: string;
  activation_status: string;
}

interface CompleteSystemData {
  os_name: string;
  os_version: string;
  hostname: string;
  serial_number: string;
  activation_status: string;
  timestamp: string;
}

interface ApiConfig {
  endpoint: string;
  token: string;
  timeout: number;
  maxRetries: number;
  retryDelay: number;
}

class SystemMonitor {
  private config: ApiConfig;
  private intervalId?: number;
  private isRunning = false;

  constructor() {
    this.config = {
      endpoint: import.meta.env.VITE_API_ENDPOINT,
      token: import.meta.env.VITE_API_TOKEN,
      timeout: 30000,
      maxRetries: 3,
      retryDelay: 5000
    };

    logInfo('[SystemMonitor] Instancia de SystemMonitor creada');
    logDebug(`[SystemMonitor] Configuración inicial: endpoint=${this.config.endpoint}, timeout=${this.config.timeout}ms`);
    
    this.startDailyScheduler();
  }

  // Recolectar toda la información del sistema
  async collectSystemData(): Promise<CompleteSystemData> {
    await logInfo('[SystemMonitor] 🔍 Iniciando recolección completa de datos del sistema');
    
    try {
      // Información básica (podría ser desde TS, pero usamos Rust por consistencia)
      const basicInfo = await invoke<BasicSystemInfo>('get_basic_system_info');
      await logInfo('[SystemMonitor] ✅ Información básica del sistema recolectada');

      // Información específica de Windows (SOLO desde Rust)
      const windowsInfo = await invoke<WindowsSpecificInfo>('get_windows_specific_info');
      await logInfo('[SystemMonitor] ✅ Información específica de Windows recolectada');

      const timestamp = new Date().toISOString();

      const completeData: CompleteSystemData = {
        ...basicInfo,
        ...windowsInfo,
        timestamp
      };

      await logInfo('[SystemMonitor] ✅ Recolección completa de datos del sistema completada exitosamente');

      return completeData;

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      await logError(`[SystemMonitor] ❌ Error en recolección de datos del sistema: ${errorMessage}`);
      throw error;
    }
  }

  // Proceso completo de envío con logging paso a paso
  async sendSystemData(): Promise<void> {
    await logInfo('[SystemMonitor] 🚀 Iniciando transmisión completa de datos');
    
    try {
      // Verificar conectividad
      await logDebug('[SystemMonitor] Verificando conectividad a internet');
      const hasInternet = await invoke<boolean>('check_internet_connectivity');
      
      if (!hasInternet) {
        throw new Error('No se puede proceder: Sin conectividad a internet');
      }
      
      // Recolectar datos
      await logDebug('[SystemMonitor] Recolectando datos del sistema');
      const systemData = await this.collectSystemData();
      
      // Enviar datos
      await logDebug('[SystemMonitor] Enviando datos a la API');
      await this.sendToAPI(systemData);
      
      // Actualizar registro de último envío
      await logDebug('[SystemMonitor] Actualizando timestamp de último envío');
      this.updateLastSent();
      
      await logInfo(`[SystemMonitor] ✅ Transmisión completa de datos completada exitosamente`);
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      await logError(`[SystemMonitor] ❌ Transmisión de datos falló: ${errorMessage}`);
      throw error;
    }
  }

  // Enviar datos a la API externa
  async sendToAPI(data: CompleteSystemData): Promise<void> {
    await logInfo('[ApiClient] 📡 Iniciando petición POST a API');
    
    await logDebug('[ApiClient] Preparando payload de la petición');
    const payload = {
      system_info: {
        os_name: data.os_name,
        os_version: data.os_version,
        hostname: data.hostname,
        serial_number: data.serial_number,
        activation_status: data.activation_status,
        timestamp: data.timestamp
      },
      client_version: '1.0.0',
      report_type: 'daily_system_check'
    };
    const payloadString = JSON.stringify(payload);

    await logDebug(`[ApiClient] Payload preparado con ${Object.keys(data).length} campos de datos`);

    let lastError: Error | null = null;
    
    for (let attempt = 1; attempt <= this.config.maxRetries; attempt++) {
      const attemptStartTime = Date.now();
      await logInfo(`[ApiClient] Intento ${attempt}/${this.config.maxRetries} de envío a API`);

      try {        
        const controller = new AbortController();
        const timeoutId = setTimeout(() => {
            logWarn(`[ApiClient] Timeout de ${this.config.timeout}ms alcanzado, abortando petición`);
            controller.abort();
          }, this.config.timeout);

        const response = await invoke<string>('send_to_api', {
          endpoint: this.config.endpoint,
          payload: payloadString,
          token: this.config.token
        });
        
        clearTimeout(timeoutId);

        const attemptDuration = Date.now() - attemptStartTime;
        await logDebug(`[ApiClient] Respuesta recibida exitosamente (${attemptDuration}ms)`);
        await logDebug(`[ApiClient] Cuerpo de respuesta: ${response.substring(0, 200)}...`);
        
        await logInfo(`[ApiClient] ✅ Datos enviados exitosamente en intento ${attempt}`);
        return;
        
      } catch (error) {
        lastError = error as Error;
        const attemptDuration = Date.now() - attemptStartTime;
        const errorMessage = error instanceof Error ? error.message : String(error);
        await logWarn(`[ApiClient] ⚠️ Intento ${attempt} falló: ${errorMessage} (${attemptDuration}ms)`);
        
        if (attempt < this.config.maxRetries) {
          await logInfo(`[ApiClient] ⏳ Reintentando en ${this.config.retryDelay}ms...`);
          await this.sleep(this.config.retryDelay);
        }
      }
    }
    
    const finalError = `Todos los ${this.config.maxRetries} intentos fallaron. Último error: ${lastError?.message}`;
    await logError(`[ApiClient] ❌ ${finalError}`);
    throw new Error(finalError);
  }

  // Scheduler diario (ejecutar una vez al día)
  startDailyScheduler(): void {
    if (this.isRunning) {
      logWarn('[Scheduler] ⚠️ Scheduler ya está ejecutándose, ignorando inicio duplicado');
      return;
    }

    logInfo('[Scheduler] 📅 Iniciando scheduler diario de transmisión de datos');
    this.isRunning = true;

    // Verificar si ya se envió hoy
    const checkAndSend = async () => {
      try {
        logDebug('[Scheduler] Verificando si hay internet...');
        const hasInternet = await invoke<boolean>('check_internet_connectivity');
        if (!hasInternet){
          logError('❌ Sin conexión a internet. Se omite el envío de datos.');
          return;
        }
        const lastSent = this.getLastSent();
        const now = new Date();
        const today = now.toDateString();
        
        if (!lastSent) {
          await logInfo('[Scheduler] 📅 Primer envío - no hay registro de envío anterior');
          await this.sendDailyData();
        } else {
          const lastSentDate = new Date(lastSent).toDateString();
          if (lastSentDate !== today) {
            await logInfo(`[Scheduler] 📅 Es momento de enviar datos diarios (último envío: ${lastSentDate})`);
            await this.sendDailyData();
          } else {
            await logDebug(`[Scheduler] Datos ya enviados hoy (${lastSentDate}), saltando envío`);
          }
        }
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        await logError(`[Scheduler] ❌ Error en verificación del scheduler: ${errorMessage}`);
      }
    };

    // Ejecutar inmediatamente
    logDebug('[Scheduler] Ejecutando verificación inicial');
    checkAndSend();
    
    // Luego cada hora verificar
    this.intervalId = window.setInterval(() => {
      logDebug('[Scheduler] Ejecutando verificación horaria');
      checkAndSend();
    }, 1 * 60 * 1000); // 1 hora
    
    logInfo('[Scheduler] ✅ Scheduler diario configurado (verificación cada hora)');
  }

  stopScheduler(): void {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = undefined;
    }
    this.isRunning = false;
    logInfo('[Scheduler] 🛑 Scheduler detenido');
  }

  // Envío diario con manejo de errores
  async sendDailyData(): Promise<void> {
    const startTime = Date.now();
    await logInfo('[Scheduler] 📅 Iniciando transmisión diaria programada de datos');
    
    try {
      await this.sendSystemData();
      
      const duration = Date.now() - startTime;
      await logInfo(`[Scheduler] ✅ Transmisión diaria completada exitosamente (${duration}ms)`);
      
    } catch (error) {
      const duration = Date.now() - startTime;
      const errorMessage = error instanceof Error ? error.message : String(error);
      await logError(`[Scheduler] ❌ Transmisión diaria falló: ${errorMessage} (${duration}ms)`);
      
      // No re-lanzar error para que el scheduler continue funcionando
      await logInfo('[Scheduler] Scheduler continuará funcionando para próximos intentos');
    }
  }

  // Envío manual (para testing o forzar envío)
  async sendNow(): Promise<void> {
    await logInfo('[SystemMonitor] 🚀 Envío manual solicitado por usuario');
    await this.sendDailyData();
  }

  // Storage helpers con logging
  private updateLastSent(): void {
    const timestamp = new Date().toISOString();
    localStorage.setItem('lastSent', timestamp);
    logDebug(`[Storage] 📝 Timestamp de último envío actualizado: ${timestamp}`);
  }

  private getLastSent(): string | null {
    const lastSent = localStorage.getItem('lastSent');
    if (lastSent) {
      logDebug(`[Storage] 📝 Último envío encontrado: ${lastSent}`);
    } else {
      logDebug('[Storage] 📝 No hay registro de envío anterior');
    }
    return lastSent;
  }

  // Utilities
  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

// Singleton instance
const systemMonitor = new SystemMonitor();

// Export para uso desde otros módulos
export default systemMonitor;