//! Страничный контейнер 1С (формат .cf/.cfe/.epf/.hbk).
//!
//! Восстановлено по файлам (см. docs/reverse-engineering.md):
//! - заголовок 32-бит: `free:u32=0x7fffffff, page:u32, count:u32, reserved:u32`;
//!   64-бит: `free:u64=!0, page:u32, count:u32, reserved:u32`;
//! - блок: `\r\n{size} {page} {next} \r\n` + данные страницы, `%08x` или `%016X`;
//! - первый блок (смещение 20) — оглавление из троек `(header, data, END)`;
//! - заголовок записи: `created:u64, modified:u64, reserved:u32, name:UTF-16LE`;
//! - данные верхнего уровня сжаты raw deflate, во вложенных контейнерах — нет.

use std::io::{Read, Write};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Width {
    W32,
    W64,
}

impl Width {
    fn end(self) -> u64 {
        match self {
            Width::W32 => 0x7fff_ffff,
            Width::W64 => u64::MAX,
        }
    }
    fn header_len(self) -> usize {
        match self {
            Width::W32 => 31,
            Width::W64 => 55,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub name: String,
    pub created: u64,
    pub modified: u64,
    /// Байты как в файле (на верхнем уровне — сжатые).
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct Container {
    pub width: Width,
    pub entries: Vec<Entry>,
    /// Конец последнего блока: за ним может идти следующий контейнер.
    pub end: usize,
}

#[derive(Debug)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}

fn err<T>(m: impl Into<String>) -> Result<T, Error> {
    Err(Error(m.into()))
}

pub fn is_container(b: &[u8]) -> bool {
    detect(b).is_some()
}

/// Первое поле заголовка — первая свободная страница (`0x7fffffff`, если её нет; у .hbk бывает число),
/// поэтому формат узнаем по заголовку блока оглавления.
fn detect(b: &[u8]) -> Option<Width> {
    let block = |at: usize, digits: usize| {
        b.len() >= at + 2 + 3 * (digits + 1) + 2
            && &b[at..at + 2] == b"\r\n"
            && b[at + 2..at + 2 + digits].iter().all(u8::is_ascii_hexdigit)
            && b[at + 2 + digits] == b' '
    };
    if block(16, 8) {
        Some(Width::W32)
    } else if block(20, 16) {
        Some(Width::W64)
    } else {
        None
    }
}

struct Reader<'a> {
    b: &'a [u8],
    base: usize,
    w: Width,
    end: usize,
}

impl Reader<'_> {
    fn header(&self, off: u64) -> Result<[u64; 3], Error> {
        let at = self.base.checked_add(off as usize).filter(|&a| a + self.w.header_len() <= self.b.len());
        let Some(at) = at else { return err(format!("блок за концом файла: {off:#x}")) };
        let h = &self.b[at..at + self.w.header_len()];
        if &h[..2] != b"\r\n" || &h[h.len() - 2..] != b"\r\n" {
            return err(format!("нет заголовка блока по смещению {:#x}", at));
        }
        let txt = std::str::from_utf8(&h[2..h.len() - 2]).map_err(|_| Error("заголовок блока не ASCII".into()))?;
        let mut v = [0u64; 3];
        for (i, p) in txt.split_whitespace().take(3).enumerate() {
            v[i] = u64::from_str_radix(p, 16).map_err(|_| Error(format!("плохое число в заголовке блока: {p}")))?;
        }
        Ok(v)
    }

    fn block(&mut self, mut off: u64) -> Result<Vec<u8>, Error> {
        let [size, mut page, mut next] = self.header(off)?;
        let size = size as usize;
        let mut out = Vec::with_capacity(size.min(1 << 28));
        loop {
            let start = self.base + off as usize + self.w.header_len();
            let take = (page as usize).min(size - out.len());
            if start + take > self.b.len() {
                return err("данные блока за концом файла");
            }
            out.extend_from_slice(&self.b[start..start + take]);
            self.end = self.end.max(start + page as usize);
            if next == self.w.end() || out.len() >= size {
                return Ok(out);
            }
            off = next;
            let h = self.header(off)?;
            page = h[1];
            next = h[2];
        }
    }
}

/// Читает контейнер, начинающийся по смещению `base`.
pub fn read_at(b: &[u8], base: usize) -> Result<Container, Error> {
    let Some(w) = detect(&b[base..]) else { return err("это не контейнер 1С") };
    let mut r = Reader { b, base, w, end: base };
    let toc = r.block(if w == Width::W32 { 16 } else { 20 })?;
    let rec = if w == Width::W32 { 12 } else { 24 };
    let mut entries = Vec::new();
    for t in toc.chunks_exact(rec) {
        let (ha, da) = match w {
            Width::W32 => (u32::from_le_bytes(t[0..4].try_into().unwrap()) as u64, u32::from_le_bytes(t[4..8].try_into().unwrap()) as u64),
            Width::W64 => (u64::from_le_bytes(t[0..8].try_into().unwrap()), u64::from_le_bytes(t[8..16].try_into().unwrap())),
        };
        if ha == w.end() {
            continue;
        }
        let h = r.block(ha)?;
        if h.len() < 20 {
            return err("короткий заголовок записи");
        }
        let units: Vec<u16> = h[20..].chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).take_while(|&u| u != 0).collect();
        let data = if da == w.end() { Vec::new() } else { r.block(da)? };
        entries.push(Entry {
            name: String::from_utf16_lossy(&units),
            created: u64::from_le_bytes(h[0..8].try_into().unwrap()),
            modified: u64::from_le_bytes(h[8..16].try_into().unwrap()),
            data,
        });
    }
    Ok(Container { width: w, entries, end: r.end })
}

/// Читает все контейнеры файла подряд (у крупных .cf за 32-битным идёт 64-битный).
pub fn read_all(b: &[u8]) -> Result<Vec<Container>, Error> {
    let mut parts = vec![read_at(b, 0)?];
    while let Some(next) = parts.last().map(|c| c.end) {
        if next >= b.len() || detect(&b[next..]).is_none() {
            break;
        }
        parts.push(read_at(b, next)?);
    }
    Ok(parts)
}

pub fn inflate(data: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    flate2::read::DeflateDecoder::new(data).read_to_end(&mut out).ok()?;
    Some(out)
}

pub fn deflate(data: &[u8]) -> Vec<u8> {
    let mut e = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
    e.write_all(data).expect("запись в память");
    e.finish().expect("запись в память")
}

/// Данные записи верхнего уровня: распакованные, если сжаты.
pub fn content(e: &Entry) -> Vec<u8> {
    inflate(&e.data).unwrap_or_else(|| e.data.clone())
}

fn put_block(out: &mut Vec<u8>, w: Width, data: &[u8], page: usize) {
    let page = page.max(data.len());
    match w {
        Width::W32 => write!(out, "\r\n{:08x} {:08x} {:08x} \r\n", data.len(), page, 0x7fff_ffffu32),
        Width::W64 => write!(out, "\r\n{:016X} {:016X} {:016X} \r\n", data.len(), page, u64::MAX),
    }
    .unwrap();
    out.extend_from_slice(data);
    out.resize(out.len() + page - data.len(), 0);
}

/// Собирает контейнер. Ширина выбирается по размеру: свыше 2 ГБ — 64-битный.
pub fn write(entries: &[Entry]) -> Vec<u8> {
    let total: usize = entries.iter().map(|e| e.data.len() + 600).sum();
    write_as(entries, if total < 0x7000_0000 { Width::W32 } else { Width::W64 })
}

/// Собирает контейнер заданной ширины: часть .cf пишется той же ширины, что была (у БП 3.0 вторая часть 64-битная,
/// и 32-битную на ее месте платформа не читает — загружает пустую первую часть).
pub fn write_as(entries: &[Entry], w: Width) -> Vec<u8> {
    let total: usize = entries.iter().map(|e| e.data.len() + 600).sum();
    let (hdr, rec, hl) = match w {
        Width::W32 => (16, 12, 31),
        Width::W64 => (20, 24, 55),
    };
    let toc_len = (entries.len() * rec).max(1);
    // у 64-битной части .cf платформы страница оглавления — 64 КБ (каркас, БП); с меньшей платформа часть не читает
    let toc_page = toc_len.max(if w == Width::W64 { 0x10000 } else { 512 });
    let mut out = Vec::with_capacity(total);
    match w {
        Width::W32 => {
            out.extend_from_slice(&0x7fff_ffffu32.to_le_bytes());
            out.extend_from_slice(&512u32.to_le_bytes());
            out.extend_from_slice(&(entries.len() as u32).to_le_bytes());
            out.extend_from_slice(&0u32.to_le_bytes());
        }
        Width::W64 => {
            out.extend_from_slice(&u64::MAX.to_le_bytes());
            out.extend_from_slice(&512u32.to_le_bytes());
            out.extend_from_slice(&(entries.len() as u32).to_le_bytes());
            out.extend_from_slice(&0u32.to_le_bytes());
        }
    }
    debug_assert_eq!(out.len(), hdr);
    let mut body = Vec::new();
    let mut toc = Vec::with_capacity(toc_len);
    let mut pos = hdr + hl + toc_page;
    for e in entries {
        let mut h = Vec::with_capacity(20 + e.name.len() * 2 + 4);
        h.extend_from_slice(&e.created.to_le_bytes());
        h.extend_from_slice(&e.modified.to_le_bytes());
        h.extend_from_slice(&0u32.to_le_bytes());
        for u in e.name.encode_utf16() {
            h.extend_from_slice(&u.to_le_bytes());
        }
        h.extend_from_slice(&[0, 0, 0, 0]);
        let ha = pos;
        put_block(&mut body, w, &h, h.len());
        pos = hdr + hl + toc_page + body.len();
        let da = pos;
        put_block(&mut body, w, &e.data, 512);
        pos = hdr + hl + toc_page + body.len();
        match w {
            Width::W32 => {
                for v in [ha as u32, da as u32, 0x7fff_ffff] {
                    toc.extend_from_slice(&v.to_le_bytes());
                }
            }
            Width::W64 => {
                for v in [ha as u64, da as u64, u64::MAX] {
                    toc.extend_from_slice(&v.to_le_bytes());
                }
            }
        }
    }
    put_block(&mut out, w, &toc, toc_page);
    out.extend_from_slice(&body);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_nested() {
        let inner = write(&[Entry { name: "text".into(), created: 1, modified: 2, data: "Процедура А() КонецПроцедуры".as_bytes().to_vec() }]);
        let outer = write(&[
            Entry { name: "root".into(), created: 3, modified: 4, data: deflate(b"{2,x,}") },
            Entry { name: "m.0".into(), created: 5, modified: 6, data: deflate(&inner) },
        ]);
        let c = read_at(&outer, 0).unwrap();
        assert_eq!(c.width, Width::W32);
        assert_eq!(c.entries.len(), 2);
        assert_eq!(content(&c.entries[0]), b"{2,x,}");
        assert_eq!((c.entries[1].created, c.entries[1].modified), (5, 6));
        let n = read_at(&content(&c.entries[1]), 0).unwrap();
        assert_eq!(n.entries[0].name, "text");
        assert_eq!(String::from_utf8(n.entries[0].data.clone()).unwrap(), "Процедура А() КонецПроцедуры");
        assert_eq!(c.end, outer.len());
    }

    #[test]
    fn chained_containers() {
        let mut b = write(&[Entry { name: "a".into(), created: 0, modified: 0, data: vec![1; 700] }]);
        b.extend(write(&[Entry { name: "b".into(), created: 0, modified: 0, data: vec![2; 10] }]));
        let parts = read_all(&b).unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[1].entries[0].name, "b");
    }

    #[test]
    fn width_kept() {
        let e = [Entry { name: "a".into(), created: 1, modified: 2, data: vec![7; 1000] }];
        let c = read_at(&write_as(&e, Width::W64), 0).unwrap();
        assert_eq!((c.width, c.entries[0].data.len(), c.entries[0].modified), (Width::W64, 1000, 2));
        assert_eq!(read_at(&write_as(&e, Width::W32), 0).unwrap().width, Width::W32);
    }

    #[test]
    fn rejects_garbage() {
        assert!(read_at(b"not a container at all, just some text here.......", 0).is_err());
    }
}
