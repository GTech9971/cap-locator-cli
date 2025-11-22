use std::ffi::CString;

use anyhow::{anyhow, Context, Result};
use hidapi::{HidApi, HidDevice};

/// HIDデバイスIOを抽象化するトレイト（テストでモックしやすくするため）
pub trait HidDeviceIo {
    fn send_feature_report(&self, data: &[u8]) -> hidapi::HidResult<()>;
    fn get_feature_report(&self, data: &mut [u8]) -> hidapi::HidResult<usize>;
}

impl HidDeviceIo for HidDevice {
    fn send_feature_report(&self, data: &[u8]) -> hidapi::HidResult<()> {
        HidDevice::send_feature_report(self, data)
    }

    fn get_feature_report(&self, data: &mut [u8]) -> hidapi::HidResult<usize> {
        HidDevice::get_feature_report(self, data)
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

        Self {
            path: info.path().to_owned(),
            vendor_id: info.vendor_id(),
            product_id: info.product_id(),
            serial_number: info.serial_number().map(|s| s.to_string()),
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

pub fn pick_single_device(api: &HidApi, filter: &FilterArgs, id: &str) -> Result<DeviceDescriptor> {
    let candidates = snapshot_devices(api, filter)
        .into_iter()
        .filter(|d| d.matches_id(id))
        .collect::<Vec<_>>();

    match candidates.len() {
        0 => Err(anyhow!("一致するlocatorがありませんでした: {}", id)),
        1 => Ok(candidates.into_iter().next().unwrap()),
        _ => Err(anyhow!(
            "複数のlocatorが一致しました。vendor/productやusageで絞り込んでください"
        )),
    }
}

pub fn query_status(
    device: &dyn HidDeviceIo,
    protocol: &ProtocolArgs,
    status_index: usize,
) -> Result<LocatorStatus> {
    let report_len = protocol.report_len.max(status_index + 1).max(2);

    let mut request = vec![0u8; report_len];
    request[0] = protocol.report_id;
    request[1] = protocol.command_status;
    device
        .send_feature_report(&request)
        .context("feature report送信に失敗")?;

    let mut response = vec![0u8; report_len];
    response[0] = protocol.report_id;
    let received = device
        .get_feature_report(&mut response)
        .context("feature report受信に失敗")?;
    response.truncate(received);

    let is_on = response.get(status_index).copied().unwrap_or_default() != 0;

    Ok(LocatorStatus {
        is_on,
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
    let report_len = protocol.report_len.max(3);
    let mut report = vec![0u8; report_len];
    report[0] = protocol.report_id;
    report[1] = protocol.command_set;
    report[2] = if turn_on { on_value } else { off_value };

    device
        .send_feature_report(&report)
        .context("feature report送信に失敗")?;
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
        fn send_feature_report(&self, data: &[u8]) -> hidapi::HidResult<()> {
            self.sent.borrow_mut().push(data.to_vec());
            Ok(())
        }

        fn get_feature_report(&self, data: &mut [u8]) -> hidapi::HidResult<usize> {
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
