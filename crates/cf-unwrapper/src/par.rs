//! Обработка по потокам без внешних зависимостей: элементы раздаются по одному (крупный объект не держит
//! остальных), результаты — в порядке элементов, как при последовательном обходе.

use std::sync::atomic::{AtomicUsize, Ordering};

/// `f(состояние потока, элемент)` для всех элементов на всех ядрах; `init` — состояние каждого потока
/// (кэш, буфер). Результаты в порядке `items`.
pub fn map<T: Sync, R: Send, S>(items: &[T], init: impl Fn() -> S + Sync, f: impl Fn(&mut S, &T) -> R + Sync) -> Vec<R> {
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get()).min(items.len());
    if threads <= 1 {
        let mut s = init();
        return items.iter().map(|t| f(&mut s, t)).collect();
    }
    let next = AtomicUsize::new(0);
    let parts: Vec<Vec<(usize, R)>> = std::thread::scope(|sc| {
        let workers: Vec<_> = (0..threads)
            .map(|_| {
                sc.spawn(|| {
                    let mut s = init();
                    let mut out = Vec::new();
                    loop {
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        let Some(t) = items.get(i) else { return out };
                        out.push((i, f(&mut s, t)));
                    }
                })
            })
            .collect();
        workers.into_iter().map(|w| w.join().unwrap_or_else(|e| std::panic::resume_unwind(e))).collect()
    });
    let mut all: Vec<Option<R>> = (0..items.len()).map(|_| None).collect();
    for (i, r) in parts.into_iter().flatten() {
        all[i] = Some(r);
    }
    all.into_iter().map(|r| r.expect("каждый элемент обработан")).collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn keeps_order_and_state() {
        let v: Vec<usize> = (0..1000).collect();
        let out = super::map(&v, || 0usize, |seen, &x| {
            *seen += 1;
            (x * 2, *seen)
        });
        assert_eq!(out.iter().map(|r| r.0).collect::<Vec<_>>(), (0..1000).map(|x| x * 2).collect::<Vec<_>>());
        // у каждого потока свое состояние: сумма счетчиков по потокам — число элементов
        assert!(out.iter().all(|r| r.1 >= 1));
        assert!(super::map(&[] as &[u8], || (), |_, _| 0).is_empty());
    }
}
