use windows::core::GUID;

/// Class id of the text service (the COM object TSF instantiates).
pub const CLSID_SINHALA_TIP: GUID = GUID::from_u128(0x060036e6_8d14_4bb1_ad0b_4513f41d3579);
/// Id of the single Sinhala language profile.
pub const GUID_SINHALA_PROFILE: GUID = GUID::from_u128(0x566199c5_f803_4be1_a75c_319cc05582b5);
/// LANGID for Sinhala (Sri Lanka): LANG_SINHALESE 0x5B, SUBLANG_DEFAULT 0x01.
pub const LANGID_SINHALA: u16 = 0x045B;
pub const TIP_DESCRIPTION: &str = "Sinhala Input";
