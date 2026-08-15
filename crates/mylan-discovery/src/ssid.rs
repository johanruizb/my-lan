//! Detección pura-Rust del SSID de la interfaz conectada (sin shell-out).

#![allow(unsafe_code)]
//!
//! - **Linux**: NetworkManager vía D-Bus con la API bloqueante de `zbus`.
//! - **Windows**: WLAN API nativa (`wlanapi`) del crate `windows`.
//! - **Otras** (macOS incluido): stub que devuelve `None`.

use crate::iface::LanInterface;

/// Detecta el SSID de la interfaz conectada (Wi-Fi). `None` si no es inalámbrica
/// o no se pudo leer. Best-effort: cualquier fallo degrada a `None`.
#[must_use]
pub fn detect_ssid(iface: &LanInterface) -> Option<String> {
    detect_ssid_impl(&iface.name)
}

#[cfg(target_os = "linux")]
fn detect_ssid_impl(iface_name: &str) -> Option<String> {
    linux::current_ssid(iface_name)
}

#[cfg(target_os = "windows")]
fn detect_ssid_impl(iface_name: &str) -> Option<String> {
    windows_wlan::current_ssid(iface_name)
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn detect_ssid_impl(_iface_name: &str) -> Option<String> {
    None
}

/// Linux: SSID vía NetworkManager (D-Bus, API bloqueante de zbus).
#[cfg(target_os = "linux")]
mod linux {
    use zbus::blocking::{Connection, Proxy};
    use zbus::zvariant::OwnedObjectPath;

    const NM_DEST: &str = "org.freedesktop.NetworkManager";
    const NM_PATH: &str = "/org/freedesktop/NetworkManager";

    /// Lee el SSID de la conexión inalámbrica activa de `iface_name`.
    ///
    /// Camino D-Bus: `GetDeviceByIpIface(nombre)` → objeto Device →
    /// propiedad `ActiveAccessPoint` (interfaz `Device.Wireless`) → propiedad
    /// `Ssid` (`ay`, bytes crudos) del `AccessPoint`. Cualquier fallo (NM ausente,
    /// interfaz no inalámbrica, sin AP) devuelve `None`.
    pub(super) fn current_ssid(iface_name: &str) -> Option<String> {
        let conn = Connection::system().ok()?;

        let nm = Proxy::new(&conn, NM_DEST, NM_PATH, NM_DEST).ok()?;
        // Objeto Device de NM para el nombre de interfaz del kernel.
        let device: OwnedObjectPath = nm.call("GetDeviceByIpIface", &(iface_name,)).ok()?;

        // Interfaz `Device.Wireless`: expone `ActiveAccessPoint`. Si el device no
        // es inalámbrico, la lectura de la propiedad falla → `None`.
        let wireless = Proxy::new(
            &conn,
            NM_DEST,
            device.as_str(),
            "org.freedesktop.NetworkManager.Device.Wireless",
        )
        .ok()?;
        let ap: OwnedObjectPath = wireless.get_property("ActiveAccessPoint").ok()?;
        // "/" = sin AP activo (no conectado a Wi-Fi).
        if ap.as_str() == "/" {
            return None;
        }

        let access_point = Proxy::new(
            &conn,
            NM_DEST,
            ap.as_str(),
            "org.freedesktop.NetworkManager.AccessPoint",
        )
        .ok()?;
        let ssid: Vec<u8> = access_point.get_property("Ssid").ok()?;
        if ssid.is_empty() {
            return None;
        }
        Some(String::from_utf8_lossy(&ssid).into_owned())
    }
}

/// Windows: SSID vía la WLAN API nativa (`wlanapi`).
#[cfg(target_os = "windows")]
mod windows_wlan {
    use std::ffi::c_void;
    use std::ptr;

    use windows::core::GUID;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::NetworkManagement::WiFi::{
        wlan_interface_state_connected, wlan_intf_opcode_current_connection, WlanCloseHandle,
        WlanEnumInterfaces, WlanFreeMemory, WlanOpenHandle, WlanQueryInterface,
        WLAN_CONNECTION_ATTRIBUTES, WLAN_INTERFACE_INFO_LIST,
    };

    const ERROR_SUCCESS: u32 = 0;
    const CLIENT_VERSION: u32 = 2;

    /// Lee el SSID de la primera interfaz WLAN conectada.
    ///
    /// La WLAN API enumera por GUID, no por nombre de interfaz del kernel; se
    /// asume un único adaptador Wi-Fi y se ignora `iface_name`. `None` ante
    /// cualquier fallo (sin adaptador, no conectado) — degrada a etiqueta/CIDR.
    pub(super) fn current_ssid(_iface_name: &str) -> Option<String> {
        unsafe {
            let mut handle = HANDLE(ptr::null_mut());
            let mut negotiated = 0u32;
            if WlanOpenHandle(CLIENT_VERSION, None, &mut negotiated, &mut handle) != ERROR_SUCCESS {
                return None;
            }
            let ssid = first_connected_ssid(handle);
            let _ = WlanCloseHandle(handle, None);
            ssid
        }
    }

    /// Enumera las interfaces WLAN y devuelve el SSID de la primera conectada.
    unsafe fn first_connected_ssid(handle: HANDLE) -> Option<String> {
        let mut list_ptr: *mut WLAN_INTERFACE_INFO_LIST = ptr::null_mut();
        if WlanEnumInterfaces(handle, None, &mut list_ptr) != ERROR_SUCCESS || list_ptr.is_null() {
            return None;
        }
        let list = &*list_ptr;
        let count = list.dwNumberOfItems as usize;
        let infos = list.InterfaceInfo.as_ptr();
        let mut result = None;
        for i in 0..count {
            let info = &*infos.add(i);
            if info.isState != wlan_interface_state_connected {
                continue;
            }
            if let Some(ssid) = connection_ssid(handle, &info.InterfaceGuid) {
                result = Some(ssid);
                break;
            }
        }
        WlanFreeMemory(list_ptr as *const c_void);
        result
    }

    /// Consulta `wlan_intf_opcode_current_connection` y extrae el SSID del
    /// `WLAN_CONNECTION_ATTRIBUTES`.
    unsafe fn connection_ssid(handle: HANDLE, guid: &GUID) -> Option<String> {
        let mut data_size = 0u32;
        let mut data_ptr: *mut c_void = ptr::null_mut();
        if WlanQueryInterface(
            handle,
            guid,
            wlan_intf_opcode_current_connection,
            None,
            &mut data_size,
            &mut data_ptr,
            None,
        ) != ERROR_SUCCESS
            || data_ptr.is_null()
        {
            return None;
        }
        let attrs = &*(data_ptr as *const WLAN_CONNECTION_ATTRIBUTES);
        let ssid = &attrs.wlanAssociationAttributes.dot11Ssid;
        let len = (ssid.uSSIDLength as usize).min(ssid.ucSSID.len());
        let result = if len == 0 {
            None
        } else {
            Some(String::from_utf8_lossy(&ssid.ucSSID[..len]).into_owned())
        };
        WlanFreeMemory(data_ptr as *const c_void);
        result
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    use std::net::Ipv4Addr;

    /// Crea una [`LanInterface`] mínima para tests (sin I/O, sin hardware).
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    fn fake_iface(name: &str) -> LanInterface {
        LanInterface {
            name: name.to_string(),
            ip: Ipv4Addr::new(192, 168, 1, 10),
            prefix_len: 24,
            mac: None,
            gateway_ip: None,
            gateway_mac: None,
            dns_servers: Vec::new(),
            ssid: None,
        }
    }

    #[test]
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    fn detect_ssid_returns_none_on_stub_platform() {
        let iface = fake_iface("wlan0");
        assert!(detect_ssid(&iface).is_none());
    }

    #[test]
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    fn detect_ssid_returns_none_for_arbitrary_iface_on_stub() {
        for name in ["eth0", "wlan0", "enp3s0", "", "lo"] {
            let iface = fake_iface(name);
            assert!(
                detect_ssid(&iface).is_none(),
                "stub detect_ssid debe devolver None para '{name}'"
            );
        }
    }
}

// Nota de determinismo: en Linux (`#[cfg(target_os = "linux")]`) y Windows
// (`#[cfg(target_os = "windows")]`), `detect_ssid` delega en D-Bus/WLAN API
// nativas que requieren hardware/daemon reales (NetworkManager, wlanapi). No
// se testea el camino live aquí por no ser determinista; el contrato del trait
// `SsidDetector` se valida vía `MockSsid` y la degradación a `None` en la
// plataforma stub. Ver AC-5 del plan ralplan-crear-set-pruebas-core-ui.
