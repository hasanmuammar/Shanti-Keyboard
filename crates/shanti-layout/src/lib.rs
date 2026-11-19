/*
 *  Shanti layout model — Rust port of src/frontend/Layout.{h,cpp} and the
 *  image half of LayoutViewer::decodeAndDecompress (Phase 3, dialogs/layout
 *  viewing).
 *
 *  Parsing mirrors QJsonValue's accessors exactly: `toString()` yields ""
 *  for non-strings, `toInt()` yields 0 for non-numbers (so a JSON *string*
 *  "3" records 0), and `isUndefined()` — not `isString()` — decides whether
 *  an image key is present. The shipped layouts under `data/` are the golden
 *  fixtures; their values were read directly, never from this code.
 *
 *  `info.type` plus the reserved name "Khipro" select the kind, exactly as
 *  Layout::loadDesc() does.
 */

use std::collections::BTreeMap;

use serde_json::Value;

/// How LayoutViewer renders a layout: fixed layouts paint a keyboard from the
/// key map; phonetic/khipro layouts show the embedded instructional image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutKind {
    Fixed,
    Phonetic,
    Khipro,
}

/// `LayoutDesc` — everything AboutFile shows and LayoutViewer branches on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutDesc {
    pub kind: LayoutKind,
    pub name: String,
    pub ver: String,
    /// `info.version` via Qt's `QJsonValue::toInt()`. Dead data in the Qt
    /// frontend (assigned, never read); reproduced for parity.
    pub file_ver: i64,
    /// Raw base91 text of the embedded image, "" when absent.
    pub image: String,
    pub dev_name: String,
    pub dev_comment: String,
}

/// A parsed layout file: its description plus the untouched document, so the
/// key map is produced on demand like Qt's `getLayoutMap()`.
pub struct LayoutFile {
    desc: LayoutDesc,
    doc: Value,
}

impl LayoutFile {
    pub fn parse(text: &str) -> Result<Self, serde_json::Error> {
        let doc: Value = serde_json::from_str(text)?;
        Ok(Self {
            desc: Self::describe(&doc),
            doc,
        })
    }

    fn describe(doc: &Value) -> LayoutDesc {
        let info = doc.get("info").and_then(Value::as_object);
        let layout = info.and_then(|i| i.get("layout"));
        let get = |key: &str| -> String {
            layout
                .and_then(|l| l.get(key))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        };

        // QString type = info["type"].toString(); QString name = info["layout"]["name"].toString();
        let type_name = info
            .and_then(|i| i.get("type"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let name = get("name");
        let kind = if type_name == "transliteration" {
            if name == "Khipro" {
                LayoutKind::Khipro
            } else {
                LayoutKind::Phonetic
            }
        } else {
            LayoutKind::Fixed
        };

        // QJsonValue::toInt(): only a Double converts; strings yield 0.
        let file_ver = match info.and_then(|i| i.get("version")) {
            Some(Value::Number(n)) => n
                .as_i64()
                .or_else(|| n.as_f64().map(|f| f as i64))
                .unwrap_or(0),
            _ => 0,
        };

        // Present-but-non-string keys take the branch and produce "" (no
        // image0 fallback), matching the `!isUndefined()` test in C++.
        let image = match layout.and_then(|l| l.get("image")) {
            Some(value) => value.as_str().unwrap_or("").to_string(),
            None => match layout.and_then(|l| l.get("image0")) {
                Some(value) => value.as_str().unwrap_or("").to_string(),
                None => String::new(),
            },
        };

        let developer = layout.and_then(|l| l.get("developer"));
        let dev = |key: &str| -> String {
            developer
                .and_then(|d| d.get(key))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        };

        LayoutDesc {
            kind,
            name,
            ver: get("version"),
            file_ver,
            image,
            dev_name: dev("name"),
            dev_comment: dev("comment"),
        }
    }

    pub fn desc(&self) -> &LayoutDesc {
        &self.desc
    }

    /// Flat `Key_<name>_Normal|AltGr -> glyph` map. Only fixed layouts have
    /// one, so Qt returns an empty map for every other kind.
    pub fn keymap(&self) -> BTreeMap<String, String> {
        let mut map = BTreeMap::new();
        if self.desc.kind != LayoutKind::Fixed {
            return map;
        }
        if let Some(entries) = self.doc.get("layout").and_then(Value::as_object) {
            for (key, value) in entries {
                // QJsonValue::toString(): non-strings become "".
                map.insert(key.clone(), value.as_str().unwrap_or("").to_string());
            }
        }
        map
    }
}

/// Custom basE91 decode table (rlyeh's modification) from src/frontend/base.hpp.
/// Index 91 marks "ignore this byte".
const DECTAB: [u8; 256] = [
    // 000..015
    91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, // 016..031
    91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, // 032..047
    91, 62, 91, 63, 64, 65, 66, 90, 67, 68, 69, 70, 71, 76, 72, 73, // 048..063
    52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 74, 75, 91, 77, 91, 79, // 064..079
    80, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, // 080..095
    15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 81, 78, 82, 83, 84, // 096..111
    85, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, // 112..127
    41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 86, 87, 88, 89, 91, // 128..143
    91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, // 144..159
    91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, // 160..175
    91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, // 176..191
    91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, // 192..207
    91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, // 208..223
    91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, // 224..239
    91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, // 240..255
    91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91, 91,
];

/// Port of `base<91>::decode` in src/frontend/base.hpp.
fn base91_decode(text: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len());
    let mut queue: u64 = 0;
    let mut nbits: u32 = 0;
    let mut val: i64 = -1;

    for &byte in text {
        let d = DECTAB[byte as usize] as i64;
        if d == 91 {
            continue; // ignore non-alphabet chars
        }
        if val == -1 {
            val = d;
        } else {
            val += d * 91;
            queue |= (val as u64) << nbits;
            nbits += if (val & 8191) > 88 { 13 } else { 14 };
            loop {
                out.push((queue & 0xff) as u8);
                queue >>= 8;
                nbits -= 8;
                if nbits <= 7 {
                    break;
                }
            }
            val = -1;
        }
    }

    if val != -1 {
        out.push((queue | ((val as u64) << nbits)) as u8);
    }
    out
}

/// `LayoutViewer::decodeAndDecompress`: base91 decode then ZSTD decompress.
///
/// Qt logs and falls back to the base91-decoded bytes whenever the frame
/// reports an error or an unknown content size; this port falls back on any
/// decompression failure, which is identical for every shipped layout image
/// (they all carry a known frame size) and additionally handles a
/// size-less frame Qt would have refused to show.
pub fn decode_image(image: &str) -> Vec<u8> {
    let decoded = base91_decode(image.as_bytes());
    match zstd::stream::decode_all(&decoded[..]) {
        Ok(bytes) => bytes,
        Err(_) => decoded,
    }
}
