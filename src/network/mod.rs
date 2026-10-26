pub mod http;
pub mod wifi;

use esp_radio::wifi::{AuthenticationMethodConfig, Password};
pub use http::*;
use serde::{Deserialize, Deserializer};
pub use wifi::*;

use crate::errors::Error;

#[derive(Debug, Deserialize)]
pub struct Auth {
    pub ssid: heapless::String<64>,
    pub password: heapless::String<128>,
    pub auth_method: AuthMethod,
}

#[derive(Debug, Clone)]
pub enum AuthMethod {
    Open,
    Wep,
    Wpa,
    Wpa2Personal,
    WpaWpa2Personal,
}

impl<'de> Deserialize<'de> for AuthMethod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = u8::deserialize(deserializer)?;
        AuthMethod::try_from(id).map_err(serde::de::Error::custom)
    }
}

#[macro_export]
macro_rules! mk_static {
    ($t:ty, $val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        STATIC_CELL.init($val)
    }};
}

impl TryFrom<u8> for AuthMethod {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => AuthMethod::Open,
            1 => AuthMethod::Wep,
            2 => AuthMethod::Wpa,
            3 => AuthMethod::Wpa2Personal,
            4 => AuthMethod::WpaWpa2Personal,
            _ => return Err("failed to parse auth metod")
        })
    }
}

impl TryFrom<&AuthMethod> for u8 {
    type Error = &'static str;

    fn try_from(value: &AuthMethod) -> Result<Self, Self::Error> {
        Ok(match value {
            AuthMethod::Open => 0,
            AuthMethod::Wep => 1,
            AuthMethod::Wpa => 2,
            AuthMethod::Wpa2Personal => 3,
            AuthMethod::WpaWpa2Personal => 4,
            _ => return Err("failed to parse auth metod")
        })
    }
}

impl AuthMethod {
    pub fn to_amc<'a>(&self, password: &'a str) -> Result<AuthenticationMethodConfig, Error> {
        Ok(match self {
            AuthMethod::Open => AuthenticationMethodConfig::Open,
            AuthMethod::Wep => AuthenticationMethodConfig::Wep(password.try_into().unwrap()),
            AuthMethod::Wpa => AuthenticationMethodConfig::Wpa(password.try_into().unwrap()),
            AuthMethod::Wpa2Personal => AuthenticationMethodConfig::Wpa2Personal(password.try_into().unwrap()),
            AuthMethod::WpaWpa2Personal => AuthenticationMethodConfig::WpaWpa2Personal(password.try_into().unwrap()),
            _ => return Err(Error::Parse)
        })
    }
}