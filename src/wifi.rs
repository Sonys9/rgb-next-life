use alloc::string::ToString;
use esp_radio::wifi::{self, Password, Ssid, ap::AccessPointConfig};
// use esp_wifi::EspWifiController;

const AP_SSID: &'static str = "RGB-NEXT-LIFE";
const AP_PASSWORD: &'static str = "rgb!nextlife";

pub async fn start_ap() {
    let ap_config = wifi::Config::AccessPoint(
        AccessPointConfig::default()
            .with_ssid(AP_SSID.try_into().unwrap())
            .with_authentication(wifi::AuthenticationMethodConfig::Wpa2Personal(AP_PASSWORD.try_into().unwrap()))
        );
    
}