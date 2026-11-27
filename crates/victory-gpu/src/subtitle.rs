//! Canlı Türkçe altyazı: sansürsüz GTA sokak dili çeviri çekirdeği,
//! kalıcı önbellek (`tr_subtitles_cache.json`), arka plan çeviri işçisi,
//! ekran alt bandı OCR okuyucu ([`ocr`]) ve D3D12 bindirme katmanı ([`overlay`]).
//!
//! Küfür ve argo **olduğu gibi** Türkçe sokak karşılığına gider; yıldızlama
//! (`***`) ve yumuşatma yok. `tr_subtitles_cache.json` elle düzenlenebilir:
//! var olan satırlar bir daha çevrilmez, 0 ms gecikmeyle gösterilir ve diskte
//! güncellenirse oyun açıkken yeniden yüklenir.

pub(crate) mod ocr;
pub(crate) mod overlay;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime};

/// Where the cache lives: beside the process (working directory); the user
/// can hand-edit it. `VICTORY_SUBS_CACHE` moves it.
pub(crate) fn cache_path() -> PathBuf {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        std::env::var_os("VICTORY_SUBS_CACHE")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("tr_subtitles_cache.json"))
    })
    .clone()
}

/// Whether the live subtitles start enabled (`VICTORY_SUBS=1`; F8 can always
/// toggle them at runtime while our window has focus).
pub(crate) fn asked() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| crate::switch::on("VICTORY_SUBS"))
}

/// Persistent English -> Turkish cache with exact and case-folded 0 ms lookup.
struct Cache {
    map: HashMap<String, String>,
    lower: HashMap<String, String>,
    mtime: Option<SystemTime>,
    last_check: Instant,
}

impl Cache {
    fn load() -> Self {
        let path = cache_path();
        let mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        let map = load_cache_file(&path);
        let lower = map.iter().map(|(k, v)| (normalize_key(k), v.clone())).collect();
        Self { map, lower, mtime, last_check: Instant::now() }
    }

    fn reload_if_changed(&mut self) {
        if self.last_check.elapsed() < Duration::from_secs(2) {
            return;
        }
        self.last_check = Instant::now();
        let path = cache_path();
        let mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        if mtime.is_some() && mtime != self.mtime {
            let map = load_cache_file(&path);
            if !map.is_empty() || std::fs::metadata(&path).is_ok_and(|m| m.len() <= 4) {
                self.lower = map.iter().map(|(k, v)| (normalize_key(k), v.clone())).collect();
                self.map = map;
                self.mtime = mtime;
            }
        }
    }

    fn lookup(&self, en: &str) -> Option<String> {
        if let Some(hit) = self.map.get(en) {
            return Some(hit.clone());
        }
        let trimmed = en.trim();
        if let Some(hit) = self.map.get(trimmed) {
            return Some(hit.clone());
        }
        self.lower.get(&normalize_key(trimmed)).cloned()
    }
}

fn normalize_key(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ").to_ascii_lowercase()
}

static CACHE: Mutex<Option<Cache>> = Mutex::new(None);

fn cache() -> std::sync::MutexGuard<'static, Option<Cache>> {
    CACHE.lock().unwrap_or_else(|e| e.into_inner())
}

/// Ensures `tr_subtitles_cache.json` exists on disk so the user can find and
/// hand-edit it, without ever overwriting an existing file.
pub(crate) fn ensure_cache_file() {
    let path = cache_path();
    if !path.exists() {
        let _ = std::fs::write(&path, "{\n}\n");
    }
}

/// Look a line up in the persistent cache (0 ms; lazily loads
/// `tr_subtitles_cache.json` so user hand-edits hit immediately).
pub(crate) fn cached(en: &str) -> Option<String> {
    let mut guard = cache();
    let c = guard.get_or_insert_with(Cache::load);
    c.lookup(en)
}

/// Translate `en` (sansürsüz) and remember it in `tr_subtitles_cache.json`.
/// Called from the background translation thread only, never the render thread.
pub(crate) fn translate_and_remember(en: &str) -> String {
    let key = en.trim();
    if key.is_empty() {
        return String::new();
    }
    {
        let mut guard = cache();
        let c = guard.get_or_insert_with(Cache::load);
        c.reload_if_changed();
        if let Some(hit) = c.lookup(key) {
            return hit;
        }
    }
    let tr = translate(key);
    let mut guard = cache();
    let c = guard.get_or_insert_with(Cache::load);
    if !c.map.contains_key(key) {
        c.map.insert(key.to_owned(), tr.clone());
        c.lower.insert(normalize_key(key), tr.clone());
        let path = cache_path();
        write_cache_file(&path, &c.map);
        c.mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        c.last_check = Instant::now();
    }
    tr
}

/// Coalescing background translation queue: only the newest subtitle line is
/// kept waiting so the worker never falls behind rapid dialogue.
struct Translator {
    pending: Mutex<Option<String>>,
    wake: Condvar,
    ready: Mutex<Option<(String, String)>>,
}

/// Starts (once) and returns the static translation worker state.
fn worker() -> &'static Translator {
    static WORKER: OnceLock<&'static Translator> = OnceLock::new();
    WORKER.get_or_init(|| {
        let state: &'static Translator = Box::leak(Box::new(Translator {
            pending: Mutex::new(None),
            wake: Condvar::new(),
            ready: Mutex::new(None),
        }));
        let _ = std::thread::Builder::new()
            .name("victory-subs-tr".into())
            .spawn(move || {
                // SAFETY: lowering our own background thread's priority.
                unsafe {
                    use windows::Win32::System::Threading::{
                        GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_BELOW_NORMAL,
                    };
                    let _ = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_BELOW_NORMAL);
                }
                ensure_cache_file();
                let _ = cache().get_or_insert_with(Cache::load);
                loop {
                    let next = {
                        let mut guard = state.pending.lock().unwrap_or_else(|e| e.into_inner());
                        loop {
                            if let Some(line) = guard.take() {
                                break line;
                            }
                            guard = state.wake.wait(guard).unwrap_or_else(|e| e.into_inner());
                        }
                    };
                    let tr = translate_and_remember(&next);
                    *state.ready.lock().unwrap_or_else(|e| e.into_inner()) = Some((next, tr));
                }
            });
        state
    })
}

/// Enqueues `en` for background uncensored translation (non-blocking, coalescing).
pub(crate) fn ask_to_translate(en: &str) {
    let w = worker();
    *w.pending.lock().unwrap_or_else(|e| e.into_inner()) = Some(en.to_owned());
    w.wake.notify_one();
}

/// Returns the completed Turkish translation if the worker has finished `en`.
pub(crate) fn take_translated(en: &str) -> Option<String> {
    let w = worker();
    let mut ready = w.ready.lock().unwrap_or_else(|e| e.into_inner());
    if ready.as_ref().is_some_and(|(k, _)| k == en) {
        ready.take().map(|(_, v)| v)
    } else {
        None
    }
}

/// `tr_subtitles_cache.json` as `HashMap<String, String>`: a flat
/// `{"english": "türkçe"}` object. Parse errors give an empty map (the game
/// runs on; the next write fixes the file).
fn load_cache_file(path: &std::path::Path) -> HashMap<String, String> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return HashMap::new();
    };
    parse_json_object(&text).unwrap_or_default()
}

/// Writes the map back atomically (temp file + rename, fallback direct write),
/// best effort so a locked file never stalls or crashes the game.
fn write_cache_file(path: &std::path::Path, map: &HashMap<String, String>) {
    let mut out = String::from("{\n");
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();
    for (idx, key) in keys.iter().enumerate() {
        out.push_str("  ");
        push_json_string(&mut out, key);
        out.push_str(": ");
        push_json_string(&mut out, &map[*key]);
        if idx + 1 < keys.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("}\n");
    let tmp = path.with_extension("json.tmp");
    if std::fs::write(&tmp, &out).is_ok() {
        if std::fs::rename(&tmp, path).is_err() {
            let _ = std::fs::write(path, &out);
            let _ = std::fs::remove_file(&tmp);
        }
    } else {
        let _ = std::fs::write(path, &out);
    }
}

/// A minimal JSON string-object reader for the cache file (the repo takes
/// no JSON dependency): one flat object of string -> string. Escapes
/// `\"`, `\n`, `\t` and `\uXXXX` (Turkish characters live there or raw UTF-8).
fn parse_json_object(text: &str) -> Option<HashMap<String, String>> {
    let bytes = text.strip_prefix('\u{feff}').unwrap_or(text).as_bytes();
    let (mut i, n) = (0, bytes.len());
    let skip_ws = |i: &mut usize| {
        while *i < n && bytes[*i].is_ascii_whitespace() {
            *i += 1;
        }
    };
    skip_ws(&mut i);
    if bytes.get(i) != Some(&b'{') {
        return None;
    }
    i += 1;
    let mut map = HashMap::new();
    loop {
        skip_ws(&mut i);
        match bytes.get(i) {
            Some(&b'}') => {
                return Some(map);
            }
            Some(&b',') => {
                i += 1;
                continue;
            }
            Some(&b'"') => {}
            _ => return None,
        }
        i += 1;
        let key = read_json_string(bytes, &mut i)?;
        skip_ws(&mut i);
        if bytes.get(i) != Some(&b':') {
            return None;
        }
        i += 1;
        skip_ws(&mut i);
        if bytes.get(i) != Some(&b'"') {
            return None;
        }
        i += 1;
        let value = read_json_string(bytes, &mut i)?;
        map.insert(key, value);
    }
}

fn read_json_string(bytes: &[u8], i: &mut usize) -> Option<String> {
    let mut out = String::new();
    while *i < bytes.len() {
        match bytes[*i] {
            b'"' => {
                *i += 1;
                return Some(out);
            }
            b'\\' => {
                *i += 1;
                match bytes.get(*i) {
                    Some(&b'n') => out.push('\n'),
                    Some(&b't') => out.push('\t'),
                    Some(&b'r') => out.push('\r'),
                    Some(&b'"') => out.push('"'),
                    Some(&b'\\') => out.push('\\'),
                    Some(&b'/') => out.push('/'),
                    Some(&b'u') => {
                        let hex = std::str::from_utf8(bytes.get(*i + 1..*i + 5)?).ok()?;
                        let code = u32::from_str_radix(hex, 16).ok()?;
                        *i += 4;
                        if (0xD800..0xDC00).contains(&code)
                            && bytes.get(*i + 1) == Some(&b'\\')
                            && bytes.get(*i + 2) == Some(&b'u')
                            && let Some(low) = bytes.get(*i + 3..*i + 7).and_then(|s| std::str::from_utf8(s).ok())
                            && let Some(low) = u32::from_str_radix(low, 16).ok()
                        {
                            if let Some(c) = char::from_u32(
                                0x10000 + ((code - 0xD800) << 10) + (low - 0xDC00),
                            ) {
                                out.push(c);
                                *i += 6;
                            }
                        } else if let Some(c) = char::from_u32(code) {
                            out.push(c);
                        }
                    }
                    _ => return None,
                }
                *i += 1;
            }
            byte if byte < 0x80 => {
                out.push(byte as char);
                *i += 1;
            }
            _ => {
                let width = utf8_width(bytes[*i]);
                let slice = bytes.get(*i..*i + width)?;
                out.push_str(std::str::from_utf8(slice).ok()?);
                *i += width;
            }
        }
    }
    None
}

fn utf8_width(byte: u8) -> usize {
    if byte >= 0xF0 {
        4
    } else if byte >= 0xE0 {
        3
    } else if byte >= 0xC0 {
        2
    } else {
        1
    }
}

fn push_json_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Sansürsüz kural + kalıp + sözlük çevirisi: GTA üslubu, sokak dili. Küfür ve
/// argo olduğu gibi kalır — yıldızlama yok. Çok kelimeli deyimler ve küfür
/// kalıpları önce yakalanır; hikâye özel adları (Jason, Lucia, Vice City,
/// Leonida...) korunur; `tr_subtitles_cache.json` ile kullanıcı genişletir.
pub(crate) fn translate(en: &str) -> String {
    let raw_words: Vec<&str> = en.split_whitespace().collect();
    if raw_words.is_empty() {
        return String::new();
    }
    let starts_upper = raw_words[0]
        .chars()
        .find(|c| c.is_alphabetic())
        .is_some_and(|c| c.is_uppercase());

    // Split each token into (leading punct, bare word, trailing punct, lowercase bare).
    let tokens: Vec<(&str, &str, &str, String)> = raw_words
        .iter()
        .map(|&w| {
            let (h, b, t) = split_word(w);
            (h, b, t, b.to_ascii_lowercase())
        })
        .collect();

    let mut parts: Vec<String> = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        // 1. Try longest multi-word phrase match (4 down to 2 words) when
        //    internal tokens have no sentence-breaking punctuation.
        let mut matched = false;
        let max_span = (tokens.len() - i).min(4);
        for span in (2..=max_span).rev() {
            let inner_clean = (i..i + span - 1).all(|k| tokens[k].2.is_empty())
                && (i + 1..i + span).all(|k| tokens[k].0.is_empty());
            if !inner_clean {
                continue;
            }
            let joined = tokens[i..i + span]
                .iter()
                .map(|t| t.3.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            if let Ok(idx) = PHRASES.binary_search_by(|(k, _)| (*k).cmp(joined.as_str())) {
                let tr = PHRASES[idx].1;
                let head = tokens[i].0;
                let tail = tokens[i + span - 1].2;
                if !tr.is_empty() {
                    parts.push(format!("{head}{tr}{tail}"));
                } else if !head.is_empty() || !tail.is_empty() {
                    parts.push(format!("{head}{tail}"));
                }
                i += span;
                matched = true;
                break;
            }
        }
        if matched {
            continue;
        }

        // 2. Single-word lookup (case-insensitive on `DICT`).
        let (head, bare, tail, ref lower) = tokens[i];
        if bare.is_empty() {
            if !head.is_empty() || !tail.is_empty() {
                parts.push(format!("{head}{tail}"));
            }
            i += 1;
            continue;
        }
        match DICT.binary_search_by(|(k, _)| (*k).cmp(lower.as_str())) {
            Ok(idx) => {
                let tr = DICT[idx].1;
                if tr.is_empty() {
                    // Dropped article/copula ("the", "a", "an", "is", "are"):
                    // keep attached punctuation if any without leaving dangling spaces.
                    if !tail.is_empty() {
                        if let Some(last) = parts.last_mut() {
                            last.push_str(tail);
                        } else {
                            parts.push(tail.to_owned());
                        }
                    }
                } else {
                    parts.push(format!("{head}{tr}{tail}"));
                }
            }
            Err(_) => {
                // Unknown word (proper nouns, street/mission names): preserve as-is.
                parts.push(format!("{head}{bare}{tail}"));
            }
        }
        i += 1;
    }

    let mut out = parts.join(" ");
    if starts_upper {
        out = capitalize_first_tr(&out);
    }
    out.trim().to_owned()
}

/// Capitalizes the first alphabetic character using Turkish casing (`i -> İ`).
fn capitalize_first_tr(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    let mut done = false;
    for c in s.chars() {
        if !done && c.is_alphabetic() {
            done = true;
            match c {
                'i' => out.push('İ'),
                'ı' => out.push('I'),
                'ş' => out.push('Ş'),
                'ç' => out.push('Ç'),
                'ğ' => out.push('Ğ'),
                'ö' => out.push('Ö'),
                'ü' => out.push('Ü'),
                other => {
                    for u in other.to_uppercase() {
                        out.push(u);
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// A word into (punctuation head, bare word including inner apostrophe, punctuation tail).
fn split_word(word: &str) -> (&str, &str, &str) {
    let is_word_char = |c: char| c.is_alphanumeric() || c == '\'' || c == '\u{2019}';
    let start = word.find(|c: char| c.is_alphanumeric()).unwrap_or(word.len());
    let mut end = start;
    for (idx, c) in word[start..].char_indices() {
        if is_word_char(c) {
            end = start + idx + c.len_utf8();
        } else {
            break;
        }
    }
    // Trim trailing apostrophe if it was a closing quote (keep contractions like "don't", "y'all").
    while end > start && matches!(word[..end].chars().last(), Some('\'' | '\u{2019}')) {
        let ch = word[..end].chars().last().unwrap();
        end -= ch.len_utf8();
    }
    (&word[..start], &word[start..end], &word[end..])
}

/// Multi-word uncensored phrases, idioms and GTA VI story terms (sorted alphabetically).
static PHRASES: &[(&str, &str)] = &[
    ("all right", "tamam"),
    ("are you okay", "iyi misin"),
    ("back off", "geri bas"),
    ("be careful", "dikkatli ol"),
    ("calm down", "sakin ol"),
    ("come on", "hadi"),
    ("damn it", "lanet olsun"),
    ("don't move", "kıpırdama"),
    ("don't shoot", "ateş etme"),
    ("don't worry", "merak etme"),
    ("eat shit", "bok ye"),
    ("excuse me", "pardon"),
    ("follow me", "beni takip et"),
    ("for real", "cidden mi"),
    ("fuck it", "siktir et"),
    ("fuck off", "siktir git"),
    ("fuck that", "siktir et onu"),
    ("fuck this", "sikerim bunu"),
    ("fuck you", "siktir git"),
    ("full of shit", "palavracı"),
    ("get down", "yat yere"),
    ("get in", "bin içeri"),
    ("get out", "çık dışarı"),
    ("get the fuck", "siktir git"),
    ("get the fuck out", "siktir git buradan"),
    ("give me", "bana ver"),
    ("goddamn it", "lanet olsun"),
    ("good luck", "bol şans"),
    ("hands up", "eller yukarı"),
    ("hang on", "bekle"),
    ("hold on", "bekle"),
    ("holy shit", "hassiktir"),
    ("hurry up", "acele et"),
    ("i don't care", "umurumda değil"),
    ("i don't know", "bilmiyorum"),
    ("kick your ass", "kıçını tekmeleyeceğim"),
    ("kiss my ass", "kıçımı öp"),
    ("let me", "bırak"),
    ("let's go", "hadi gidelim"),
    ("look out", "dikkat et"),
    ("make sure", "emin ol"),
    ("mount kalaga", "Mount Kalaga"),
    ("my ass", "kıçım"),
    ("my bad", "benim hatam"),
    ("no problem", "sorun değil"),
    ("no shit", "hadi canım"),
    ("no way", "yok artık"),
    ("not yet", "henüz değil"),
    ("of course", "tabii ki"),
    ("over here", "buraya"),
    ("over there", "şurada"),
    ("piece of shit", "bok parçası"),
    ("piss off", "siktir git"),
    ("port gellhorn", "Port Gellhorn"),
    ("real dimez", "Real Dimez"),
    ("right now", "hemen şimdi"),
    ("screw you", "siktir git"),
    ("see you", "görüşürüz"),
    ("shut the fuck up", "siktir git kapa çeneni"),
    ("shut up", "kapa çeneni"),
    ("son of a bitch", "orospu çocuğu"),
    ("sons of bitches", "orospu çocukları"),
    ("stay back", "uzak dur"),
    ("stay here", "burada kal"),
    ("step back", "geri çekil"),
    ("suck my dick", "sikimi yala"),
    ("take care", "kendine iyi bak"),
    ("tell me", "söyle bana"),
    ("thank you", "teşekkürler"),
    ("that way", "o tarafa"),
    ("this way", "bu tarafa"),
    ("too late", "çok geç"),
    ("trust me", "bana güven"),
    ("vice beach", "Vice Beach"),
    ("vice city", "Vice City"),
    ("wake up", "uyan"),
    ("watch out", "dikkat et"),
    ("what happened", "ne oldu"),
    ("what the fuck", "ne siktir lan"),
    ("what's going on", "neler oluyor"),
    ("what's up", "ne haber"),
    ("you know", "biliyorsun"),
    ("you okay", "iyi misin"),
];

/// The word dictionary: strictly alphabetical by ASCII key (binary search).
/// Swears stay uncensored at full GTA street-Turkish strength — no stars, no softening.
static DICT: &[(&str, &str)] = &[
    ("a", ""),
    ("about", "hakkında"),
    ("after", "sonra"),
    ("again", "tekrar"),
    ("ain't", "değil"),
    ("all", "hepsi"),
    ("alright", "tamam"),
    ("always", "hep"),
    ("ambrosia", "Ambrosia"),
    ("an", ""),
    ("and", "ve"),
    ("anyway", "neyse"),
    ("are", ""),
    ("aren't", "değil"),
    ("around", "etrafta"),
    ("ass", "kıç"),
    ("asshole", "amcık"),
    ("assholes", "amcıklar"),
    ("baby", "bebeğim"),
    ("back", "geri"),
    ("bad", "kötü"),
    ("bag", "çanta"),
    ("bank", "banka"),
    ("bastard", "piç"),
    ("bastards", "piçler"),
    ("beach", "sahil"),
    ("because", "çünkü"),
    ("before", "önce"),
    ("behind", "arkada"),
    ("better", "daha iyi"),
    ("big", "büyük"),
    ("bitch", "orospu"),
    ("bitches", "orospular"),
    ("blood", "kan"),
    ("boat", "tekne"),
    ("boobie", "Boobie"),
    ("boss", "patron"),
    ("boy", "oğlum"),
    ("break", "kır"),
    ("brian", "Brian"),
    ("bring", "getir"),
    ("bro", "kardeş"),
    ("brother", "kardeş"),
    ("bullshit", "saçmalık"),
    ("business", "iş"),
    ("but", "ama"),
    ("buy", "satın al"),
    ("cal", "Cal"),
    ("call", "ara"),
    ("can", "yapabilir"),
    ("can't", "yapamam"),
    ("car", "araba"),
    ("careful", "dikkatli"),
    ("cars", "arabalar"),
    ("cash", "nakit"),
    ("catch", "yakala"),
    ("check", "kontrol et"),
    ("city", "şehir"),
    ("clean", "temiz"),
    ("close", "yakın"),
    ("club", "kulüp"),
    ("cock", "sik"),
    ("cocksucker", "yarak kafalı"),
    ("cold", "soğuk"),
    ("come", "gel"),
    ("coming", "geliyorum"),
    ("cool", "havalı"),
    ("cop", "polis"),
    ("cops", "polisler"),
    ("could", "olabilir"),
    ("couldn't", "yapamadım"),
    ("crap", "bok"),
    ("crazy", "deli"),
    ("crew", "ekip"),
    ("cunt", "amcık"),
    ("cut", "kes"),
    ("damn", "lan"),
    ("damnit", "lanet olsun"),
    ("dark", "karanlık"),
    ("day", "gün"),
    ("dead", "ölü"),
    ("deal", "anlaşma"),
    ("dick", "sik"),
    ("dickhead", "sik kafalı"),
    ("did", "yaptı"),
    ("didn't", "yapmadı"),
    ("die", "öl"),
    ("dirty", "pis"),
    ("do", "yap"),
    ("does", "yapar"),
    ("doesn't", "yapmıyor"),
    ("doing", "yapıyor"),
    ("don't", "yapma"),
    ("done", "bitti"),
    ("door", "kapı"),
    ("down", "aşağı"),
    ("dre'quan", "Dre'Quan"),
    ("drink", "içki"),
    ("drive", "sür"),
    ("driver", "şoför"),
    ("driving", "sürüyor"),
    ("drop", "bırak"),
    ("drugs", "uyuşturucu"),
    ("dude", "kanka"),
    ("dumbass", "gerizekalı"),
    ("easy", "kolay"),
    ("else", "başka"),
    ("enemy", "düşman"),
    ("enough", "yeter"),
    ("escape", "kaçış"),
    ("ever", "hiç"),
    ("everybody", "herkes"),
    ("everyone", "herkes"),
    ("everything", "her şey"),
    ("eyes", "gözler"),
    ("face", "yüz"),
    ("family", "aile"),
    ("fast", "hızlı"),
    ("feds", "federaller"),
    ("feel", "hissediyorum"),
    ("fight", "kavga"),
    ("find", "bul"),
    ("fine", "tamam"),
    ("fire", "ateş"),
    ("first", "ilk"),
    ("fix", "hallet"),
    ("follow", "takip et"),
    ("fool", "aptal"),
    ("for", "için"),
    ("forget", "unut"),
    ("free", "serbest"),
    ("freeze", "don kal"),
    ("friend", "arkadaş"),
    ("friends", "arkadaşlar"),
    ("from", "buradan"),
    ("front", "ön"),
    ("fuck", "siktir"),
    ("fucked", "sikildi"),
    ("fucker", "orospu çocuğu"),
    ("fuckers", "orospu çocukları"),
    ("fucking", "amına koduğumun"),
    ("full", "dolu"),
    ("funny", "komik"),
    ("game", "oyun"),
    ("gang", "çete"),
    ("gas", "gaz"),
    ("get", "al"),
    ("getting", "alıyor"),
    ("girl", "kız"),
    ("give", "ver"),
    ("go", "git"),
    ("god", "tanrı"),
    ("goddamn", "lanet"),
    ("going", "gidiyor"),
    ("gonna", "gidecek"),
    ("good", "iyi"),
    ("got", "aldım"),
    ("gotta", "lazım"),
    ("grab", "kap"),
    ("grassrivers", "Grassrivers"),
    ("great", "harika"),
    ("gun", "silah"),
    ("guns", "silahlar"),
    ("guy", "adam"),
    ("guys", "adamlar"),
    ("hand", "el"),
    ("hands", "eller"),
    ("happen", "olur"),
    ("happened", "oldu"),
    ("hard", "zor"),
    ("have", "var"),
    ("he", "o"),
    ("he's", "o"),
    ("head", "kafa"),
    ("hear", "duy"),
    ("heat", "baskı"),
    ("heist", "soygun"),
    ("hell", "cehennem"),
    ("hello", "selam"),
    ("help", "yardım et"),
    ("her", "onu"),
    ("here", "buraya"),
    ("here's", "işte"),
    ("hey", "hey"),
    ("hi", "selam"),
    ("hide", "saklan"),
    ("him", "onu"),
    ("his", "onun"),
    ("hit", "vur"),
    ("hold", "tut"),
    ("home", "ev"),
    ("homie", "kanka"),
    ("hot", "sıcak"),
    ("house", "ev"),
    ("how", "nasıl"),
    ("hurry", "acele et"),
    ("i", "ben"),
    ("i'll", "ben"),
    ("i'm", "ben"),
    ("i've", "ben"),
    ("idiot", "salak"),
    ("if", "eğer"),
    ("in", "içinde"),
    ("inside", "içeride"),
    ("into", "içine"),
    ("is", ""),
    ("isn't", "değil"),
    ("it", "o"),
    ("it's", "bu"),
    ("jackass", "eşek herif"),
    ("jail", "hapis"),
    ("jason", "Jason"),
    ("job", "iş"),
    ("jump", "atla"),
    ("just", "sadece"),
    ("keep", "tut"),
    ("key", "anahtar"),
    ("keys", "anahtarlar"),
    ("kid", "çocuk"),
    ("kill", "öldür"),
    ("killed", "öldürdü"),
    ("kinda", "biraz"),
    ("know", "biliyorum"),
    ("last", "son"),
    ("late", "geç"),
    ("later", "sonra"),
    ("leave", "git"),
    ("left", "sola"),
    ("leonida", "Leonida"),
    ("let", "hadi"),
    ("let's", "hadi"),
    ("life", "hayat"),
    ("like", "gibi"),
    ("listen", "dinle"),
    ("little", "küçük"),
    ("live", "yaşa"),
    ("lock", "kilitle"),
    ("long", "uzun"),
    ("look", "bak"),
    ("looking", "bakıyor"),
    ("lose", "kaybet"),
    ("lost", "kayboldu"),
    ("lot", "çok"),
    ("love", "aşk"),
    ("lucia", "Lucia"),
    ("luck", "şans"),
    ("mad", "deli"),
    ("make", "yap"),
    ("man", "adam"),
    ("maybe", "belki"),
    ("me", "beni"),
    ("mean", "demek"),
    ("meet", "buluş"),
    ("mess", "bela"),
    ("mind", "akıl"),
    ("minute", "dakika"),
    ("miss", "kaçır"),
    ("mission", "görev"),
    ("money", "para"),
    ("more", "daha"),
    ("morning", "sabah"),
    ("motel", "motel"),
    ("motherfucker", "orospu çocuğu"),
    ("motherfuckers", "orospu çocukları"),
    ("move", "hareket et"),
    ("much", "çok"),
    ("must", "lazım"),
    ("my", "benim"),
    ("name", "isim"),
    ("need", "lazım"),
    ("never", "asla"),
    ("new", "yeni"),
    ("next", "sonraki"),
    ("nice", "güzel"),
    ("night", "gece"),
    ("no", "hayır"),
    ("nobody", "kimse"),
    ("not", "değil"),
    ("nothing", "hiçbir şey"),
    ("now", "şimdi"),
    ("of", ""),
    ("off", "dışarı"),
    ("ok", "tamam"),
    ("okay", "tamam"),
    ("old", "eski"),
    ("on", "üstünde"),
    ("one", "bir"),
    ("only", "sadece"),
    ("open", "aç"),
    ("or", "veya"),
    ("our", "bizim"),
    ("out", "dışarı"),
    ("outside", "dışarıda"),
    ("outta", "dışarı"),
    ("over", "bitti"),
    ("own", "kendi"),
    ("park", "park et"),
    ("partner", "ortak"),
    ("party", "parti"),
    ("pay", "öde"),
    ("people", "insanlar"),
    ("phone", "telefon"),
    ("pick", "al"),
    ("piss", "işe"),
    ("pissed", "sinirli"),
    ("place", "yer"),
    ("plan", "plan"),
    ("play", "oyna"),
    ("please", "lütfen"),
    ("police", "polis"),
    ("prick", "göt"),
    ("prison", "hapishane"),
    ("problem", "sorun"),
    ("pull", "çek"),
    ("pussy", "amcık"),
    ("put", "koy"),
    ("quick", "çabuk"),
    ("quiet", "sessiz"),
    ("raul", "Raul"),
    ("ready", "hazır"),
    ("real", "gerçek"),
    ("really", "cidden"),
    ("relax", "rahatla"),
    ("remember", "hatırla"),
    ("ride", "sürüş"),
    ("right", "doğru"),
    ("road", "yol"),
    ("rob", "soy"),
    ("room", "oda"),
    ("roxy", "Roxy"),
    ("run", "kaç"),
    ("running", "koşuyor"),
    ("safe", "güvenli"),
    ("said", "dedi"),
    ("same", "aynı"),
    ("save", "kurtar"),
    ("say", "söyle"),
    ("score", "vurgun"),
    ("screw", "siktir et"),
    ("second", "saniye"),
    ("see", "gör"),
    ("sell", "sat"),
    ("serious", "ciddi"),
    ("she", "o"),
    ("she's", "o"),
    ("shit", "bok"),
    ("shitty", "bok gibi"),
    ("shoot", "ateş et"),
    ("shop", "dükkân"),
    ("shot", "vurdu"),
    ("should", "gerek"),
    ("shouldn't", "yapmamalı"),
    ("show", "göster"),
    ("shut", "kapat"),
    ("side", "taraf"),
    ("sit", "otur"),
    ("slow", "yavaş"),
    ("slut", "kaşar"),
    ("smart", "akıllı"),
    ("smoke", "duman"),
    ("so", "yani"),
    ("some", "biraz"),
    ("somebody", "biri"),
    ("someone", "biri"),
    ("something", "bir şey"),
    ("son", "oğul"),
    ("soon", "yakında"),
    ("sorry", "özür dilerim"),
    ("start", "başla"),
    ("stay", "kal"),
    ("steal", "çal"),
    ("still", "hâlâ"),
    ("stop", "dur"),
    ("store", "dükkân"),
    ("story", "hikâye"),
    ("street", "sokak"),
    ("streets", "sokaklar"),
    ("stuff", "mal"),
    ("stupid", "aptal"),
    ("sure", "eminim"),
    ("take", "al"),
    ("talk", "konuş"),
    ("talking", "konuşuyor"),
    ("tell", "söyle"),
    ("thanks", "sağ ol"),
    ("that", "o"),
    ("that's", "bu"),
    ("the", ""),
    ("their", "onların"),
    ("them", "onları"),
    ("then", "sonra"),
    ("there", "orada"),
    ("there's", "var"),
    ("these", "bunlar"),
    ("they", "onlar"),
    ("they'll", "onlar"),
    ("they're", "onlar"),
    ("they've", "onlar"),
    ("thing", "şey"),
    ("things", "şeyler"),
    ("think", "sanıyorum"),
    ("this", "bu"),
    ("those", "şunlar"),
    ("through", "içinden"),
    ("time", "zaman"),
    ("to", ""),
    ("today", "bugün"),
    ("together", "birlikte"),
    ("told", "söyledim"),
    ("tonight", "bu gece"),
    ("too", "da"),
    ("town", "kasaba"),
    ("track", "iz"),
    ("trouble", "bela"),
    ("truck", "kamyon"),
    ("true", "doğru"),
    ("trust", "güven"),
    ("try", "dene"),
    ("turn", "dön"),
    ("two", "iki"),
    ("under", "altında"),
    ("understand", "anlıyor musun"),
    ("up", "yukarı"),
    ("us", "bizi"),
    ("use", "kullan"),
    ("very", "çok"),
    ("wait", "bekle"),
    ("waiting", "bekliyor"),
    ("walk", "yürü"),
    ("wanna", "istiyorum"),
    ("want", "istiyorum"),
    ("was", "idi"),
    ("wasn't", "değildi"),
    ("watch", "izle"),
    ("water", "su"),
    ("way", "yol"),
    ("we", "biz"),
    ("we'll", "biz"),
    ("we're", "biz"),
    ("we've", "biz"),
    ("weapon", "silah"),
    ("well", "peki"),
    ("were", "idi"),
    ("weren't", "değildi"),
    ("what", "ne"),
    ("what's", "ne"),
    ("whatever", "her neyse"),
    ("when", "ne zaman"),
    ("where", "nerede"),
    ("where's", "nerede"),
    ("which", "hangi"),
    ("while", "iken"),
    ("who", "kim"),
    ("who's", "kim"),
    ("whole", "bütün"),
    ("whore", "orospu"),
    ("why", "neden"),
    ("will", "olacak"),
    ("win", "kazan"),
    ("with", "ile"),
    ("without", "olmadan"),
    ("woman", "kadın"),
    ("won't", "yapmayacak"),
    ("word", "söz"),
    ("work", "iş"),
    ("working", "çalışıyor"),
    ("world", "dünya"),
    ("worry", "endişelenme"),
    ("would", "olurdu"),
    ("wouldn't", "olmazdı"),
    ("wrong", "yanlış"),
    ("y'all", "hepiniz"),
    ("yeah", "evet"),
    ("yes", "evet"),
    ("yet", "henüz"),
    ("yo", "yo"),
    ("you", "sen"),
    ("you'll", "sen"),
    ("you're", "sen"),
    ("you've", "sen"),
    ("your", "senin"),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Profanity stays uncensored, full strength, GTA register.
    #[test]
    fn profanity_is_translated_not_censored() {
        let wtf = translate("What the fuck");
        assert!(wtf.to_lowercase().contains("siktir"), "got: {wtf}");
        assert!(!wtf.contains('*'));
        assert!(translate("You asshole").to_lowercase().contains("amcık"));
        assert!(translate("You son of a bitch").to_lowercase().contains("orospu"));
        assert_eq!(translate("Goddamn it"), "Lanet olsun");
    }

    /// Unknown words and GTA VI story names survive translation, punctuation attached.
    #[test]
    fn names_survive() {
        let tr = translate("Jason, Lucia is here.");
        assert!(tr.contains("Jason,"), "got: {tr}");
        assert!(tr.contains("Lucia"), "got: {tr}");
        assert!(tr.ends_with('.'), "got: {tr}");
    }

    /// Both tables are strictly alphabetical for binary search.
    #[test]
    fn dict_is_sorted() {
        for pair in DICT.windows(2) {
            assert!(pair[0].0 < pair[1].0, "DICT: {} !< {}", pair[0].0, pair[1].0);
        }
        for pair in PHRASES.windows(2) {
            assert!(pair[0].0 < pair[1].0, "PHRASES: {} !< {}", pair[0].0, pair[1].0);
        }
    }

    /// Multi-word idioms and contractions translate cleanly without double spaces.
    #[test]
    fn phrases_and_contractions_translate_cleanly() {
        let s = translate("Shut the fuck up, let's go!");
        assert!(s.to_lowercase().contains("siktir git kapa çeneni"), "got: {s}");
        assert!(s.to_lowercase().contains("hadi gidelim!"), "got: {s}");
        assert!(!s.contains("  "), "double space in: {s}");
    }

    /// The cache file path is the one the task names.
    #[test]
    fn the_cache_lives_where_the_task_says() {
        if std::env::var_os("VICTORY_SUBS_CACHE").is_none() {
            assert_eq!(cache_path(), PathBuf::from("tr_subtitles_cache.json"));
        }
    }

    /// The hand-rolled JSON reader and writer round-trip, Turkish
    /// characters and escapes included.
    #[test]
    fn cache_json_round_trip() {
        let mut map = HashMap::new();
        map.insert("what the fuck?".to_owned(), "ne oluyor lan?".to_owned());
        map.insert("quote\"and\\slash".to_owned(), "tırnak \"ve\\bölü".to_owned());
        map.insert("line\nbreak".to_owned(), "satır\nkırık ğüşıİöç".to_owned());
        let mut text = String::from("{");
        for (k, v) in &map {
            text.push_str("\n  ");
            push_json_string(&mut text, k);
            text.push_str(": ");
            push_json_string(&mut text, v);
            text.push(',');
        }
        text.push_str("\n}");
        let back = parse_json_object(&text).expect("parses");
        assert_eq!(back, map);
    }

    /// A user-edited cache file (pretty, non-ASCII Turkish, trailing comma)
    /// parses.
    #[test]
    fn a_user_edited_file_parses() {
        let text = "{\n  \"hello\": \"selam\",\n  \"money\": \"para\",\n}\n";
        let map = parse_json_object(text).expect("parses");
        assert_eq!(map.get("hello").map(String::as_str), Some("selam"));
        assert_eq!(map.get("money").map(String::as_str), Some("para"));
    }

    /// A broken cache file is an empty map, not a crash.
    #[test]
    fn a_broken_file_is_empty() {
        assert_eq!(parse_json_object("not json at all"), None);
        assert_eq!(parse_json_object(""), None);
    }
}
