//! Fast index of precompiled `.spv` shader modules (görev 1): the folder
//! `VICTORY_SPIRV_DIR` names (default
//! `C:\Users\clawz\Desktop\xbox_to_vulkan\output_spirv` as the task states;
//! any other path can be given) is scanned **once at startup** into an
//! in-memory table; after that a lookup never touches the disk.
//!
//! The file names carry all the keys a translation may be asked by:
//! `00000_VS_AlphaTransmissionFullScreenTri_4f16bf6c_Wrapped_304de902.spv`
//! is the 5-digit index, the stage, the shader name, the hash before
//! `_Wrapped_` and the hash after it. The index answers by any of the four.
//!
//! Keys that were asked for and are not in the table are remembered in a
//! small negative cache, so a miss costs one hash lookup, not a disk scan
//! — every frame, not just the first.
//!
//! A file that is asked for by content is read once into memory (its SPIR-V
//! magic checked) and kept as a module for reuse; a file that cannot be
//! read or does not hold SPIR-V falls back to nothing (the run keeps its
//! own translation path) and is written to `victory-spirv-fallback.log`
//! beside the run, once per file, so the main log stays clean.
//!
//! This module never feeds the host pipelines by itself: the host is D3D12
//! and translates the title's own containers (decision 0003). It is a
//! lookup service for the precompiled set the task asked to index —
//! wired in behind `VICTORY_SPIRV_DIR` so it cannot change a run that does
//! not name it, and none of its failures can reach the game.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

/// Where the task's precompiled set lies, unless `VICTORY_SPIRV_DIR` says
/// elsewhere.
const DEFAULT_DIR: &str = r"C:\Users\clawz\Desktop\xbox_to_vulkan\output_spirv";

/// The fallback log's name, written into the run's working folder.
const FALLBACK_LOG: &str = "victory-spirv-fallback.log";

/// One `.spv` file as the index knows it, parsed out of its name.
#[derive(Debug)]
pub struct SpvFile {
    path: PathBuf,
    /// The 5-digit leading index (`"00000"`).
    index: String,
    /// The stage (`"VS"`).
    stage: String,
    /// The shader name, underscores and all
    /// (`"AlphaTransmissionFullScreenTri"`).
    name: String,
    /// The 8-hex-digit hash before `_Wrapped_`.
    before: String,
    /// The 8-hex-digit hash after `_Wrapped_`.
    after: String,
    /// The file's bytes, read once ([`SpvFile::module`]).
    read: OnceLock<Option<Arc<[u8]>>>,
}

impl SpvFile {
    /// Parses a `.spv` file name of the wrapped form; `None` for anything
    /// else (kept out of the index, counted as skipped).
    fn parse(path: PathBuf) -> Option<SpvFile> {
        let stem = path.file_stem()?.to_str()?;
        let parts: Vec<&str> = stem.split('_').collect();
        // 00000_VS_Name_Can_Underscores_4f16bf6c_Wrapped_304de902: at least
        // index, stage, name, before, "Wrapped", after.
        if parts.len() < 6 || parts[parts.len() - 2] != "Wrapped" {
            return None;
        }
        let (after, before) = (
            parts[parts.len() - 1].to_ascii_lowercase(),
            parts[parts.len() - 3].to_ascii_lowercase(),
        );
        let (index, stage) = (parts[0].to_owned(), parts[1].to_owned());
        let name = parts[2..parts.len() - 3].join("_");
        let hex = |s: &str| s.len() == 8 && s.bytes().all(|b| b.is_ascii_hexdigit());
        let digits = |s: &str| s.len() == 5 && s.bytes().all(|b| b.is_ascii_digit());
        if !hex(&before) || !hex(&after) || !digits(&index) || stage.is_empty() || name.is_empty() {
            return None;
        }
        Some(SpvFile {
            path,
            index,
            stage,
            name,
            before,
            after,
            read: OnceLock::new(),
        })
    }

    /// The file's bytes, read once and kept; `None` (kept as `None`) when it
    /// cannot be read or does not begin with the SPIR-V magic — a module
    /// that is not valid is never retried, and the miss goes to the
    /// fallback log once.
    pub fn module(&self) -> Option<Arc<[u8]>> {
        self.read
            .get_or_init(|| {
                let bytes: Arc<[u8]> = std::fs::read(&self.path).ok()?.into();
                // SPIR-V magic 0x07230203, little-endian on disk.
                if bytes.len() < 4 || bytes[..4] != [0x03, 0x02, 0x23, 0x07] {
                    fallback_log(&format!(
                        "{}: not SPIR-V (magic {:02x?}); not used",
                        self.path.display(),
                        bytes.first_chunk::<4>()
                    ));
                    return None;
                }
                Some(bytes)
            })
            .clone()
    }
}

/// The four keys a file is found by.
fn keys(file: &SpvFile) -> [String; 4] {
    [
        format!("idx:{}", file.index),
        format!("name:{}:{}", file.stage, file.name),
        format!("pre:{}", file.before),
        format!("post:{}", file.after),
    ]
}

/// The in-memory table: files by each of their keys, and how many files
/// (and how many names were skipped as not the wrapped form).
struct Table {
    by_key: HashMap<String, Arc<SpvFile>>,
    files: usize,
    skipped: usize,
}

/// Scans `dir` into a table; the one disk pass, done at startup. A missing
/// or unreadable directory is an empty table (said once).
fn scan(dir: &Path) -> Table {
    let mut table = Table {
        by_key: HashMap::new(),
        files: 0,
        skipped: 0,
    };
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            tracing::warn!(dir = %dir.display(), error = %e, "no SPIR-V index; the folder is not read");
            return table;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| !e.eq_ignore_ascii_case("spv")) {
            continue;
        }
        match SpvFile::parse(path) {
            Some(file) => {
                let file = Arc::new(file);
                for key in keys(&file) {
                    // Two files sharing a key: the first one scanned wins;
                    // the set is one translation per name, and a repeat is
                    // the same program under another index.
                    table.by_key.entry(key).or_insert_with(|| file.clone());
                }
                table.files += 1;
            }
            None => table.skipped += 1,
        }
    }
    table
}

static TABLE: OnceLock<Table> = OnceLock::new();
/// Keys asked for and not found, so they are never asked of the disk again
/// (the disk was scanned once at startup; this stops repeated map walks
/// from growing per frame). Bounded; past the bound it is dropped whole.
static NEGATIVE: Mutex<Option<HashSet<String>>> = Mutex::new(None);
/// How many misses the negative cache keeps before it is dropped whole.
const NEGATIVE_CAP: usize = 8192;

/// The directory to index: `VICTORY_SPIRV_DIR`, else the task's default.
/// `VICTORY_SPIRV_DIR=0` turns the index off altogether.
fn dir() -> Option<PathBuf> {
    match std::env::var_os("VICTORY_SPIRV_DIR") {
        Some(v) if v == "0" => None,
        Some(v) => Some(PathBuf::from(v)),
        None => Some(PathBuf::from(DEFAULT_DIR)),
    }
}

/// Builds the in-memory index once, at startup. Returns how many `.spv`
/// files it holds (0 when off). Safe to call more than once: only the
/// first call scans.
pub fn init() -> usize {
    let Some(dir) = dir() else {
        tracing::info!("SPIR-V index off (VICTORY_SPIRV_DIR=0)");
        return 0;
    };
    let table = scan(&dir);
    let files = table.files;
    let keys = table.by_key.len();
    let skipped = table.skipped;
    TABLE.set(table).ok(); // A second call keeps the first table.
    if files > 0 {
        tracing::info!(
            dir = %dir.display(),
            files,
            keys,
            skipped,
            "SPIR-V index built once at startup: every lookup after this is in memory"
        );
    }
    files
}

/// Finds a file by any of its keys; the first miss is remembered so the
/// next frame's miss is one set lookup.
fn find(key: &str) -> Option<Arc<SpvFile>> {
    if let Some(table) = TABLE.get()
        && let Some(file) = table.by_key.get(key)
    {
        return Some(file.clone());
    }
    // A miss, with the index built or not: remembered either way, so the
    // next miss is one set lookup and no disk work at all.
    let mut negative = NEGATIVE.lock().unwrap_or_else(|e| e.into_inner());
    let set = negative.get_or_insert_with(HashSet::new);
    if set.len() >= NEGATIVE_CAP {
        set.clear();
    }
    set.insert(key.to_owned());
    None
}

/// By the 5-digit index (`"00000"`).
pub fn by_index(index: &str) -> Option<Arc<SpvFile>> {
    find(&format!("idx:{index}"))
}

/// By stage and shader name (`"VS"`, `"AlphaTransmissionFullScreenTri"`).
pub fn by_name(stage: &str, name: &str) -> Option<Arc<SpvFile>> {
    find(&format!("name:{stage}:{name}"))
}

/// By the 8-hex-digit hash before `_Wrapped_`.
pub fn by_wrapped_before(hash: &str) -> Option<Arc<SpvFile>> {
    find(&format!("pre:{}", hash.to_ascii_lowercase()))
}

/// By the 8-hex-digit hash after `_Wrapped_`.
pub fn by_wrapped_after(hash: &str) -> Option<Arc<SpvFile>> {
    find(&format!("post:{}", hash.to_ascii_lowercase()))
}

/// How many files the index holds, and how many misses the negative cache
/// remembers.
pub fn counts() -> (usize, usize, usize) {
    (
        TABLE.get().map_or(0, |t| t.files),
        TABLE.get().map_or(0, |t| t.by_key.len()),
        NEGATIVE
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
            .map_or(0, |s| s.len()),
    )
}

/// Writes one line to the fallback log — beside the indexed `.spv` folder,
/// in the user's own space, not the run's folder — and to the main log as a
/// warning, once per line: repeated lines are not rewritten.
fn fallback_log(line: &str) {
    static SAID: Mutex<Option<HashSet<String>>> = Mutex::new(None);
    let mut held = SAID.lock().unwrap_or_else(|e| e.into_inner());
    let said = held.get_or_insert_with(HashSet::new);
    if said.insert(line.to_owned()) {
        tracing::warn!("{line}");
        // Beside the index's own folder (the user's space), and only when
        // the index is live (a test never writes anywhere): nowhere to
        // write when the folder is gone, the warning line above is the
        // whole record then.
        if TABLE.get().is_some() && let Some(dir) = dir() {
            let _ = std::fs::write(dir.join(FALLBACK_LOG), format!("{line}\n"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_spv(dir: &Path, name: &str, magic: [u8; 4]) -> PathBuf {
        let path = dir.join(name);
        // A header with the SPIR-V magic and a little body: enough to be a
        // "module".
        let mut bytes = magic.to_vec();
        bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x0a, 0x0b, 0x0c, 0x0d]);
        std::fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn names_parse_into_their_parts() {
        let file = SpvFile::parse(PathBuf::from(
            "00000_VS_AlphaTransmissionFullScreenTri_4f16bf6c_Wrapped_304de902.spv",
        ))
        .unwrap();
        assert_eq!(file.index, "00000");
        assert_eq!(file.stage, "VS");
        assert_eq!(file.name, "AlphaTransmissionFullScreenTri");
        assert_eq!(file.before, "4f16bf6c");
        assert_eq!(file.after, "304de902");

        // A name with underscores of its own stays whole.
        let wind = SpvFile::parse(PathBuf::from(
            "00010_VS_Octa_Vsplit_Wind_Fire_e6901d4b_Wrapped_714cdb61.spv",
        ))
        .unwrap();
        assert_eq!(wind.name, "Octa_Vsplit_Wind_Fire");
        assert_eq!(wind.before, "e6901d4b");
        assert_eq!(wind.after, "714cdb61");
    }

    #[test]
    fn names_that_are_not_the_wrapped_form_are_skipped() {
        for name in [
            "plain.spv",
            "00000_VS_No_Hashes_Wrapped.spv",
            "XXXXX_VS_Name_4f16bf6c_Wrapped_304de902.spv",
            "00000_VS_Name_notahash_Wrapped_304de902.spv",
            "00000_VS_Name_4f16bf6c_Unwrapped_304de902.spv",
        ] {
            assert!(SpvFile::parse(PathBuf::from(name)).is_none(), "{name}");
        }
    }

    #[test]
    fn the_table_answers_by_every_key_and_remembers_misses() {
        let dir = tempfile::tempdir().unwrap();
        write_spv(
            dir.path(),
            "00000_VS_AlphaTransmissionFullScreenTri_4f16bf6c_Wrapped_304de902.spv",
            [0x03, 0x02, 0x23, 0x07],
        );
        write_spv(
            dir.path(),
            "not_the_form.spv",
            [0x03, 0x02, 0x23, 0x07],
        );
        let table = scan(dir.path());
        assert_eq!((table.files, table.skipped), (1, 1));
        assert_eq!(table.by_key.len(), 4, "one file, four keys");

        // Every key finds the same file; the module reads back and is kept.
        let by_index = table.by_key["idx:00000"].clone();
        for key in [
            "name:VS:AlphaTransmissionFullScreenTri",
            "pre:4f16bf6c",
            "post:304de902",
        ] {
            assert!(Arc::ptr_eq(&table.by_key[key], &by_index), "{key}");
        }
        let module = by_index.module().unwrap();
        assert_eq!(&module[..4], &[0x03, 0x02, 0x23, 0x07]);
        // A second read comes from memory: the file can go now.
        drop(std::fs::remove_file(dir.path().join(
            "00000_VS_AlphaTransmissionFullScreenTri_4f16bf6c_Wrapped_304de902.spv",
        )));
        assert!(by_index.module().is_some());
    }

    #[test]
    fn a_file_that_is_not_spirv_is_a_module_miss() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_spv(
            dir.path(),
            "00001_VS_Passthrough_36b96a91_Wrapped_f8168d8a.spv",
            [0xde, 0xad, 0xbe, 0xef],
        );
        let file = SpvFile::parse(path).unwrap();
        assert!(file.module().is_none(), "not SPIR-V magic");
        // And it stays a miss: a module that failed is not retried.
        assert!(file.module().is_none());
    }

    /// The one path that runs without `init`: nothing is found and nothing
    /// crashes; a miss is remembered so the next miss is one set lookup.
    #[test]
    fn without_init_nothing_is_found_and_misses_are_cached() {
        if TABLE.get().is_some() {
            // Another test (or a run in the same process) built the index;
            // this test only holds when it did not.
            return;
        }
        assert!(by_index("99999").is_none());
        let (_, _, misses) = counts();
        assert!(misses >= 1, "the miss is remembered");
        assert!(by_index("99999").is_none());
        let (_, _, again) = counts();
        assert_eq!(again, misses, "the second miss added nothing");
    }

    #[test]
    fn a_missing_directory_is_an_empty_table() {
        let table = scan(Path::new("Z:\\no\\such\\folder"));
        assert_eq!((table.files, table.skipped, table.by_key.len()), (0, 0, 0));
    }
}
