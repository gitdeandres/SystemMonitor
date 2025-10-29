#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use serde::Serialize;
use sysinfo::{System, SystemExt};
use std::process::{Command, Stdio};
use tauri_plugin_log::{Target, TargetKind, RotationStrategy};
use log::{info, error, warn, debug};
use std::path::PathBuf;
use tauri_plugin_http::reqwest;

#[derive(Serialize)]
struct BasicSystemInfo {
    os_name: String,
    os_version: String,
    hostname: String,
}

#[derive(Serialize)]
struct WindowsSpecificInfo {
    serial_number: String,
    activation_status: String,
}

// Comando original de ejemplo (puedes mantenerlo o eliminarlo)
#[tauri::command]
fn greet(name: &str) -> String {
    info!("Greet command called with name: {}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Comando para obtener información básica del sistema
#[tauri::command]
async fn get_basic_system_info() -> Result<BasicSystemInfo, String> {
    info!("🔍 Iniciando recolección de información básica del sistema");

    let mut sys = System::new_all();
    sys.refresh_all();

    let os_name = sys.name().unwrap_or_else(|| {
        warn!("No se pudo determinar el nombre del sistema operativo");
        "Unknown".to_string()
    });
    
    let os_version = sys.os_version().unwrap_or_else(|| {
        warn!("No se pudo determinar la versión del sistema operativo");
        "Unknown".to_string()
    });
    
    // Hostname desde variable de entorno
    // Hostname consultando al sistema
    let hostname = get_hostname().await.unwrap_or_else(|_| {
        warn!("No se pudo obtener el hostname del sistema, usando 'Desconocido'");
        "Desconocido".to_string()
    });

    let result = BasicSystemInfo {
        os_name: os_name.clone(),
        os_version: os_version.clone(),
        hostname: hostname.clone(),
    };

    info!("✅ Información básica del sistema recopilada correctamente");

    Ok(result)
}

// Comando SOLO para lo que TypeScript no puede hacer
#[tauri::command]
async fn get_windows_specific_info() -> Result<WindowsSpecificInfo, String> {
    info!("🔍 Iniciar la recopilación de información específica de Windows");

    debug!("Recopilando número de serie...");
    let serial_number = match get_serial_number().await {
        Ok(serial) => {
            info!("✅ Número de serie obtenido satisfactoriamente");
            serial
        },
        Err(e) => {
            error!("❌ No se pudo obtener el número de serie: {}", e);
            "Unknown".to_string()
        }
    };
    
    debug!("Recopilando estado de activación del SO...");
    let activation_status = match get_activation_status().await {
        Ok(status) => {
            info!("✅ Estado de activación obtenido satisfactoriamente");
            status
        },
        Err(e) => {
            error!("❌ No se pudo obtener el estado de activación: {}", e);
            "Unknown".to_string()
        }
    };

    let result = WindowsSpecificInfo {
        serial_number,
        activation_status,
    };

    info!("✅ Se completó la recopilación de información específica de Windows");
    Ok(result)
}

// Comando para verificar conectividad a internet
#[tauri::command]
async fn check_internet_connectivity() -> Result<bool, String> {
    info!("🌐 Iniciando la verificación de conectividad a Internet");
    
    // Lista de servidores confiables para verificar conectividad
    let test_hosts = vec![
        "8.8.8.8",      // Google DNS
        "1.1.1.1",      // Cloudflare DNS  
        "208.67.222.222" // OpenDNS
    ];

    debug!("Probando la conectividad contra {} hosts", test_hosts.len());
    
    for host in &test_hosts {
        info!("Verficando conectividad a {}", host);
        if ping_host(host).await {
            info!("Conexión satisfactoria a {}", host);
            return Ok(true);
        }else {
            debug!("❌ El host {} no es accesible", host);
        }
    }
    
    warn!("❌ No se detectó conectividad a internet - todos los {} hosts fallaron", test_hosts.len());
    Ok(false)
}

// Comando para enviar datos a una API externa
#[tauri::command]
async fn send_to_api(endpoint: String, payload: String, token: Option<String>) -> Result<String, String> {
    info!("📡 Enviando datos a API: {}", endpoint);
    
    let client = reqwest::Client::new();
    let mut request = client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .header("User-Agent", "SystemMonitor/1.0");
    
    // Agregar token si está presente
    if let Some(auth_token) = token {
        if !auth_token.is_empty() {
            request = request.header("Authorization", format!("Bearer {}", auth_token));
            debug!("Token de autorización agregado al request");
        }
    }
    
    match request.body(payload).send().await {
        Ok(response) => {
            let status = response.status();
            debug!("Respuesta HTTP: {}", status);
            
            if status.is_success() {
                match response.text().await {
                    Ok(body) => {
                        info!("✅ Datos enviados exitosamente a la API");
                        Ok(body)
                    },
                    Err(e) => {
                        error!("Error leyendo respuesta: {}", e);
                        Err(format!("Error leyendo respuesta: {}", e))
                    }
                }
            } else {
                let error_msg = format!("HTTP {}: {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown"));
                error!("Error HTTP: {}", error_msg);
                Err(error_msg)
            }
        },
        Err(e) => {
            error!("Error en petición HTTP: {}", e);
            Err(format!("Error en petición: {}", e))
        }
    }
}

async fn get_serial_number() -> Result<String, Box<dyn std::error::Error>> {
    debug!("Ejecutando comando PowerShell para obtener número de serie del BIOS");

    // Método 1: Intentar con Win32_BIOS
    let methods = vec![
        ("Win32_BIOS", "Get-WmiObject -Class Win32_BIOS | Select-Object -ExpandProperty SerialNumber"),
        ("Win32_ComputerSystemProduct", "Get-WmiObject -Class Win32_ComputerSystemProduct | Select-Object -ExpandProperty IdentifyingNumber"),
        ("Win32_SystemEnclosure", "Get-WmiObject -Class Win32_SystemEnclosure | Select-Object -ExpandProperty SerialNumber"),
        ("CIM", "(Get-CimInstance -ClassName Win32_BIOS).SerialNumber"),
        ("WMIC", "wmic bios get serialnumber /value"),
    ];

    for (method_name, command) in methods {
        debug!("Intentando método {}: {}", method_name, command);
        
        let output = if method_name == "WMIC" {
            // WMIC comando directo
            #[cfg(target_os = "windows")]
            Command::new("wmic")
                .args(["bios", "get", "serialnumber", "/value"])
                .creation_flags(0x08000000)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .stdin(Stdio::null())
                .output()
        } else {
            // PowerShell commands
            #[cfg(target_os = "windows")]
            Command::new("powershell")
                .args(["-Command", command])
                .creation_flags(0x08000000)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .stdin(Stdio::null())
                .output()
        };

        #[cfg(not(target_os = "windows"))]
        let output = Command::new("powershell")
            .args(["-Command", command])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .output();

        match output {
            Ok(result) => {
                debug!("Método {} - Estado: {:?}", method_name, result.status);
                
                if result.status.success() {
                    let stdout = String::from_utf8_lossy(&result.stdout);
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    
                    debug!("Método {} - Stdout: '{}'", method_name, stdout);
                    if !stderr.is_empty() {
                        debug!("Método {} - Stderr: '{}'", method_name, stderr);
                    }
                    
                    // Procesar diferentes formatos de salida
                    let serial = if method_name == "WMIC" {
                        // WMIC retorna: SerialNumber=ABCD1234
                        stdout.lines()
                            .find(|line| line.starts_with("SerialNumber="))
                            .and_then(|line| line.split('=').nth(1))
                            .unwrap_or("")
                            .trim()
                            .to_string()
                    } else {
                        stdout.trim().to_string()
                    };
                    
                    debug!("Método {} - Serial procesado: '{}'", method_name, serial);
                    
                    if !serial.is_empty() && serial != "0" && serial.to_lowercase() != "to be filled by o.e.m." {
                        info!("✅ Número de serie obtenido via {}: {}", method_name, serial);
                        return Ok(serial);
                    } else {
                        debug!("Método {} - Serial inválido o vacío: '{}'", method_name, serial);
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    debug!("Método {} falló - Stderr: '{}'", method_name, stderr);
                }
            },
            Err(e) => {
                debug!("Error ejecutando método {}: {}", method_name, e);
            }
        }
    }
    
    warn!("❌ No se pudo obtener número de serie con ningún método");
    Ok("Desconocido".to_string())
}

async fn get_activation_status() -> Result<String, Box<dyn std::error::Error>> {
    debug!("Ejecución del comando de PowerShell para recuperar el estado de activación de Windows");

    #[cfg(target_os = "windows")]
    let output = Command::new("powershell")
        .args(["-Command", "Get-WmiObject -Class SoftwareLicensingProduct | Where-Object {$_.PartialProductKey -and $_.Name -like '*Windows*'} | Select-Object -ExpandProperty LicenseStatus"])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .output()?;

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("powershell")
        .args(["-Command", "Get-WmiObject -Class SoftwareLicensingProduct | Where-Object {$_.PartialProductKey} | Select-Object -ExpandProperty LicenseStatus"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .output()?;

    if output.status.success() {
        let status_code = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let status_text = match status_code.as_str() {
            "0" => "Unlicensed",
            "1" => "Licensed", 
            "2" => "OOBGrace",
            "3" => "OOTGrace",
            "4" => "NonGenuineGrace",
            "5" => "Notification",
            "6" => "ExtendedGrace",
            _ => "Unknown",
        };
        debug!("Valid serial number retrieved from BIOS");
        return Ok(status_text.to_string());
    }

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let status_code = output_str.as_str();
        
        debug!("Código de estado de activación sin procesar: '{}'", status_code);
        
        let status_text = match status_code {
            "0" => "Unlicensed",
            "1" => "Licensed", 
            "2" => "OOBGrace",
            "3" => "OOTGrace",
            "4" => "NonGenuineGrace",
            "5" => "Notification",
            "6" => "ExtendedGrace",
            _ => {
                debug!("Código de estado de activación desconocido: '{}'", status_code);
                "Unknown"
            },
        };
        
        debug!("Estado de activación interpretado como: {}", status_text);
        return Ok(status_text.to_string());
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        debug!("El comando de activación de PowerShell falló con stderr: {}", stderr);
    }

    Ok("Unknown".to_string())
}

async fn get_hostname() -> Result<String, Box<dyn std::error::Error>> {
    debug!("Ejecutando comando para obtener hostname del sistema");
    
    // Intentar primero con hostname
    #[cfg(target_os = "windows")]
    let output = Command::new("hostname")
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .output()?;

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("hostname")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .output()?;

    if output.status.success() {
        let hostname = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !hostname.is_empty() {
            debug!("Hostname obtenido via comando hostname: {}", hostname);
            return Ok(hostname);
        }
    }

    // Fallback a PowerShell
    #[cfg(target_os = "windows")]
    let output = Command::new("powershell")
        .args(["-Command", "$env:COMPUTERNAME"])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .output()?;

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("powershell")
        .args(["-Command", "$env:COMPUTERNAME"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .output()?;

    if output.status.success() {
        let hostname = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !hostname.is_empty() {
            debug!("Hostname obtenido via PowerShell: {}", hostname);
            return Ok(hostname);
        }
    }

    Err("No se pudo obtener hostname por ningún método".into())
}

async fn ping_host(host: &str) -> bool {
    debug!("Ejecutando ping al host: {}", host);

    let output = if cfg!(target_os = "windows") {
        Command::new("ping")
            .args(["-n", "1", "-w", "3000", host])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .output()
    } else {
        Command::new("ping")
            .args(["-c", "1", "-W", "3", host])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .output()
    };
    
    match output {
        Ok(result) => {
            let success = result.status.success();
            if success {
                debug!("✅ Ping a {} satisfactorio", host);
            } else {
                debug!("❌ Ping a {} fallido (código de salida: {:?})", host, result.status.code());
                
                // Log stderr for debugging if available
                if !result.stderr.is_empty() {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    debug!("Ping stderr: {}", stderr);
                }
            }
            success
        },
        Err(e) => {
            debug!("❌ Error al ejecutar el comando ping a {}: {}", host, e);
            false
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
            .rotation_strategy(RotationStrategy::KeepOne)
            .max_file_size(5_000_000) // 5 MB
            .targets([
                Target::new(TargetKind::Folder {
                  path:  PathBuf::from("logs"),
                  file_name: Some("system-monitor".to_string())
                }),
                Target::new(TargetKind::Stdout),
            ])
            .build(),
        )
        .invoke_handler(tauri::generate_handler![
            greet,
            get_basic_system_info,
            get_windows_specific_info,
            check_internet_connectivity,
            send_to_api
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}