use std::ffi::CString;

use anyhow::{anyhow, bail, Context, Result};
use hidapi::{HidApi, HidDevice};

use crate::util::format_bytes;

const COMMAND_STATUS: u8 = 0x01;
const COMMAND_SET: u8 = 0x02;
const RESPONSE_HEADER: u8 = 0xFF;

/// HIDデバイスIOを抽象化するトレイト（テストでモックしやすくするため）
pub trait HidDeviceIo {
    fn write(&self, data: &[u8]) -> hidapi::HidResult<usize>;
    fn read_timeout(&self, data: &mut [u8], timeout_ms: i32) -> hidapi::HidResult<usize>;
}

impl HidDeviceIo for HidDevice {
    fn write(&self, data: &[u8]) -> hidapi::HidResult<usize> {
        HidDevice::write(self, data)
    }

    fn read_timeout(&self, data: &mut [u8], timeout_ms: i32) -> hidapi::HidResult<usize> {
        HidDevice::read_timeout(self, data, timeout_ms)
    }
}

use crate::cli::{FilterArgs, ProtocolArgs};

#[derive(Clone, Debug)]
pub struct DeviceDescriptor {
    pub path: CString,
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial_number: Option<String>,
    pub usage_page: Option<u16>,
    pub usage: Option<u16>,
}

impl DeviceDescriptor {
    pub fn from_info(info: &hidapi::DeviceInfo) -> Self {
        #[cfg(not(target_os = "linux"))]
        let usage_page = Some(info.usage_page());
        #[cfg(not(target_os = "linux"))]
        let usage = Some(info.usage());
        #[cfg(target_os = "linux")]
        let usage_page = None;
        #[cfg(target_os = "linux")]
        let usage = None;

        // 空文字シリアルは識別に使えないので None 扱いにする
        let serial_number = info
            .serial_number()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());

        Self {
            path: info.path().to_owned(),
            vendor_id: info.vendor_id(),
            product_id: info.product_id(),
            serial_number,
            usage_page,
            usage,
        }
    }

    pub fn locator_id(&self) -> String {
        self.serial_number
            .clone()
            .unwrap_or_else(|| self.path.to_string_lossy().into_owned())
    }

    pub fn matches_id(&self, query: &str) -> bool {
        if query.is_empty() {
            return false;
        }
        let serial_match = self
            .serial_number
            .as_ref()
            .map(|serial| serial == query || serial.contains(query))
            .unwrap_or(false);
        serial_match || self.path.to_string_lossy().contains(query)
    }
}

pub struct LocatorStatus {
    pub is_on: bool,
    pub mask: u8,
    pub raw: Vec<u8>,
}

pub fn snapshot_devices(api: &HidApi, filter: &FilterArgs) -> Vec<DeviceDescriptor> {
    // HIDデバイス一覧を取得し、フィルタに合致するものだけ抽出
    let mut devices = Vec::new();
    for info in api.device_list() {
        if !matches_filter(&info, filter) {
            continue;
        }
        devices.push(DeviceDescriptor::from_info(&info));
    }
    devices
}

pub fn pick_single_device(api: &HidApi, filter: &FilterArgs, id: Option<&str>) -> Result<DeviceDescriptor> {
    let mut candidates = snapshot_devices(api, filter);
    if let Some(id) = id {
        if !id.is_empty() {
            candidates.retain(|d| d.matches_id(id));
        }
    }

    match candidates.len() {
        0 => {
            if let Some(id) = id {
                Err(anyhow!("一致するlocatorがありませんでした: {}", id))
            } else {
                Err(anyhow!("フィルタに一致するlocatorがありませんでした"))
            }
        }
        1 => Ok(candidates.into_iter().next().unwrap()),
        _ => Err(anyhow!(
            "複数のlocatorが見つかりました。idを指定するか、vendor/productやusageで絞り込んでください"
        )),
    }
}

pub fn query_status(
    device: &dyn HidDeviceIo,
    protocol: &ProtocolArgs,
) -> Result<LocatorStatus> {
    let report_len = protocol.report_len.max(2);

    let mut request = vec![0u8; report_len];
    request[0] = COMMAND_STATUS;
    device
        .write(&request)
        .context("output report送信に失敗")?;

    let mut response = vec![0u8; report_len];
    let received = device
        .read_timeout(&mut response, protocol.read_timeout_ms)
        .context("input report受信に失敗")?;
    response.truncate(received);

    if response.first().copied() != Some(RESPONSE_HEADER) {
        bail!(
            "応答先頭が0xFFではありません: [{}]",
            format_bytes(&response)
        );
    }

    let mask = response
        .get(1)
        .copied()
        .ok_or_else(|| anyhow!("LED状態のバイトが不足しています: [{}]", format_bytes(&response)))?;
    let is_on = mask != 0;

    Ok(LocatorStatus {
        is_on,
        mask,
        raw: response,
    })
}

pub fn set_light(
    device: &dyn HidDeviceIo,
    protocol: &ProtocolArgs,
    turn_on: bool,
    on_value: u8,
    off_value: u8,
) -> Result<()> {
    let report_len = protocol.report_len.max(2);
    let mut report = vec![0u8; report_len];
    report[0] = COMMAND_SET;
    report[1] = if turn_on { on_value } else { off_value };

    device
        .write(&report)
        .context("output report送信に失敗")?;
    Ok(())
}

fn matches_filter(info: &hidapi::DeviceInfo, filter: &FilterArgs) -> bool {
    if let Some(vendor_id) = filter.vendor_id {
        if info.vendor_id() != vendor_id {
            return false;
        }
    }
    if let Some(product_id) = filter.product_id {
        if info.product_id() != product_id {
            return false;
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        if let Some(usage_page) = filter.usage_page {
            if info.usage_page() != usage_page {
                return false;
            }
        }
        if let Some(usage) = filter.usage {
            if info.usage() != usage {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
pub mod mock {
    use std::cell::RefCell;

    use super::*;
    use hidapi::HidError;

    /// テスト用の簡易モックデバイス。送信内容を記録し、あらかじめ設定した応答を返す
    pub struct MockDevice {
        pub sent: RefCell<Vec<Vec<u8>>>,
        next_response: RefCell<Option<Vec<u8>>>,
    }

    impl MockDevice {
        pub fn with_response(response: Vec<u8>) -> Self {
            Self {
                sent: RefCell::new(Vec::new()),
                next_response: RefCell::new(Some(response)),
            }
        }

        pub fn last_sent(&self) -> Option<Vec<u8>> {
            self.sent.borrow().last().cloned()
        }
    }

    impl HidDeviceIo for MockDevice {
        fn write(&self, data: &[u8]) -> hidapi::HidResult<usize> {
            self.sent.borrow_mut().push(data.to_vec());
            Ok(data.len())
        }

        fn read_timeout(&self, data: &mut [u8], _timeout_ms: i32) -> hidapi::HidResult<usize> {
            let response = self.next_response.borrow_mut().take().ok_or_else(|| {
                HidError::HidApiError {
                    message: "mock response not set".to_string(),
                }
            })?;

            let len = std::cmp::min(data.len(), response.len());
            data[..len].copy_from_slice(&response[..len]);
            Ok(len)
        }
    }
}
