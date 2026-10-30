//! Порядок обхода хэш-таблицы платформы (как `std::unordered_map` MSVC): записи, которые платформа строит из такой
//! таблицы (состав плана обмена), хранят элементы в этом порядке. Модель сверена на составе плана обмена
//! `АвтономнаяРабота` БП (1 693 элемента — совпал весь порядок):
//! - корзины — степень двойки (начало 8), корзина = хэш & (корзин − 1);
//! - новый элемент: пустая корзина — в конец списка, иначе — перед первым элементом своей корзины;
//! - после вставки, если элементов больше корзин, — перестройка: корзин ×8 (пока меньше 512), дальше — наименьшая
//!   степень двойки не меньше числа элементов; элементы проходятся по списку, первый в корзине остается на месте,
//!   следующие встают перед первым элементом своей корзины.

/// Элементы `items` (в порядке вставки) в порядке обхода таблицы.
pub fn msvc_order<T: Clone>(items: &[T], hash: impl Fn(&T) -> u64) -> Vec<T> {
    let mut b: u64 = 8;
    let mut list: Vec<T> = Vec::with_capacity(items.len());
    for it in items {
        let h = hash(it) & (b - 1);
        match list.iter().position(|x| hash(x) & (b - 1) == h) {
            Some(first) => list.insert(first, it.clone()),
            None => list.push(it.clone()),
        }
        if list.len() as u64 > b {
            let n = list.len() as u64;
            let req = n.next_power_of_two().max(b);
            b = if b < 512 && b * 8 >= req { b * 8 } else { req };
            let mut out: Vec<T> = Vec::with_capacity(list.len());
            for x in list {
                let hx = hash(&x) & (b - 1);
                match out.iter().position(|y| hash(y) & (b - 1) == hx) {
                    Some(first) => out.insert(first, x),
                    None => out.push(x),
                }
            }
            list = out;
        }
    }
    list
}

/// Порядок обхода `boost::unordered_map` платформы (core85.dll 8.5.1.1529, xml-mxl.md «Порядок пар Соответствия»):
/// 16 корзин, коэффициент 1.0, число корзин — степень двойки не меньше 4; корзина — Wang-перемешивание 64 бит
/// от `hash` & (корзин − 1); ключ в пустую корзину — в начало списка, в занятую — в начало ее группы; при
/// `размер + 1 > корзин` — перехеширование на `min(размер + max(размер/2, 1))` (обход списка: в пустую корзину —
/// на место, в занятую — в начало ее группы). Ключи `items` различны, вставка — по порядку.
pub fn boost_order<T: Clone>(items: &[T], hash: impl Fn(&T) -> u64) -> Vec<T> {
    let mix = |mut k: u64| {
        k = k.wrapping_mul(0x200000).wrapping_add(!k);
        k = ((k >> 24) ^ k).wrapping_mul(0x109);
        k = ((k >> 14) ^ k).wrapping_mul(0x15);
        ((k >> 28) ^ k).wrapping_mul(0x8000_0001)
    };
    let hs: Vec<u64> = items.iter().map(|x| mix(hash(x))).collect();
    let min_buckets = |n: usize| (n + 1).max(4).next_power_of_two() as u64;
    let mut nb = 16u64;
    let mut list: Vec<usize> = Vec::with_capacity(items.len());
    for i in 0..items.len() {
        let size = list.len();
        if size + 1 > nb as usize {
            let want = min_buckets((size + (size / 2).max(1)).max(size + 1));
            if want != nb {
                nb = want;
                let mut out: Vec<usize> = Vec::with_capacity(size);
                for j in list {
                    match out.iter().position(|&x| hs[x] & (nb - 1) == hs[j] & (nb - 1)) {
                        Some(p) => out.insert(p, j),
                        None => out.push(j),
                    }
                }
                list = out;
            }
        }
        match list.iter().position(|&x| hs[x] & (nb - 1) == hs[i] & (nb - 1)) {
            Some(p) => list.insert(p, i),
            None => list.insert(0, i),
        }
    }
    list.into_iter().map(|i| items[i].clone()).collect()
}

/// Хэш GUID у платформы — первое 32-битное поле (Data1), то есть первые 8 шестнадцатеричных цифр.
pub fn guid_hash(g: &str) -> u64 {
    u64::from_str_radix(g.get(..8).unwrap_or("0"), 16).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boost_regs_bu() {
        // тот же случай, что xml::mxl map_order_regs_bu: строковые ключи, hashCode — h·37 + код UTF-16 (знаковый)
        let h = |s: &&str| s.encode_utf16().fold(0u32, |h, c| h.wrapping_mul(37).wrapping_add(c as u32)) as i32 as i64 as u64;
        let rec = ["КорСчет", "Счет", "ИмяОбъекта", "СчетКт", "Отбор", "СчетПредставление", "СчетДт"];
        assert_eq!(boost_order(&rec, h), ["СчетДт", "СчетПредставление", "Отбор", "ИмяОбъекта", "СчетКт", "Счет", "КорСчет"]);
        // 40 ключей — с перехешированием, все на месте
        let many: Vec<u64> = (0..40).collect();
        let mut o = boost_order(&many, |x| *x);
        o.sort();
        assert_eq!(o, many);
    }

    #[test]
    fn buckets_and_rehash() {
        // 3 элемента одной корзины (младшие биты 1) и один другой: новый в непустой корзине встает в ее начало
        let v = msvc_order(&[1u64, 9, 2, 17], |x| *x);
        assert_eq!(v, [17, 9, 1, 2]);
        // девятый элемент вызывает перестройку на 64 корзины: 1 и 65 остаются вместе, 9 и 17 расходятся
        let items: Vec<u64> = vec![1, 9, 17, 65, 2, 3, 4, 5, 6];
        let v = msvc_order(&items, |x| *x);
        assert_eq!(v.len(), 9);
        let p = |x: u64| v.iter().position(|y| *y == x).unwrap();
        assert_eq!(p(1) + 1, p(65));
    }
}
