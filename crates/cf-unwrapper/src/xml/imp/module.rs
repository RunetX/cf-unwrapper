//! Модули `Ext/<Роль>.bsl` → запись `<uuid>.<суффикс>`: вложенный контейнер `info` + `text`.
//! `text` — файл как есть (BOM и CRLF); `info` — `{3,1,0,"",0}` (у модулей без исходного текста, `.bin`, — `{3,2,…}`).

use super::{Rec, BOM};

pub fn record(bsl: &[u8]) -> Result<Rec, String> {
    if !bsl.starts_with(BOM) {
        // как платформа пишет модуль без BOM, не проверено
        return Err("модуль без BOM".into());
    }
    std::str::from_utf8(bsl).map_err(|_| "модуль не в UTF-8")?;
    Ok(Rec::Dir(vec![("info".into(), [BOM, b"{3,1,0,\"\",0}"].concat()), ("text".into(), bsl.to_vec())]))
}
