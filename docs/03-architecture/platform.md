# Platform Layer

> Windows-specific abstractions and trait implementations.

---

## Table of Contents

- [Trait Design](#trait-design)
- [WMI Provider](#wmi-provider)
- [Registry Provider](#registry-provider)
- [Service Provider](#service-provider)
- [Credential Manager](#credential-manager)
- [COM Helpers](#com-helpers)
- [Testing with Mocks](#testing-with-mocks)

---

## Trait Design

```rust
// src/core/traits.rs
use crate::core::error::CoreError;

/// Abstraction for system information collection
#[cfg_attr(test, mockall::automock)]
pub trait SystemInfoProvider: Send + Sync {
    fn cpu(&self) -> Result<CpuInfo, CoreError>;
    fn memory(&self) -> Result<MemoryInfo, CoreError>;
    fn disks(&self) -> Result<Vec<DiskInfo>, CoreError>;
    fn gpu(&self) -> Result<Vec<GpuInfo>, CoreError>;
    fn motherboard(&self) -> Result<MotherboardInfo, CoreError>;
    fn temperatures(&self) -> Result<Vec<TemperatureReading>, CoreError>;
}

/// Abstraction for Windows Registry access
#[cfg_attr(test, mockall::automock)]
pub trait RegistryProvider: Send + Sync {
    fn read_key(&self, path: &str, name: &str) -> Result<String, CoreError>;
    fn enum_values(&self, path: &str) -> Result<Vec<RegValue>, CoreError>;
    fn delete_value(&self, path: &str, name: &str) -> Result<(), CoreError>;
    fn create_key(&self, path: &str) -> Result<(), CoreError>;
    fn write_value(&self, path: &str, name: &str, value: &RegValue) -> Result<(), CoreError>;
}

/// Abstraction for Windows Service control
#[cfg_attr(test, mockall::automock)]
pub trait ServiceProvider: Send + Sync {
    fn list_all(&self) -> Result<Vec<ServiceInfo>, CoreError>;
    fn get_status(&self, name: &str) -> Result<ServiceStatus, CoreError>;
    fn set_start_type(&self, name: &str, start_type: StartType) -> Result<(), CoreError>;
    fn stop(&self, name: &str) -> Result<(), CoreError>;
    fn start(&self, name: &str) -> Result<(), CoreError>;
}
```

---

## WMI Provider

```rust
// src/platform/windows/wmi.rs
use wmi::{COMLibrary, WMIConnection, Variant};
use crate::core::traits::SystemInfoProvider;

pub struct WmiSystemInfoProvider {
    con: WMIConnection,
}

impl WmiSystemInfoProvider {
    pub fn new() -> Result<Self, CoreError> {
        let com = COMLibrary::new()?;
        let con = WMIConnection::new(com)?;
        Ok(Self { con })
    }
}

impl SystemInfoProvider for WmiSystemInfoProvider {
    fn cpu(&self) -> Result<CpuInfo, CoreError> {
        let results: Vec<HashMap<String, Variant>> = self.con
            .raw_query("SELECT Name, NumberOfCores, NumberOfLogicalProcessors, MaxClockSpeed FROM Win32_Processor")?;

        let cpu = results.first().ok_or(CoreError::WmiEmpty)?;

        Ok(CpuInfo {
            name: cpu.get("Name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
            cores: cpu.get("NumberOfCores").and_then(|v| v.as_u32()).unwrap_or(0) as u8,
            threads: cpu.get("NumberOfLogicalProcessors").and_then(|v| v.as_u32()).unwrap_or(0) as u8,
            max_speed_mhz: cpu.get("MaxClockSpeed").and_then(|v| v.as_u32()).unwrap_or(0),
        })
    }
}
```

---

## Registry Provider

```rust
// src/platform/windows/registry.rs
use windows::Win32::System::Registry::{
    HKEY, REG_SZ, REG_DWORD, KEY_READ, KEY_WRITE,
    RegOpenKeyExW, RegQueryValueExW, RegEnumValueW, RegCloseKey,
};
use crate::core::traits::RegistryProvider;

pub struct WinRegistryProvider;

impl RegistryProvider for WinRegistryProvider {
    fn read_key(&self, path: &str, name: &str) -> Result<String, CoreError> {
        let hkey = parse_hkey(path)?;
        let subkey = path.split('\').skip(1).collect::<Vec<_>>().join("\");

        unsafe {
            let mut handle = HKEY::default();
            RegOpenKeyExW(hkey, &windows_str(&subkey), 0, KEY_READ, &mut handle)?;

            let mut buf = [0u8; 1024];
            let mut size = 1024u32;
            let mut type_id = 0u32;

            RegQueryValueExW(
                handle,
                &windows_str(name),
                None,
                Some(&mut type_id),
                Some(buf.as_mut_ptr()),
                Some(&mut size),
            )?;

            RegCloseKey(handle)?;

            if type_id == REG_SZ.0 {
                Ok(String::from_utf16_lossy(
                    &buf[..size as usize].chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect::<Vec<_>>()
                ))
            } else {
                Err(CoreError::RegistryTypeMismatch)
            }
        }
    }
}
```

---

## Service Provider

```rust
// src/platform/windows/services.rs
use windows::Win32::System::Services::{
    OpenSCManagerW, OpenServiceW, QueryServiceConfigW, ChangeServiceConfigW,
    ControlService, SERVICE_QUERY_CONFIG, SERVICE_CHANGE_CONFIG, SERVICE_STOP,
    SERVICE_STATUS, SC_MANAGER_ALL_ACCESS,
};
use crate::core::traits::ServiceProvider;

pub struct WinServiceProvider;

impl ServiceProvider for WinServiceProvider {
    fn list_all(&self) -> Result<Vec<ServiceInfo>, CoreError> {
        // Use EnumServicesStatusExW
        // Filter out Windows services to identify third-party
        todo!()
    }

    fn set_start_type(&self, name: &str, start_type: StartType) -> Result<(), CoreError> {
        unsafe {
            let scm = OpenSCManagerW(None, None, SC_MANAGER_ALL_ACCESS)?;
            let service = OpenServiceW(scm, &windows_str(name), SERVICE_CHANGE_CONFIG)?;

            let win_start_type = match start_type {
                StartType::Auto => SERVICE_AUTO_START,
                StartType::Manual => SERVICE_DEMAND_START,
                StartType::Disabled => SERVICE_DISABLED,
            };

            ChangeServiceConfigW(
                service,
                SERVICE_NO_CHANGE,
                win_start_type,
                SERVICE_NO_CHANGE,
                None, None, None, None, None, None, None,
            )?;

            CloseServiceHandle(service)?;
            CloseServiceHandle(scm)?;

            Ok(())
        }
    }
}
```

---

## Credential Manager

See [04-security/credential-manager.md](credential-manager.md) for full details.

```rust
// src/platform/windows/credential.rs
use windows::Win32::Security::Credentials::{
    CredWriteW, CredReadW, CredDeleteW,
    CREDENTIALW, CRED_TYPE_GENERIC, CRED_PERSIST_LOCAL_MACHINE,
};

pub struct CredentialManager;

impl CredentialManager {
    pub fn write(target: &str, secret: &[u8]) -> Result<(), CredentialError> {
        todo!()
    }

    pub fn read(target: &str) -> Result<Vec<u8>, CredentialError> {
        todo!()
    }
}
```

---

## COM Helpers

```rust
// src/platform/windows/com.rs
use windows::Win32::System::Com::{
    CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED,
};

pub struct ComInit;

impl ComInit {
    pub fn new() -> Result<Self, windows::core::Error> {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED)?;
        }
        Ok(Self)
    }
}

impl Drop for ComInit {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
```

---

## Testing with Mocks

```rust
// tests/integration/audit_flow.rs
use highlander_forge_blade::core::traits::{
    MockSystemInfoProvider, MockRegistryProvider, MockServiceProvider,
};
use highlander_forge_blade::core::audit::Auditor;

#[tokio::test]
async fn test_full_audit_flow() {
    let mut mock_sys = MockSystemInfoProvider::new();
    let mut mock_reg = MockRegistryProvider::new();
    let mut mock_svc = MockServiceProvider::new();

    mock_sys.expect_cpu()
        .times(1)
        .returning(|| Ok(CpuInfo {
            name: "Intel i7-9700K".into(),
            cores: 8, threads: 8, max_speed_mhz: 4900,
        }));

    mock_sys.expect_memory()
        .times(1)
        .returning(|| Ok(MemoryInfo {
            total_bytes: 17179869184,
            modules: vec![MemoryModule {
                capacity: 17179869184, speed_mhz: 3200, manufacturer: "Corsair".into(),
            }],
        }));

    mock_reg.expect_enum_values()
        .with(eq(r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run"))
        .times(1)
        .returning(|_| Ok(vec![
            RegValue { name: "OneDrive".into(), value: "C:\\Program Files\\...".into() },
        ]));

    mock_svc.expect_list_all()
        .times(1)
        .returning(|| Ok(vec![
            ServiceInfo { name: "Fax".into(), display_name: "Fax".into(),
                         start_type: StartType::Auto, status: ServiceStatus::Stopped },
        ]));

    let auditor = Auditor::new(&mock_sys, &mock_reg, &mock_svc);
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    let result = auditor.run_full(tx).await;
    assert!(result.is_ok());

    let msg = rx.recv().await.unwrap();
    assert!(matches!(msg, AppMsg::AuditProgress { phase: AuditPhase::Hardware, .. }));
}
```

---

*Last updated: 2026-06-20 | Document version: 1.0*
