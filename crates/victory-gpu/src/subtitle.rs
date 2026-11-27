//! Canlı Türkçe altyazı: çeviri çekirdeği (kural + sözlük, sansürsüz GTA
//! üslubu) ve kalıcı önbellek (`tr_subtitles_cache.json`).
//!
//! Küfür ve argo **olduğu gibi** Türkçe sokak karşılığına gider; yıldızlama
//! (`***`) ve yumuşatma yok. Önbellek elle düzenlenebilir: var olan satırlar
//! bir daha çevrilmez, 0 ms gecikmeyle gösterilir.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

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

/// Whether the live subtitles are on at all (`VICTORY_SUBS`; the launcher
/// profiles set it).
pub(crate) fn asked() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| crate::switch::on("VICTORY_SUBS"))
}

/// The persistent cache: English -> Turkish. Read on first use, written by
/// the translation thread whenever a new line is translated (best effort: a
/// failed write only logs; the subtitles must never take the game down).
struct Cache {
    map: HashMap<String, String>,
}

static CACHE: std::sync::Mutex<Option<Cache>> = std::sync::Mutex::new(None);

fn cache() -> std::sync::MutexGuard<'static, Option<Cache>> {
    CACHE.lock().unwrap_or_else(|e| e.into_inner())
}

/// Look a line up in the cache (0 ms; the user's own edits included).
pub(crate) fn cached(en: &str) -> Option<String> {
    cache().as_ref()?.get(en).cloned()
}

/// Translate `en` (sansürsüz) and remember it in the cache. Called from the
/// translation thread only, never the render thread.
pub(crate) fn translate_and_remember(en: &str) -> String {
    let tr = translate(en);
    let mut guard = cache();
    let cache = guard.get_or_insert_with(|| Cache { map: load_cache_file() });
    if !cache.map.contains_key(en) {
        cache.map.insert(en.to_owned(), tr.clone());
        write_cache_file(&cache.map);
    }
    tr
}

/// `tr_subtitles_cache.json` as `HashMap<String, String>`: a flat
/// `{"english": "türkçe"}` object. Parse errors give an empty map (the game
/// runs on; the next write fixes the file).
fn load_cache_file() -> HashMap<String, String> {
    let Ok(text) = std::fs::read_to_string(cache_path()) else {
        return HashMap::new();
    };
    parse_json_object(&text).unwrap_or_default()
}

/// Writes the map back, best effort.
fn write_cache_file(map: &HashMap<String, String>) {
    let mut out = String::from("{\n");
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();
    for key in keys {
        out.push_str("  ");
        push_json_string(&mut out, key);
        out.push_str(": ");
        push_json_string(&mut out, &map[key]);
        out.push_str(",\n");
    }
    // A trailing comma on the last member is ugly but valid enough for this
    // parser; strip it for a clean file.
    if out.len() > 2 {
        let cut = out.len() - 2;
        out.truncate(cut);
        out.push('\n');
    }
    out.push('}');
    let _ = std::fs::write(cache_path(), out);
}

/// A minimal JSON string-object reader for the cache file (the repo takes
/// no JSON dependency): one flat object of string -> string. Escapes
/// `\\"`, `\\n`, `\\t` and `\\uXXXX` (Turkish characters live there).
fn parse_json_object(text: &str) -> Option<HashMap<String, String>> {
    let bytes = text.as_bytes();
    let (mut i, n) = (0, bytes.len());
    let skip_ws = |i: &mut usize| {
        while *i < n && (bytes[*i] as char).is_whitespace() {
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
                    Some(b'u') => {
                        let hex = std::str::from_utf8(bytes.get(*i + 1..*i + 5)?).ok()?;
                        let code = u32::from_str_radix(hex, 16).ok()?;
                        *i += 4;
                        // A surrogate pair: the low half follows.
                        if (0xD800..0xDC00).contains(&code)
                            && bytes.get(*i + 1) == Some(&b'\\')
                            && bytes.get(*i + 2) == Some(&b'u')
                            && let Some(low) = std::str::from_utf8(bytes.get(*i + 3..*i + 7)?).ok()
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
            byte if byte < 0x80 => out.push(byte as char),
            // UTF-8 multibyte: whole sequence for the char.
            _ => {
                let width = utf8_width(bytes[*i]);
                let slice = bytes.get(*i..*i + width)?;
                out.push_str(std::str::from_utf8(slice).ok()?);
                *i += width - 1;
            }
        }
        *i += 1;
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
            // Non-ASCII (Turkish characters) goes out raw: the file stays
            // readable for hand edits.
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Sansürsüz kural + sözlük çevirisi: GTA üslubu, sokak dili. Küfür ve argo
/// olduğu gibi kalır — yıldızlama yok. Sözlük oyun dilinin çekirdeğini
/// karşılar; `tr_subtitles_cache.json` ile kullanıcı genişletir.
pub(crate) fn translate(en: &str) -> String {
    let mut out = String::with_capacity(en.len() + 16);
    for (i, word) in en.split_whitespace().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let (head, bare, tail) = split_word(word);
        match DICT.binary_search_by(|(k, _)| (*k).cmp(bare)) {
            Ok(i) => {
                let tr = DICT[i].1;
                if tr.is_empty() {
                    // "the/a/an" gibi: hiçbir şey basma.
                } else {
                    out.push_str(head);
                    out.push_str(tr);
                    out.push_str(tail);
                }
            }
            Err(_) => out.push_str(word),
        }
    }
    out.trim().to_owned()
}

/// A word into (punctuation head, bare lowercase, punctuation tail).
fn split_word(word: &str) -> (&str, &str, &str) {
    let start = word.find(|c: char| c.is_alphanumeric()).unwrap_or(word.len());
    let end = word[start..]
        .find(|c: char| !c.is_alphanumeric())
        .map(|i| start + i)
        .unwrap_or(word.len());
    let head = &word[..start];
    let bare = &word[start..end];
    let tail = &word[end..];
    (head, bare, tail)
}

/// The dictionary: alphabetical by key (binary search). Swears at full
/// strength, GTA register — no stars, no softening.
static DICT: &[(&str, &str)] = &[
    ("a", ""),
    ("alright", "tamam"),
    ("an", ""),
    ("and", "ve"),
    ("anyway", "neyse"),
    ("are", ""),
    ("ass", "kıç"),
    ("asshole", "amcık"),
    ("bastard", "piç"),
    ("bitch", "orospu"),
    ("boss", "patron"),
    ("bro", "kardeş"),
    ("bullshit", "saçmalık"),
    ("car", "araba"),
    ("cash", "nakit"),
    ("city", "şehir"),
    ("come", "gel"),
    ("cops", "polisler"),
    ("crap", "bok"),
    ("damn", "lan"),
    ("dick", "sik"),
    ("dude", "kanka"),
    ("enemy", "düşman"),
    ("fast", "hızlı"),
    ("friend", "arkadaş"),
    ("fuck", "siktir"),
    ("fucked", "siktir ettim"),
    ("fucker", "orospu çocuğu"),
    ("fucking", "siktirgin"),
    ("get", "al"),
    ("give", "ver"),
    ("go", "git"),
    ("god", "tanrı"),
    ("goddamn", "lanet"),
    ("good", "iyi"),
    ("great", "harika"),
    ("guy", "adam"),
    ("guys", "adamlar"),
    ("gun", "silah"),
    ("hell", "cehennem"),
    ("hello", "selam"),
    ("hey", "hey"),
    ("here", "buraya"),
    ("hi", "selam"),
    ("home", "ev"),
    ("house", "ev"),
    ("how", "nasıl"),
    ("i", "ben"),
    ("is", ""),
    ("it", "o"),
    ("jason", "Jason"),
    ("job", "iş"),
    ("kill", "öldür"),
    ("know", "biliyorum"),
    ("let", "hadi"),
    ("look", "bak"),
    ("man", "adam"),
    ("me", "beni"),
    ("money", "para"),
    ("motherfucker", "annenin amına koyayım"),
    ("my", "benim"),
    ("need", "lazım"),
    ("new", "yeni"),
    ("night", "gece"),
    ("no", "hayır"),
    ("not", "değil"),
    ("now", "şimdi"),
    ("of", ""),
    ("okay", "tamam"),
    ("ok", "tamam"),
    ("old", "eski"),
    ("on", "üstünde"),
    ("police", "polis"),
    ("problem", "sorun"),
    ("run", "kaç"),
    ("shit", "bok"),
    ("shitty", "bok gibi"),
    ("slut", "kaşar"),
    ("sorry", "özür dilerim"),
    ("stop", "dur"),
    ("street", "sokak"),
    ("sure", "eminim"),
    ("take", "al"),
    ("the", ""),
    ("they", "onlar"),
    ("think", "sanıyorum"),
    ("time", "zaman"),
    ("thanks", "sağ ol"),
    ("want", "istiyorum"),
    ("we", "biz"),
    ("what", "ne"),
    ("whore", "orospu"),
    ("work", "iş"),
    ("yeah", "evet"),
    ("yes", "evet"),
    ("yo", "yo"),
    ("you", "sen"),
    ("your", "senin"),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Profanity stays uncensored, full strength, GTA register.
    #[test]
    fn profanity_is_translated_not_censored() {
        assert!(translate("What the fuck").contains("siktir"));
        assert!(translate("You asshole").contains("amcık"));
        assert!(translate("You son of a bitch").contains("orospu"));
        assert_eq!(translate("Goddamn it"), "lanet o");
    }

    /// Unknown words (names) survive translation, punctuation attached.
    #[test]
    fn names_survive() {
        let tr = translate("Jason, is here.");
        assert!(tr.contains("Jason,"));
        assert!(tr.ends_with('.'));
    }

    /// The dictionary is alphabetical (binary search).
    #[test]
    fn dict_is_sorted() {
        for pair in DICT.windows(2) {
            assert!(pair[0].0 < pair[1].0, "{} !< {}", pair[0].0, pair[1].0);
        }
    }

    /// The cache file path is the one the task names.
    #[test]
    fn the_cache_lives_where_the_task_says() {
        assert_eq!(cache_path(), PathBuf::from("tr_subtitles_cache.json"));
    }

    /// The hand-rolled JSON reader and writer round-trip, Turkish
    /// characters and escapes included.
    #[test]
    fn cache_json_round_trip() {
        let mut map = HashMap::new();
        map.insert("what the fuck?".to_owned(), "ne oluyor lan?".to_owned());
        map.insert("quote\"and\\slash".to_owned(), "tırnak \"ve\\bölü".to_owned());
        map.insert("line\nbreak".to_owned(), "satır\nkırık".to_owned());
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
        assert_eq!(parse_json_object("").len(), 0);
    }
}
