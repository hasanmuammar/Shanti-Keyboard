use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::Value;

use shanti_paths;

/// Result of a layout conversion operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConversionResult {
    Ok,
    UnsupportedLayout,
    OpenError,
    SaveError,
}

/// Key conversion map from pre-v3 / Avro key names to v3 key names.
fn key_conversion_map() -> BTreeMap<&'static str, &'static str> {
    let mut m = BTreeMap::new();
    m.insert("Key_0_AltGr", "Key_0_AltGr");
    m.insert("Key_0_Normal", "Key_0_Normal");
    m.insert("Key_0_Shift", "Key_ParenRight_Normal");
    m.insert("Key_0_ShiftAltGr", "Key_ParenRight_AltGr");
    m.insert("Key_1_AltGr", "Key_1_AltGr");
    m.insert("Key_1_Normal", "Key_1_Normal");
    m.insert("Key_1_Shift", "Key_Exclaim_Normal");
    m.insert("Key_1_ShiftAltGr", "Key_Exclaim_AltGr");
    m.insert("Key_2_AltGr", "Key_2_AltGr");
    m.insert("Key_2_Normal", "Key_2_Normal");
    m.insert("Key_2_Shift", "Key_At_Normal");
    m.insert("Key_2_ShiftAltGr", "Key_At_AltGr");
    m.insert("Key_3_AltGr", "Key_3_AltGr");
    m.insert("Key_3_Normal", "Key_3_Normal");
    m.insert("Key_3_Shift", "Key_Hash_Normal");
    m.insert("Key_3_ShiftAltGr", "Key_Hash_AltGr");
    m.insert("Key_4_AltGr", "Key_4_AltGr");
    m.insert("Key_4_Normal", "Key_4_Normal");
    m.insert("Key_4_Shift", "Key_Dollar_Normal");
    m.insert("Key_4_ShiftAltGr", "Key_Dollar_AltGr");
    m.insert("Key_5_AltGr", "Key_5_AltGr");
    m.insert("Key_5_Normal", "Key_5_Normal");
    m.insert("Key_5_Shift", "Key_Percent_Normal");
    m.insert("Key_5_ShiftAltGr", "Key_Percent_AltGr");
    m.insert("Key_6_AltGr", "Key_6_AltGr");
    m.insert("Key_6_Normal", "Key_6_Normal");
    m.insert("Key_6_Shift", "Key_Circum_Normal");
    m.insert("Key_6_ShiftAltGr", "Key_Circum_AltGr");
    m.insert("Key_7_AltGr", "Key_7_AltGr");
    m.insert("Key_7_Normal", "Key_7_Normal");
    m.insert("Key_7_Shift", "Key_Ampersand_Normal");
    m.insert("Key_7_ShiftAltGr", "Key_Ampersand_AltGr");
    m.insert("Key_8_AltGr", "Key_8_AltGr");
    m.insert("Key_8_Normal", "Key_8_Normal");
    m.insert("Key_8_Shift", "Key_Asterisk_Normal");
    m.insert("Key_8_ShiftAltGr", "Key_Asterisk_AltGr");
    m.insert("Key_9_AltGr", "Key_9_AltGr");
    m.insert("Key_9_Normal", "Key_9_Normal");
    m.insert("Key_9_Shift", "Key_ParenLeft_Normal");
    m.insert("Key_9_ShiftAltGr", "Key_ParenLeft_AltGr");
    m.insert("Key_A_AltGr", "Key_a_AltGr");
    m.insert("Key_A_Normal", "Key_a_Normal");
    m.insert("Key_A_Shift", "Key_A_Normal");
    m.insert("Key_A_ShiftAltGr", "Key_A_AltGr");
    m.insert("Key_B_AltGr", "Key_b_AltGr");
    m.insert("Key_B_Normal", "Key_b_Normal");
    m.insert("Key_B_Shift", "Key_B_Normal");
    m.insert("Key_B_ShiftAltGr", "Key_B_AltGr");
    m.insert("Key_BackQuote_AltGr", "Key_Grave_AltGr");
    m.insert("Key_BackQuote_Normal", "Key_Grave_Normal");
    m.insert("Key_BackQuote_Shift", "Key_Tilde_Normal");
    m.insert("Key_BackQuote_ShiftAltGr", "Key_Tilde_AltGr");
    m.insert("Key_BackSlash_AltGr", "Key_BackSlash_AltGr");
    m.insert("Key_BackSlash_Normal", "Key_BackSlash_Normal");
    m.insert("Key_BackSlash_Shift", "Key_Bar_Normal");
    m.insert("Key_BackSlash_ShiftAltGr", "Key_Bar_AltGr");
    m.insert("Key_C_AltGr", "Key_c_AltGr");
    m.insert("Key_C_Normal", "Key_c_Normal");
    m.insert("Key_C_Shift", "Key_C_Normal");
    m.insert("Key_C_ShiftAltGr", "Key_C_AltGr");
    m.insert("Key_CloseBracket_AltGr", "Key_BracketRight_AltGr");
    m.insert("Key_CloseBracket_Normal", "Key_BracketRight_Normal");
    m.insert("Key_CloseBracket_Shift", "Key_BraceRight_Normal");
    m.insert("Key_CloseBracket_ShiftAltGr", "Key_BraceRight_AltGr");
    m.insert("Key_Comma_AltGr", "Key_Comma_AltGr");
    m.insert("Key_Comma_Normal", "Key_Comma_Normal");
    m.insert("Key_Comma_Shift", "Key_Less_Normal");
    m.insert("Key_Comma_ShiftAltGr", "Key_Less_AltGr");
    m.insert("Key_D_AltGr", "Key_d_AltGr");
    m.insert("Key_D_Normal", "Key_d_Normal");
    m.insert("Key_D_Shift", "Key_D_Normal");
    m.insert("Key_D_ShiftAltGr", "Key_D_AltGr");
    m.insert("Key_E_AltGr", "Key_e_AltGr");
    m.insert("Key_E_Normal", "Key_e_Normal");
    m.insert("Key_E_Shift", "Key_E_Normal");
    m.insert("Key_E_ShiftAltGr", "Key_E_AltGr");
    m.insert("Key_Equals_AltGr", "Key_Equals_AltGr");
    m.insert("Key_Equals_Normal", "Key_Equals_Normal");
    m.insert("Key_Equals_Shift", "Key_Plus_Normal");
    m.insert("Key_Equals_ShiftAltGr", "Key_Plus_AltGr");
    m.insert("Key_F_AltGr", "Key_f_AltGr");
    m.insert("Key_F_Normal", "Key_f_Normal");
    m.insert("Key_F_Shift", "Key_F_Normal");
    m.insert("Key_F_ShiftAltGr", "Key_F_AltGr");
    m.insert("Key_G_AltGr", "Key_g_AltGr");
    m.insert("Key_G_Normal", "Key_g_Normal");
    m.insert("Key_G_Shift", "Key_G_Normal");
    m.insert("Key_G_ShiftAltGr", "Key_G_AltGr");
    m.insert("Key_H_AltGr", "Key_h_AltGr");
    m.insert("Key_H_Normal", "Key_h_Normal");
    m.insert("Key_H_Shift", "Key_H_Normal");
    m.insert("Key_H_ShiftAltGr", "Key_H_AltGr");
    m.insert("Key_I_AltGr", "Key_i_AltGr");
    m.insert("Key_I_Normal", "Key_i_Normal");
    m.insert("Key_I_Shift", "Key_I_Normal");
    m.insert("Key_I_ShiftAltGr", "Key_I_AltGr");
    m.insert("Key_J_AltGr", "Key_j_AltGr");
    m.insert("Key_J_Normal", "Key_j_Normal");
    m.insert("Key_J_Shift", "Key_J_Normal");
    m.insert("Key_J_ShiftAltGr", "Key_J_AltGr");
    m.insert("Key_K_AltGr", "Key_k_AltGr");
    m.insert("Key_K_Normal", "Key_k_Normal");
    m.insert("Key_K_Shift", "Key_K_Normal");
    m.insert("Key_K_ShiftAltGr", "Key_K_AltGr");
    m.insert("Key_L_AltGr", "Key_l_AltGr");
    m.insert("Key_L_Normal", "Key_l_Normal");
    m.insert("Key_L_Shift", "Key_L_Normal");
    m.insert("Key_L_ShiftAltGr", "Key_L_AltGr");
    m.insert("Key_M_AltGr", "Key_m_AltGr");
    m.insert("Key_M_Normal", "Key_m_Normal");
    m.insert("Key_M_Shift", "Key_M_Normal");
    m.insert("Key_M_ShiftAltGr", "Key_M_AltGr");
    m.insert("Key_Minus_AltGr", "Key_Minus_AltGr");
    m.insert("Key_Minus_Normal", "Key_Minus_Normal");
    m.insert("Key_Minus_Shift", "Key_UnderScore_Normal");
    m.insert("Key_Minus_ShiftAltGr", "Key_UnderScore_AltGr");
    m.insert("Key_N_AltGr", "Key_n_AltGr");
    m.insert("Key_N_Normal", "Key_n_Normal");
    m.insert("Key_N_Shift", "Key_N_Normal");
    m.insert("Key_N_ShiftAltGr", "Key_N_AltGr");
    m.insert("Key_O_AltGr", "Key_o_AltGr");
    m.insert("Key_O_Normal", "Key_o_Normal");
    m.insert("Key_O_Shift", "Key_O_Normal");
    m.insert("Key_O_ShiftAltGr", "Key_O_AltGr");
    m.insert("Key_OpenBracket_AltGr", "Key_BracketLeft_AltGr");
    m.insert("Key_OpenBracket_Normal", "Key_BracketLeft_Normal");
    m.insert("Key_OpenBracket_Shift", "Key_BraceLeft_Normal");
    m.insert("Key_OpenBracket_ShiftAltGr", "Key_BraceLeft_AltGr");
    m.insert("Key_P_AltGr", "Key_p_AltGr");
    m.insert("Key_P_Normal", "Key_p_Normal");
    m.insert("Key_P_Shift", "Key_P_Normal");
    m.insert("Key_P_ShiftAltGr", "Key_P_AltGr");
    m.insert("Key_Period_AltGr", "Key_Period_AltGr");
    m.insert("Key_Period_Normal", "Key_Period_Normal");
    m.insert("Key_Period_Shift", "Key_Greater_Normal");
    m.insert("Key_Period_ShiftAltGr", "Key_Greater_AltGr");
    m.insert("Key_Q_AltGr", "Key_q_AltGr");
    m.insert("Key_Q_Normal", "Key_q_Normal");
    m.insert("Key_Q_Shift", "Key_Q_Normal");
    m.insert("Key_Q_ShiftAltGr", "Key_Q_AltGr");
    m.insert("Key_Quote_AltGr", "Key_Apostrophe_AltGr");
    m.insert("Key_Quote_Normal", "Key_Apostrophe_Normal");
    m.insert("Key_Quote_Shift", "Key_Quote_Normal");
    m.insert("Key_Quote_ShiftAltGr", "Key_Quote_AltGr");
    m.insert("Key_R_AltGr", "Key_r_AltGr");
    m.insert("Key_R_Normal", "Key_r_Normal");
    m.insert("Key_R_Shift", "Key_R_Normal");
    m.insert("Key_R_ShiftAltGr", "Key_R_AltGr");
    m.insert("Key_S_AltGr", "Key_s_AltGr");
    m.insert("Key_S_Normal", "Key_s_Normal");
    m.insert("Key_S_Shift", "Key_S_Normal");
    m.insert("Key_S_ShiftAltGr", "Key_S_AltGr");
    m.insert("Key_Semicolon_AltGr", "Key_Semicolon_AltGr");
    m.insert("Key_Semicolon_Normal", "Key_Semicolon_Normal");
    m.insert("Key_Semicolon_Shift", "Key_Colon_Normal");
    m.insert("Key_Semicolon_ShiftAltGr", "Key_Colon_AltGr");
    m.insert("Key_Slash_AltGr", "Key_Slash_AltGr");
    m.insert("Key_Slash_Normal", "Key_Slash_Normal");
    m.insert("Key_Slash_Shift", "Key_Question_Normal");
    m.insert("Key_Slash_ShiftAltGr", "Key_Question_AltGr");
    m.insert("Key_T_AltGr", "Key_t_AltGr");
    m.insert("Key_T_Normal", "Key_t_Normal");
    m.insert("Key_T_Shift", "Key_T_Normal");
    m.insert("Key_T_ShiftAltGr", "Key_T_AltGr");
    m.insert("Key_U_AltGr", "Key_u_AltGr");
    m.insert("Key_U_Normal", "Key_u_Normal");
    m.insert("Key_U_Shift", "Key_U_Normal");
    m.insert("Key_U_ShiftAltGr", "Key_U_AltGr");
    m.insert("Key_V_AltGr", "Key_v_AltGr");
    m.insert("Key_V_Normal", "Key_v_Normal");
    m.insert("Key_V_Shift", "Key_V_Normal");
    m.insert("Key_V_ShiftAltGr", "Key_V_AltGr");
    m.insert("Key_W_AltGr", "Key_w_AltGr");
    m.insert("Key_W_Normal", "Key_w_Normal");
    m.insert("Key_W_Shift", "Key_W_Normal");
    m.insert("Key_W_ShiftAltGr", "Key_W_AltGr");
    m.insert("Key_X_AltGr", "Key_x_AltGr");
    m.insert("Key_X_Normal", "Key_x_Normal");
    m.insert("Key_X_Shift", "Key_X_Normal");
    m.insert("Key_X_ShiftAltGr", "Key_X_AltGr");
    m.insert("Key_Y_AltGr", "Key_y_AltGr");
    m.insert("Key_Y_Normal", "Key_y_Normal");
    m.insert("Key_Y_Shift", "Key_Y_Normal");
    m.insert("Key_Y_ShiftAltGr", "Key_Y_AltGr");
    m.insert("Key_Z_AltGr", "Key_z_AltGr");
    m.insert("Key_Z_Normal", "Key_z_Normal");
    m.insert("Key_Z_Shift", "Key_Z_Normal");
    m.insert("Key_Z_ShiftAltGr", "Key_Z_AltGr");
    m.insert("Num0", "Num0");
    m.insert("Num1", "Num1");
    m.insert("Num2", "Num2");
    m.insert("Num3", "Num3");
    m.insert("Num4", "Num4");
    m.insert("Num5", "Num5");
    m.insert("Num6", "Num6");
    m.insert("Num7", "Num7");
    m.insert("Num8", "Num8");
    m.insert("Num9", "Num9");
    m.insert("NumAdd", "NumAdd");
    m.insert("NumDecimal", "NumDecimal");
    m.insert("NumDivide", "NumDivide");
    m.insert("NumMultiply", "NumMultiply");
    m.insert("NumSubtract", "NumSubtract");
    // Fix the previous layout format bug. Issue #98
    m.insert("Key_BackBracket_AltGr", "Key_BackSlash_AltGr");
    m.insert("Key_BackBracket_Normal", "Key_BackSlash_Normal");
    m.insert("Key_BackBracket_Shift", "Key_Bar_Normal");
    m.insert("Key_BackBracket_ShiftAltGr", "Key_Bar_AltGr");
    m
}

fn unescape_xml(escaped: &str) -> String {
    escaped
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&apos;", "'")
        .replace("&quot;", "\"")
}

/// Convert a v1 layout object to v3 and save to `save_path`.
fn convert_layout_format(layout: Value, save_path: &PathBuf) -> ConversionResult {
    let old_keys = layout
        .get("layout")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or(Value::Object(serde_json::Map::new()));

    let mut new_keys = serde_json::Map::new();
    for (k, v) in old_keys {
        let new_key = key_conversion_map()
            .get(k.as_str())
            .copied()
            .unwrap_or(k.as_str());
        new_keys.insert(new_key.to_string(), v);
    }

    let developer = layout
        .get("info")
        .and_then(|i| i.get("layout"))
        .and_then(|l| l.get("developer"))
        .cloned()
        .unwrap_or(Value::Null);
    let name = layout
        .get("info")
        .and_then(|i| i.get("layout"))
        .and_then(|l| l.get("name"))
        .cloned()
        .unwrap_or(Value::Null);
    let version = layout
        .get("info")
        .and_then(|i| i.get("layout"))
        .and_then(|l| l.get("version"))
        .cloned()
        .unwrap_or(Value::Null);

    let mut info_layout = serde_json::Map::new();
    info_layout.insert("developer".to_string(), developer);
    info_layout.insert("name".to_string(), name);
    info_layout.insert("version".to_string(), version);

    let mut info = serde_json::Map::new();
    info.insert("layout".to_string(), Value::Object(info_layout));
    info.insert("type".to_string(), Value::String("fixed".to_string()));

    let mut out = serde_json::Map::new();
    out.insert("info".to_string(), Value::Object(info));
    out.insert("layout".to_string(), Value::Object(new_keys));

    convert_to_v3(Value::Object(out), save_path)
}

/// Convert an Avro Keyboard 5 XML layout to v3 and save to `save_path`.
fn convert_avro_layout(xml_text: &str, save_path: &PathBuf) -> ConversionResult {
    // Minimal XML parsing: iterate over start elements.
    let mut info_layout = serde_json::Map::new();
    let mut layout_dev = serde_json::Map::new();
    let mut keys = serde_json::Map::new();
    let mut avro_version = String::new();
    let mut unsupported = false;

    // Very simple element scan — handles the flat Avro XML structure.
    for line in xml_text.lines() {
        let line = line.trim();
        if !line.starts_with("<") || line.starts_with("</") {
            continue;
        }
        // Extract element name
        let start = line.find('<').map(|i| i + 1).unwrap_or(0);
        let end = line[start..].find('>').map(|i| start + i).unwrap_or(line.len());
        let elem = &line[start..end];
        let elem = elem.trim_start_matches('/').trim();

        if elem == "Layout" || elem == "KeyData" || elem.is_empty() {
            continue;
        }

        // Extract element text content (between > and <)
        let content_start = line.find('>').map(|i| i + 1).unwrap_or(0);
        let content_end = line.rfind('<').unwrap_or(line.len());
        let data = if content_end > content_start {
            line[content_start..content_end].trim()
        } else {
            ""
        };

        if elem == "AvroKeyboardVersion" {
            avro_version = data.to_string();
            if avro_version != "5" {
                unsupported = true;
                break;
            }
            continue;
        }

        if !elem.contains("Key") && !elem.contains("Num") {
            // Metadata conversion
            match elem {
                "LayoutName" => {
                    info_layout.insert("name".to_string(), Value::String(data.to_string()));
                }
                "LayoutVersion" => {
                    info_layout
                        .insert("version".to_string(), Value::String(data.to_string()));
                }
                "DeveloperName" => {
                    layout_dev.insert("name".to_string(), Value::String(data.to_string()));
                }
                "DeveloperComment" => {
                    layout_dev.insert(
                        "comment".to_string(),
                        Value::String(unescape_xml(data)),
                    );
                }
                _ => {}
            }
        } else {
            // Key conversion
            let mut key = elem.to_string();
            if key.contains("OEM1") {
                key = key.replace("OEM1", "Semicolon");
            }
            if key.contains("OEM2") {
                key = key.replace("OEM2", "Slash");
            }
            if key.contains("OEM3") {
                key = key.replace("OEM3", "BackQuote");
            }
            if key.contains("OEM4") {
                key = key.replace("OEM4", "OpenBracket");
            }
            if key.contains("OEM5") {
                key = key.replace("OEM5", "BackSlash");
            }
            if key.contains("OEM6") {
                key = key.replace("OEM6", "CloseBracket");
            }
            if key.contains("OEM7") {
                key = key.replace("OEM7", "Quote");
            }
            if key.contains("MINUS") {
                key = key.replace("MINUS", "Minus");
            }
            if key.contains("PLUS") {
                key = key.replace("PLUS", "Equals");
            }
            if key.contains("PERIOD") {
                key = key.replace("PERIOD", "Period");
            }
            if key.contains("COMMA") {
                key = key.replace("COMMA", "Comma");
            }

            let new_key = key_conversion_map()
                .get(key.as_str())
                .copied()
                .unwrap_or(key.as_str());
            keys.insert(
                new_key.to_string(),
                Value::String(unescape_xml(data).to_string()),
            );
        }
    }

    if unsupported {
        return ConversionResult::UnsupportedLayout;
    }

    info_layout.insert(
        "developer".to_string(),
        Value::Object(layout_dev),
    );
    let mut info = serde_json::Map::new();
    info.insert("layout".to_string(), Value::Object(info_layout));
    info.insert("type".to_string(), Value::String("fixed".to_string()));

    let mut out = serde_json::Map::new();
    out.insert("info".to_string(), Value::Object(info));
    out.insert("layout".to_string(), Value::Object(keys));

    convert_to_v3(Value::Object(out), save_path)
}

/// Normalize a layout object into format version 3 and save it.
fn convert_to_v3(layout: Value, save_path: &PathBuf) -> ConversionResult {
    let mut layout = layout;
    let info = layout
        .get_mut("info")
        .and_then(Value::as_object_mut)
        .cloned()
        .unwrap_or_default();
    let mut info_layout = info
        .get_mut("layout")
        .and_then(Value::as_object_mut)
        .cloned()
        .unwrap_or_default();

    // Legacy phonetic layouts keep their single instructional image,
    // renamed image0 -> image, and adopt the unified "transliteration" type.
    if info.get("type").and_then(Value::as_str) == Some("phonetic") {
        if info_layout.contains_key("image0") {
            let image0 = info_layout.remove("image0").unwrap();
            info_layout.insert("image".to_string(), image0);
        }
        info.insert("type".to_string(), Value::String("transliteration".to_string()));
    }
    // v3 no longer stores the Normal/AltGr instructional images for keyed layouts.
    info_layout.remove("image0");
    info_layout.remove("image1");

    info.insert("layout".to_string(), Value::Object(info_layout));
    info.insert("version".to_string(), Value::String("3".to_string()));
    layout.insert("info".to_string(), Value::Object(info));

    save_layout(layout, save_path)
}

/// Save a layout object as JSON to `path`.
fn save_layout(layout: Value, path: &PathBuf) -> ConversionResult {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(&layout).unwrap_or_default();
    match std::fs::write(path, json) {
        Ok(()) => ConversionResult::Ok,
        Err(_) => ConversionResult::SaveError,
    }
}

/// Open and convert a layout file at `path`. Handles v1, v2, v3, and Avro XML.
pub fn convert_layout_format_file(path: &PathBuf) -> ConversionResult {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return ConversionResult::OpenError,
    };

    // Detect Avro XML layout
    if text.trim_start().starts_with("<?xml") || text.trim_start().starts_with("<Avro") {
        let save_path = user_save_path(path);
        return convert_avro_layout(&text, &save_path);
    }

    let layout: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(_) => return ConversionResult::OpenError,
    };

    let version = layout
        .get("info")
        .and_then(|i| i.get("version"))
        .and_then(Value::as_str)
        .unwrap_or("");

    let save_path = user_save_path(path);

    match version {
        "1" => convert_layout_format(layout, &save_path),
        "2" => convert_to_v3(layout, &save_path),
        "3" => {
            // v3: just copy to user data dir
            save_layout(layout, &save_path)
        }
        _ => ConversionResult::UnsupportedLayout,
    }
}

/// Determine the user save path for a converted layout.
fn user_save_path(path: &PathBuf) -> PathBuf {
    let user_data_dir = shanti_paths::user_data_dir();
    let layouts_dir = user_data_dir.join("layouts");
    let _ = std::fs::create_dir_all(&layouts_dir);
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("layout");
    layouts_dir.join(file_name)
}

pub struct LayoutConverter;

impl LayoutConverter {
    pub fn convert_layout_format(path: &PathBuf) -> ConversionResult {
        convert_layout_format_file(path)
    }
}

impl Default for LayoutConverter {
    fn default() -> Self {
        Self
    }
}
