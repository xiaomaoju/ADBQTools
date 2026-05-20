use std::path::PathBuf;
use std::fs;

fn platform_subdir() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        panic!("Unsupported platform")
    }
}

fn adb_binary_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "adb.exe"
    } else {
        "adb"
    }
}

pub struct EmbeddedResources {
    resource_dir: PathBuf,
    data_dir: PathBuf,
}

impl EmbeddedResources {
    pub fn new(resource_dir: PathBuf, data_dir: PathBuf) -> Self {
        Self { resource_dir, data_dir }
    }

    pub fn adb_path(&self) -> PathBuf {
        self.resource_dir.join(platform_subdir()).join(adb_binary_name())
    }

    pub fn bundletool_path(&self) -> PathBuf {
        self.resource_dir.join("shared").join("bundletool.jar")
    }

    pub fn jre_java_path(&self) -> PathBuf {
        let jre_dir = self.data_dir.join("jre");
        if cfg!(target_os = "windows") {
            jre_dir.join("bin").join("java.exe")
        } else {
            jre_dir.join("Contents").join("Home").join("bin").join("java")
        }
    }

    pub fn keytool_path(&self) -> PathBuf {
        let jre_dir = self.data_dir.join("jre");
        if cfg!(target_os = "windows") {
            jre_dir.join("bin").join("keytool.exe")
        } else {
            jre_dir.join("Contents").join("Home").join("bin").join("keytool")
        }
    }

    pub fn jre_zip_path(&self) -> PathBuf {
        self.resource_dir.join(platform_subdir()).join("jre.zip")
    }

    /// Legacy: check if JRE was bundled as a directory (dev mode)
    pub fn jre_source_dir(&self) -> PathBuf {
        self.resource_dir.join(platform_subdir()).join("jre")
    }

    pub fn jre_data_dir(&self) -> PathBuf {
        self.data_dir.join("jre")
    }

    pub fn ensure_jre_extracted(&self) -> Result<(), String> {
        let target = self.jre_data_dir();
        if target.exists() {
            return Ok(());
        }

        // Try zip first (release builds), then directory copy (dev mode)
        let zip_path = self.jre_zip_path();
        if zip_path.exists() {
            return Self::extract_jre_zip(&zip_path, &self.data_dir);
        }

        let source = self.jre_source_dir();
        if source.exists() {
            return copy_dir_recursive(&source, &target)
                .map_err(|e| format!("Failed to copy JRE: {}", e));
        }

        Err(format!("JRE not found: tried {:?} and {:?}", zip_path, source))
    }

    fn extract_jre_zip(zip_path: &PathBuf, data_dir: &PathBuf) -> Result<(), String> {
        let file = fs::File::open(zip_path)
            .map_err(|e| format!("Cannot open jre.zip: {}", e))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("Cannot read jre.zip: {}", e))?;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)
                .map_err(|e| format!("Zip entry error: {}", e))?;
            let out_path = data_dir.join(entry.mangled_name());

            if entry.is_dir() {
                fs::create_dir_all(&out_path)
                    .map_err(|e| format!("mkdir failed: {}", e))?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("mkdir failed: {}", e))?;
                }
                let mut outfile = fs::File::create(&out_path)
                    .map_err(|e| format!("Create file failed: {}", e))?;
                std::io::copy(&mut entry, &mut outfile)
                    .map_err(|e| format!("Write file failed: {}", e))?;

                // Preserve executable permissions on Unix
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Some(mode) = entry.unix_mode() {
                        fs::set_permissions(&out_path, fs::Permissions::from_mode(mode)).ok();
                    }
                }
            }
        }

        Ok(())
    }

    #[cfg(unix)]
    pub fn ensure_executable_permissions(&self) -> Result<(), String> {
        use std::os::unix::fs::PermissionsExt;
        let adb = self.adb_path();
        if adb.exists() {
            fs::set_permissions(&adb, fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("chmod adb failed: {}", e))?;
        }
        let java = self.jre_java_path();
        if java.exists() {
            fs::set_permissions(&java, fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("chmod java failed: {}", e))?;
        }
        Ok(())
    }

    #[cfg(not(unix))]
    pub fn ensure_executable_permissions(&self) -> Result<(), String> {
        Ok(())
    }
}

fn copy_dir_recursive(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_subdir() {
        let subdir = platform_subdir();
        if cfg!(target_os = "macos") {
            assert_eq!(subdir, "macos");
        } else if cfg!(target_os = "windows") {
            assert_eq!(subdir, "windows");
        }
    }

    #[test]
    fn test_adb_binary_name() {
        let name = adb_binary_name();
        if cfg!(target_os = "windows") {
            assert_eq!(name, "adb.exe");
        } else {
            assert_eq!(name, "adb");
        }
    }

    #[test]
    fn test_adb_path() {
        let res = EmbeddedResources::new(
            PathBuf::from("/app/resources"),
            PathBuf::from("/app/data"),
        );
        let adb = res.adb_path();
        if cfg!(target_os = "macos") {
            assert_eq!(adb, PathBuf::from("/app/resources/macos/adb"));
        }
    }

    #[test]
    fn test_jre_java_path() {
        let res = EmbeddedResources::new(
            PathBuf::from("/app/resources"),
            PathBuf::from("/app/data"),
        );
        let java = res.jre_java_path();
        if cfg!(target_os = "macos") {
            assert_eq!(java, PathBuf::from("/app/data/jre/Contents/Home/bin/java"));
        }
    }
}
