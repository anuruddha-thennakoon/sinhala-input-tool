use windows::core::GUID;

/// Class id of the text service (the COM object TSF instantiates).
pub const CLSID_SINHALA_TIP: GUID = GUID::from_u128(0x060036e6_8d14_4bb1_ad0b_4513f41d3579);
/// Id of the single Sinhala language profile.
pub const GUID_SINHALA_PROFILE: GUID = GUID::from_u128(0x566199c5_f803_4be1_a75c_319cc05582b5);
/// LANGID for Sinhala (Sri Lanka): LANG_SINHALESE 0x5B, SUBLANG_DEFAULT 0x01.
pub const LANGID_SINHALA: u16 = 0x045B;
pub const TIP_DESCRIPTION: &str = "Sinhala Input";
/// Id of the language-bar (tray) button.
pub const GUID_LANGBAR_ITEM: GUID = GUID::from_u128(0x7c1d5e0a_3b64_4f72_a2e8_91f4c60b5d37);
/// Display attribute (underline) applied to our composition text.
pub const GUID_DISPLAY_ATTR_INPUT: GUID = GUID::from_u128(0x40206842_bd98_4fc6_88ef_abf33e3620d2);
