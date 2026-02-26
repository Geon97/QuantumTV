use crate::db::db_client::Db;
use tauri::State;

fn decode_json_bytes(data: &[u8]) -> Result<String, String> {
    let text = String::from_utf8(data.to_vec()).map_err(|_| "Invalid UTF-8 payload".to_string())?;
    let trimmed = text.trim();

    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return Ok(text);
    }

    let bytes = trimmed
        .split(',')
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .map(|part| {
            part.parse::<u8>()
                .map_err(|_| "Invalid byte sequence".to_string())
        })
        .collect::<Result<Vec<u8>, String>>()?;

    String::from_utf8(bytes).map_err(|_| "Invalid UTF-8 payload".to_string())
}

#[tauri::command]
pub fn export_json(db: State<'_, Db>) -> Result<Vec<u8>, String> {
    db.export_json()
}

#[tauri::command]
pub fn import_json(db: State<'_, Db>, data: String) -> Result<(), String> {
    db.import_json(data)
}

#[tauri::command]
pub fn import_json_bytes(db: State<'_, Db>, data: Vec<u8>) -> Result<(), String> {
    let decoded = decode_json_bytes(&data)?;
    db.import_json(decoded)
}

#[tauri::command]
pub fn clear_cache(db: State<'_, Db>) -> Result<(), String> {
    db.clear_cache()
}

#[cfg(test)]
mod tests {
    use super::decode_json_bytes;

    #[test]
    fn decode_json_bytes_accepts_utf8() {
        let payload = br#"{"ok":true}"#.to_vec();
        let decoded = decode_json_bytes(&payload).unwrap();
        assert_eq!(decoded, r#"{"ok":true}"#);
    }

    #[test]
    fn decode_json_bytes_accepts_numeric_payload() {
        let payload = b"123,34,111,107,34,58,116,114,117,101,125".to_vec();
        let decoded = decode_json_bytes(&payload).unwrap();
        assert_eq!(decoded, r#"{"ok":true}"#);
    }

    #[test]
    fn decode_json_bytes_rejects_invalid_utf8() {
        let payload = vec![0xff, 0xfe, 0xfd];
        assert!(decode_json_bytes(&payload).is_err());
    }
}
