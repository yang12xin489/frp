use flate2::read::GzDecoder;
use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
};
use tar::Archive as TarArchive;
use zip::ZipArchive;

pub fn is_zip(p: &Path) -> bool {
    p.extension()
        .map(|e| e.eq_ignore_ascii_case("zip"))
        .unwrap_or(false)
}

pub fn is_tgz(p: &Path) -> bool {
    let s = p
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    s.ends_with(".tar.gz") || s.ends_with(".tgz")
}

pub fn unzip_to(archive: &Path, out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let f = File::open(archive)?;
    let mut zip = ZipArchive::new(f)?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let rel = entry
            .enclosed_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "bad zip entry path"))?
            .to_owned();
        let dest = out_dir.join(rel);
        if entry.is_dir() {
            std::fs::create_dir_all(&dest)?;
        } else {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut out = File::create(&dest)?;
            io::copy(&mut entry, &mut out)?;
        }
    }
    Ok(())
}

pub fn un_tgz_to(archive: &Path, out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let f = File::open(archive)?;
    let gz = GzDecoder::new(f);
    let mut tar = TarArchive::new(gz);
    tar.unpack(out_dir)?;
    Ok(())
}

#[inline]
pub fn frpc_name() -> &'static str {
    if cfg!(windows) {
        "frpc.exe"
    } else {
        "frpc"
    }
}

pub fn find_executable_recursively(root: &Path, target_name: &str) -> Option<PathBuf> {
    let direct = root.join(target_name);
    if direct.exists() && direct.is_file() {
        return Some(direct);
    }
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.file_name().map(|n| n == target_name).unwrap_or(false) {
                return Some(path);
            }
        }
    }
    None
}

pub fn extract_archive_to(
    archive: &Path,
    out_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if is_zip(archive) {
        unzip_to(archive, out_dir)?;
    } else if is_tgz(archive) {
        un_tgz_to(archive, out_dir)?;
    } else {
        return Err("unsupported archive (only .zip / .tar.gz / .tgz)".into());
    }
    Ok(())
}
