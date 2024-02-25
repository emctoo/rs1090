extern crate alloc;

use crate::decode::IdentityCode;
use alloc::fmt;
use deku::prelude::*;
use serde::Serialize;

/// Table: A-2-97
#[derive(Debug, PartialEq, Serialize, DekuRead, Copy, Clone)]
pub struct AircraftStatus {
    pub sub_type: AircraftStatusType,
    pub emergency_state: EmergencyState,
    pub squawk: IdentityCode,
}

#[derive(Debug, PartialEq, Serialize, DekuRead, Copy, Clone)]
#[deku(type = "u8", bits = "3")]
#[serde(rename_all = "snake_case")]
pub enum AircraftStatusType {
    #[deku(id = "0")]
    NoInformation,
    #[deku(id = "1")]
    #[serde(rename = "emergency_priority")]
    EmergencyPriorityStatus,
    #[deku(id = "2")]
    #[serde(rename = "acas_ra")]
    ACASRaBroadcast,
    #[deku(id_pat = "_")]
    Reserved,
}

#[derive(Debug, PartialEq, Serialize, DekuRead, Copy, Clone)]
#[deku(type = "u8", bits = "3")]
#[serde(rename_all = "snake_case")]
pub enum EmergencyState {
    None = 0,
    General = 1,
    Medical = 2,
    MinimumFuel = 3,
    NoCommunication = 4,
    UnlawfulInterference = 5,
    DownedAircraft = 6,
    Reserved = 7,
}

impl fmt::Display for EmergencyState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::None => "No emergency",
            Self::General => "General emergency",
            Self::Medical => "Lifeguard/Medical emergency",
            Self::MinimumFuel => "Minimum fuel",
            Self::NoCommunication => "No communication",
            Self::UnlawfulInterference => "Unlawful interference",
            Self::DownedAircraft => "Downed aircraft",
            Self::Reserved => "reserved",
        };
        write!(f, "{s}")?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone)]
pub struct OperationCodeSurface {
    #[deku(bits = "1")]
    pub poe: u8,
    #[deku(bits = "1")]
    pub cdti: u8,
    #[deku(bits = "1")]
    pub b2_low: u8,
    #[deku(bits = "3")]
    #[deku(pad_bits_before = "6")]
    pub lw: u8,
}
