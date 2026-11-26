//! The D3D12 Agility SDK exports (`src/agility.rs`, for Windows 10): D3D12
//! finds them on the executable and then loads the runtime from `D3D12\`
//! beside it. Exported only when `D3D12\D3D12Core.dll` is beside the
//! executable at build time: with the exports but no DLL, device creation
//! fails with `D3D12_ERROR_INVALID_REDIST`. Without them the in-box runtime
//! is used (Windows 11), as before. Only the MSVC linker takes /EXPORT on a
//! binary. After putting the DLL there (or taking it away), build again.

use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo::rustc-check-cfg=cfg(agility)");
    // OUT_DIR is <target>/<profile>/build/<package>-<hash>/out; the exe and
    // its D3D12\ folder are in <target>/<profile>/.
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
    let Some(profile) = out.ancestors().nth(3) else {
        return;
    };
    let dir = profile.join("D3D12");
    let core = dir.join("D3D12Core.dll");
    // Seen again when the folder or the DLL comes or goes.
    println!("cargo:rerun-if-changed={}", dir.display());
    let msvc = std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");
    if msvc && core.exists() {
        println!("cargo:rerun-if-changed={}", core.display());
        println!("cargo:rustc-link-arg-bins=/EXPORT:D3D12SDKVersion,DATA");
        println!("cargo:rustc-link-arg-bins=/EXPORT:D3D12SDKPath,DATA");
        println!("cargo:rustc-cfg=agility");
        println!("cargo:warning=D3D12\\D3D12Core.dll found: Agility SDK exports on (Windows 10)");
    }
}
