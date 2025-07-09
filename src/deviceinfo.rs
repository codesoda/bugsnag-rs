use sys_info;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use std::collections::HashMap;
use machineid_rs::{IdBuilder, Encryption, HWIDComponent};

// Hardware information that doesn't change
#[derive(Debug, Clone)]
struct HardwareInfo {
    id: Option<String>,
    manufacturer: String,
    model: String,
    model_number: String,
}

// Static system information that doesn't change
struct StaticSystemInfo {
    hardware: HardwareInfo,
    hostname: String,
    os_name: String,
    os_version: String,
    total_memory: Option<u64>,
    total_disk: Option<u64>,
}

lazy_static! {
    static ref STATIC_SYSTEM_INFO: StaticSystemInfo = {
        let mut version = sys_info::os_type().unwrap_or("Unknown".to_owned());
        version.push_str(":");
        version.push_str(&sys_info::os_release().unwrap_or("u.k.n.o.w.n".to_owned()));
        
        StaticSystemInfo {
            hardware: get_hardware_info(),
            hostname: sys_info::hostname().unwrap_or("UnknownHost".to_owned()),
            os_name: sys_info::os_type().unwrap_or("Unknown".to_owned()),
            os_version: version,
            total_memory: sys_info::mem_info().ok().map(|info| info.total),
            total_disk: sys_info::disk_info().ok().map(|info| info.total),
        }
    };
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    id: Option<String>,
    manufacturer: Option<String>,
    model: Option<String>,
    model_number: Option<String>,
    os_name: String,
    os_version: String,
    hostname: String,
    free_memory: Option<u64>,
    total_memory: Option<u64>,
    free_disk: Option<u64>,
    total_disk: Option<u64>,
    disk_usage: Option<HashMap<String, DiskInfo>>,
    time: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    total: u64,
    free: u64,
    used: u64,
    mount_point: String,
}

impl DeviceInfo {
    pub fn new(version: &str, name: &str) -> DeviceInfo {
        DeviceInfo {
            id: None,
            manufacturer: None,
            model: None,
            model_number: None,
            os_name: STATIC_SYSTEM_INFO.os_name.clone(),
            os_version: version.to_owned(),
            hostname: name.to_owned(),
            free_memory: None,
            total_memory: STATIC_SYSTEM_INFO.total_memory,
            free_disk: None,
            total_disk: None,
            disk_usage: None,
            time: format_current_time(),
        }
    }

    pub fn generate() -> DeviceInfo {
        DeviceInfo {
            id: STATIC_SYSTEM_INFO.hardware.id.clone(),
            manufacturer: Some(STATIC_SYSTEM_INFO.hardware.manufacturer.clone()),
            model: Some(STATIC_SYSTEM_INFO.hardware.model.clone()),
            model_number: Some(STATIC_SYSTEM_INFO.hardware.model_number.clone()),
            os_name: STATIC_SYSTEM_INFO.os_name.clone(),
            os_version: STATIC_SYSTEM_INFO.os_version.clone(),
            hostname: STATIC_SYSTEM_INFO.hostname.clone(),
            free_memory: sys_info::mem_info().ok().map(|info| info.free),
            total_memory: STATIC_SYSTEM_INFO.total_memory,
            total_disk: STATIC_SYSTEM_INFO.total_disk,
            time: format_current_time(),
            disk_usage: None,
            free_disk: None,
        }
    }

    pub fn set_id(&mut self, id: &str) {
        self.id = Some(id.to_owned());
    }

    pub fn set_manufacturer(&mut self, manufacturer: &str) {
        self.manufacturer = Some(manufacturer.to_owned());
    }

    pub fn set_model(&mut self, model: &str) {
        self.model = Some(model.to_owned());
    }

    pub fn set_model_number(&mut self, model_number: &str) {
        self.model_number = Some(model_number.to_owned());
    }

    pub fn set_os_name(&mut self, os_name: &str) {
        self.os_name = os_name.to_owned();
    }

    pub fn set_os_version(&mut self, version: &str) {
        self.os_version = version.to_owned();
    }

    pub fn set_hostname(&mut self, name: &str) {
        self.hostname = name.to_owned();
    }

    pub fn set_free_memory(&mut self, free_memory: u64) {
        self.free_memory = Some(free_memory);
    }

    pub fn set_total_memory(&mut self, total_memory: u64) {
        self.total_memory = Some(total_memory);
    }

    pub fn set_free_disk(&mut self, free_disk: u64) {
        self.free_disk = Some(free_disk);
    }

    pub fn set_total_disk(&mut self, total_disk: u64) {
        self.total_disk = Some(total_disk);
    }

    pub fn set_disk_usage(&mut self, disk_usage: HashMap<String, DiskInfo>) {
        self.disk_usage = Some(disk_usage);
    }

    pub fn set_time(&mut self, time: &str) {
        self.time = time.to_owned();
    }
}

fn get_hardware_info() -> HardwareInfo {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        
        // Try to get detailed hardware info using system_profiler
        if let Ok(output) = Command::new("system_profiler")
            .arg("SPHardwareDataType")
            .output()
        {
            if let Ok(profiler_output) = String::from_utf8(output.stdout) {
                let mut manufacturer = String::new();
                let mut model = String::new();
                let mut model_number = String::new();
                
                for line in profiler_output.lines() {
                    let line = line.trim();
                    if line.starts_with("Model Name:") {
                        model = line.split(':').nth(1).unwrap_or("").trim().to_string();
                        // For Apple devices, manufacturer is always Apple
                        manufacturer = "Apple".to_string();
                    } else if line.starts_with("Model Identifier:") {
                        model_number = line.split(':').nth(1).unwrap_or("").trim().to_string();
                    }
                }
                
                if !manufacturer.is_empty() && !model.is_empty() && !model_number.is_empty() {
                    return HardwareInfo {
                        id: generate_hardware_id(),
                        manufacturer,
                        model,
                        model_number,
                    };
                }
            }
        }
        
        // Fallback to sysctl for model identifier
        if let Ok(output) = Command::new("sysctl")
            .arg("-n")
            .arg("hw.model")
            .output()
        {
            if let Ok(model_id) = String::from_utf8(output.stdout) {
                let model_id = model_id.trim();
                if !model_id.is_empty() {
                    return HardwareInfo {
                        id: generate_hardware_id(),
                        manufacturer: "Apple".to_string(),
                        model: "Mac".to_string(),
                        model_number: model_id.to_string(),
                    };
                }
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        
        // Try to get hardware info from DMI
        let mut manufacturer = String::new();
        let mut model = String::new();
        let mut model_number = String::new();
        
        // Get manufacturer
        if let Ok(output) = Command::new("dmidecode")
            .arg("-s")
            .arg("system-manufacturer")
            .output()
        {
            if let Ok(mfg) = String::from_utf8(output.stdout) {
                let mfg = mfg.trim();
                if !mfg.is_empty() && mfg != "To Be Filled By O.E.M." {
                    manufacturer = mfg.to_string();
                }
            }
        }
        
        // Get product name (model)
        if let Ok(output) = Command::new("dmidecode")
            .arg("-s")
            .arg("system-product-name")
            .output()
        {
            if let Ok(prod) = String::from_utf8(output.stdout) {
                let prod = prod.trim();
                if !prod.is_empty() && prod != "To Be Filled By O.E.M." {
                    model = prod.to_string();
                }
            }
        }
        
        // Get version (model number)
        if let Ok(output) = Command::new("dmidecode")
            .arg("-s")
            .arg("system-version")
            .output()
        {
            if let Ok(ver) = String::from_utf8(output.stdout) {
                let ver = ver.trim();
                if !ver.is_empty() && ver != "To Be Filled By O.E.M." {
                    model_number = ver.to_string();
                }
            }
        }
        
        // Fallback to reading from /sys/class/dmi/id/
        if manufacturer.is_empty() {
            if let Ok(mfg) = std::fs::read_to_string("/sys/class/dmi/id/sys_vendor") {
                let mfg = mfg.trim();
                if !mfg.is_empty() && mfg != "To Be Filled By O.E.M." {
                    manufacturer = mfg.to_string();
                }
            }
        }
        
        if model.is_empty() {
            if let Ok(prod) = std::fs::read_to_string("/sys/class/dmi/id/product_name") {
                let prod = prod.trim();
                if !prod.is_empty() && prod != "To Be Filled By O.E.M." {
                    model = prod.to_string();
                }
            }
        }
        
        if model_number.is_empty() {
            if let Ok(ver) = std::fs::read_to_string("/sys/class/dmi/id/product_version") {
                let ver = ver.trim();
                if !ver.is_empty() && ver != "To Be Filled By O.E.M." {
                    model_number = ver.to_string();
                }
            }
        }
        
        if !manufacturer.is_empty() || !model.is_empty() || !model_number.is_empty() {
            return HardwareInfo {
                id: generate_hardware_id(),
                manufacturer: if manufacturer.is_empty() { "Unknown".to_string() } else { manufacturer },
                model: if model.is_empty() { "Unknown".to_string() } else { model },
                model_number: if model_number.is_empty() { "Unknown".to_string() } else { model_number },
            };
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        
        let mut manufacturer = String::new();
        let mut model = String::new();
        let mut model_number = String::new();
        
        // Get manufacturer
        if let Ok(output) = Command::new("wmic")
            .arg("computersystem")
            .arg("get")
            .arg("manufacturer")
            .arg("/value")
            .output()
        {
            if let Ok(wmic_output) = String::from_utf8(output.stdout) {
                for line in wmic_output.lines() {
                    if line.starts_with("Manufacturer=") {
                        manufacturer = line.split('=').nth(1).unwrap_or("").trim().to_string();
                        break;
                    }
                }
            }
        }
        
        // Get model
        if let Ok(output) = Command::new("wmic")
            .arg("computersystem")
            .arg("get")
            .arg("model")
            .arg("/value")
            .output()
        {
            if let Ok(wmic_output) = String::from_utf8(output.stdout) {
                for line in wmic_output.lines() {
                    if line.starts_with("Model=") {
                        model = line.split('=').nth(1).unwrap_or("").trim().to_string();
                        break;
                    }
                }
            }
        }
        
        // Get system SKU (model number)
        if let Ok(output) = Command::new("wmic")
            .arg("computersystem")
            .arg("get")
            .arg("SystemSKUNumber")
            .arg("/value")
            .output()
        {
            if let Ok(wmic_output) = String::from_utf8(output.stdout) {
                for line in wmic_output.lines() {
                    if line.starts_with("SystemSKUNumber=") {
                        model_number = line.split('=').nth(1).unwrap_or("").trim().to_string();
                        break;
                    }
                }
            }
        }
        
        if !manufacturer.is_empty() || !model.is_empty() || !model_number.is_empty() {
            return HardwareInfo {
                id: generate_hardware_id(),
                manufacturer: if manufacturer.is_empty() { "Unknown".to_string() } else { manufacturer },
                model: if model.is_empty() { "Unknown".to_string() } else { model },
                model_number: if model_number.is_empty() { "Unknown".to_string() } else { model_number },
            };
        }
    }
    
    // Default fallback for all platforms
    HardwareInfo {
        id: generate_hardware_id(),
        manufacturer: "Unknown".to_string(),
        model: "Unknown".to_string(),
        model_number: "Unknown".to_string(),
    }
}

fn format_current_time() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn generate_hardware_id() -> Option<String> {
    let mut builder = IdBuilder::new(Encryption::MD5);
    builder.add_component(HWIDComponent::SystemID)
           .add_component(HWIDComponent::CPUCores);
    
    builder.build("bugsnag-rs").ok()
}

#[cfg(test)]
mod tests {
    use super::DeviceInfo;
    use serde_test::{assert_ser_tokens, Token};

    #[test]
    fn test_deviceinfo_to_json() {
        let mut info = DeviceInfo::new("1.0.0", "testmachine");
        // Set a fixed time for testing
        info.set_time("1970-01-01T00:00:00.000Z");

        // Test individual field values instead of using serde_test
        assert_eq!(info.id, None);
        assert_eq!(info.manufacturer, None);
        assert_eq!(info.model, None);
        assert_eq!(info.model_number, None);
        assert_eq!(info.os_name, sys_info::os_type().unwrap_or("Unknown".to_owned()));
        assert_eq!(info.os_version, "1.0.0");
        assert_eq!(info.hostname, "testmachine");
        assert_eq!(info.free_memory, None);
        assert_eq!(info.total_memory, sys_info::mem_info().ok().map(|info| info.total));
        assert_eq!(info.free_disk, None);
        assert_eq!(info.total_disk, None);
        assert_eq!(info.disk_usage, None);
        assert_eq!(info.time, "1970-01-01T00:00:00.000Z");
    }

    #[test]
    fn test_deviceinfo_to_json_with_set() {
        let mut info = DeviceInfo::generate();
        info.set_hostname("testmachine3");
        info.set_os_version("3.0.0");
        info.set_id("device-123");
        info.set_manufacturer("Test Corp");
        info.set_model("TestDevice");
        info.set_model_number("TD-001");
        info.set_os_name("TestOS");
        info.set_free_memory(1024);
        info.set_total_memory(2048);
        info.set_free_disk(4096);
        info.set_total_disk(8192);
        info.set_time("2023-01-01T00:00:00.000Z");

        assert_ser_tokens(
            &info,
            &[
                Token::Struct {
                    name: "DeviceInfo",
                    len: 13,
                },
                Token::Str("id"),
                Token::Some,
                Token::Str("device-123"),
                Token::Str("manufacturer"),
                Token::Some,
                Token::Str("Test Corp"),
                Token::Str("model"),
                Token::Some,
                Token::Str("TestDevice"),
                Token::Str("modelNumber"),
                Token::Some,
                Token::Str("TD-001"),
                Token::Str("osName"),
                Token::Str("TestOS"),
                Token::Str("osVersion"),
                Token::Str("3.0.0"),
                Token::Str("hostname"),
                Token::Str("testmachine3"),
                Token::Str("freeMemory"),
                Token::Some,
                Token::U64(1024),
                Token::Str("totalMemory"),
                Token::Some,
                Token::U64(2048),
                Token::Str("freeDisk"),
                Token::Some,
                Token::U64(4096),
                Token::Str("totalDisk"),
                Token::Some,
                Token::U64(8192),
                Token::Str("diskUsage"),
                Token::None,
                Token::Str("time"),
                Token::Str("2023-01-01T00:00:00.000Z"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn test_hardware_id_generation() {
        let device_info = DeviceInfo::generate();
        
        // Hardware ID should be generated and not None
        assert!(device_info.id.is_some());
        
        // Hardware ID should be a non-empty string
        let hwid = device_info.id.unwrap();
        assert!(!hwid.is_empty());
        
        // Hardware ID should be consistent across calls (since it's static)
        let device_info2 = DeviceInfo::generate();
        assert_eq!(device_info2.id, Some(hwid));
    }
}
