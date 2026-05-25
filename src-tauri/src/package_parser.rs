use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use crate::embedded::EmbeddedResources;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub file_name: String,
    pub file_size: u64,
    pub file_type: String,
    pub package_name: String,
    pub version_name: String,
    pub version_code: String,
    pub min_sdk: String,
    pub target_sdk: String,
    pub compile_sdk: String,
    pub debuggable: bool,
    pub allow_backup: bool,
    pub permissions: Vec<String>,
    pub architectures: Vec<String>,
    pub native_libs: HashMap<String, Vec<String>>,
    pub dex_count: u32,
    pub has_assets: bool,
    pub has_resources: bool,
    pub total_uncompressed_size: u64,
    pub signing_info: Option<SigningInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningInfo {
    pub owner: String,
    pub issuer: String,
    pub serial_number: String,
    pub valid_from: String,
    pub valid_to: String,
    pub sha256: String,
    pub sha1: String,
    pub md5: String,
    pub algorithm: String,
}

pub async fn parse_package(
    resources: &EmbeddedResources,
    file_path: &str,
) -> Result<PackageInfo, String> {
    let path = Path::new(file_path);
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    let metadata = std::fs::metadata(path).map_err(|e| format!("Cannot read file: {}", e))?;
    let file_size = metadata.len();

    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let file_type = match ext.as_str() {
        "apk" => "APK",
        "aab" => "AAB",
        _ => return Err("Unsupported file type".to_string()),
    }
    .to_string();

    let zip_info = parse_zip_structure(path)?;
    let signing_info = parse_signing_info(resources, file_path).await.ok();

    let manifest_info = if ext == "apk" {
        parse_apk_manifest(path).unwrap_or_default()
    } else {
        parse_aab_manifest(resources, file_path)
            .await
            .unwrap_or_default()
    };

    Ok(PackageInfo {
        file_name,
        file_size,
        file_type,
        package_name: manifest_info.package_name,
        version_name: manifest_info.version_name,
        version_code: manifest_info.version_code,
        min_sdk: manifest_info.min_sdk,
        target_sdk: manifest_info.target_sdk,
        compile_sdk: manifest_info.compile_sdk,
        debuggable: manifest_info.debuggable,
        allow_backup: manifest_info.allow_backup,
        permissions: manifest_info.permissions,
        architectures: zip_info.architectures,
        native_libs: zip_info.native_libs,
        dex_count: zip_info.dex_count,
        has_assets: zip_info.has_assets,
        has_resources: zip_info.has_resources,
        total_uncompressed_size: zip_info.total_uncompressed_size,
        signing_info,
    })
}

#[derive(Default)]
struct ManifestInfo {
    package_name: String,
    version_name: String,
    version_code: String,
    min_sdk: String,
    target_sdk: String,
    compile_sdk: String,
    debuggable: bool,
    allow_backup: bool,
    permissions: Vec<String>,
}

struct ZipInfo {
    architectures: Vec<String>,
    native_libs: HashMap<String, Vec<String>>,
    dex_count: u32,
    has_assets: bool,
    has_resources: bool,
    total_uncompressed_size: u64,
}

fn parse_zip_structure(path: &Path) -> Result<ZipInfo, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Cannot open file: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Cannot read ZIP: {}", e))?;

    let mut dex_count: u32 = 0;
    let mut has_assets = false;
    let mut has_resources = false;
    let mut total_uncompressed_size: u64 = 0;
    let mut native_libs: HashMap<String, Vec<String>> = HashMap::new();

    let is_aab = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase() == "aab")
        .unwrap_or(false);

    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| format!("ZIP entry error: {}", e))?;
        let name = entry.name().to_string();
        total_uncompressed_size += entry.size();

        if is_aab {
            if name.ends_with(".dex") && name.starts_with("base/dex/") {
                dex_count += 1;
            }
            if name.starts_with("base/assets/") {
                has_assets = true;
            }
            if name.starts_with("base/res/") {
                has_resources = true;
            }
            if name.starts_with("base/lib/") && name.ends_with(".so") {
                let parts: Vec<&str> = name.split('/').collect();
                if parts.len() >= 4 {
                    let arch = parts[2].to_string();
                    let lib_name = parts[3].to_string();
                    native_libs.entry(arch).or_default().push(lib_name);
                }
            }
        } else {
            if name.starts_with("classes") && name.ends_with(".dex") {
                dex_count += 1;
            }
            if name.starts_with("assets/") {
                has_assets = true;
            }
            if name.starts_with("res/") {
                has_resources = true;
            }
            if name.starts_with("lib/") && name.ends_with(".so") {
                let parts: Vec<&str> = name.split('/').collect();
                if parts.len() >= 3 {
                    let arch = parts[1].to_string();
                    let lib_name = parts[2].to_string();
                    native_libs.entry(arch).or_default().push(lib_name);
                }
            }
        }
    }

    let mut architectures: Vec<String> = native_libs.keys().cloned().collect();
    architectures.sort();

    Ok(ZipInfo {
        architectures,
        native_libs,
        dex_count,
        has_assets,
        has_resources,
        total_uncompressed_size,
    })
}

fn parse_apk_manifest(path: &Path) -> Result<ManifestInfo, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Cannot open file: {}", e))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Cannot read ZIP: {}", e))?;

    let mut manifest_entry = archive
        .by_name("AndroidManifest.xml")
        .map_err(|_| "AndroidManifest.xml not found in APK".to_string())?;

    let mut buf = Vec::new();
    manifest_entry
        .read_to_end(&mut buf)
        .map_err(|e| format!("Cannot read manifest: {}", e))?;

    let mut cursor = std::io::Cursor::new(buf);
    let doc =
        axmldecoder::parse(&mut cursor).map_err(|e| format!("AXML decode error: {:?}", e))?;

    parse_axml_document(&doc)
}

fn parse_axml_document(doc: &axmldecoder::XmlDocument) -> Result<ManifestInfo, String> {
    let mut info = ManifestInfo {
        package_name: String::new(),
        version_name: String::new(),
        version_code: String::new(),
        min_sdk: String::new(),
        target_sdk: String::new(),
        compile_sdk: String::new(),
        debuggable: false,
        allow_backup: false,
        permissions: Vec::new(),
    };

    if let Some(root) = doc.get_root() {
        visit_axml_node(root, &mut info);
    }

    info.permissions.sort();
    info.permissions.dedup();
    Ok(info)
}

fn axml_attr<'a>(attrs: &'a std::collections::HashMap<String, String>, name: &str) -> Option<&'a String> {
    attrs.get(name)
        .or_else(|| attrs.get(&format!("android:{}", name)))
}

fn visit_axml_node(node: &axmldecoder::Node, info: &mut ManifestInfo) {
    match node {
        axmldecoder::Node::Element(el) => {
            let tag = el.get_tag();
            let attrs = el.get_attributes();

            match tag {
                "manifest" => {
                    if let Some(v) = axml_attr(attrs, "package") {
                        info.package_name = v.clone();
                    }
                    if let Some(v) = axml_attr(attrs, "versionCode") {
                        info.version_code = v.clone();
                    }
                    if let Some(v) = axml_attr(attrs, "versionName") {
                        info.version_name = v.clone();
                    }
                    if let Some(v) = axml_attr(attrs, "compileSdkVersion") {
                        info.compile_sdk = v.clone();
                    }
                }
                "uses-sdk" => {
                    if let Some(v) = axml_attr(attrs, "minSdkVersion") {
                        info.min_sdk = v.clone();
                    }
                    if let Some(v) = axml_attr(attrs, "targetSdkVersion") {
                        info.target_sdk = v.clone();
                    }
                    if info.compile_sdk.is_empty() {
                        if let Some(v) = axml_attr(attrs, "compileSdkVersion") {
                            info.compile_sdk = v.clone();
                        }
                    }
                }
                "uses-permission" => {
                    if let Some(v) = axml_attr(attrs, "name") {
                        if !v.is_empty() {
                            info.permissions.push(v.clone());
                        }
                    }
                }
                "application" => {
                    if let Some(v) = axml_attr(attrs, "debuggable") {
                        info.debuggable =
                            v == "true" || v == "-1" || v == "0xffffffff" || v == "4294967295";
                    }
                    if let Some(v) = axml_attr(attrs, "allowBackup") {
                        info.allow_backup =
                            v == "true" || v == "-1" || v == "0xffffffff" || v == "4294967295";
                    }
                }
                _ => {}
            }

            for child in el.get_children() {
                visit_axml_node(child, info);
            }
        }
        axmldecoder::Node::Cdata(_) => {}
    }
}

async fn parse_aab_manifest(
    resources: &EmbeddedResources,
    aab_path: &str,
) -> Result<ManifestInfo, String> {
    let java_path = resources.jre_java_path();
    let bundletool_path = resources.bundletool_path();

    // On Windows, Java may fail with non-ASCII paths — use safe copy
    let effective_jar = crate::installer::safe_jar_path(&bundletool_path)?;

    let output = crate::util::create_command(&java_path)
        .args([
            "-cp",
            &effective_jar.to_string_lossy(),
            crate::installer::BUNDLETOOL_MAIN,
            "dump",
            "manifest",
            &format!("--bundle={}", aab_path),
        ])
        .output()
        .await
        .map_err(|e| format!("bundletool dump manifest failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("bundletool error: {}", stderr));
    }

    let xml_string = String::from_utf8_lossy(&output.stdout).to_string();
    parse_xml_manifest(&xml_string)
}

fn get_attr_name(attr: &quick_xml::events::attributes::Attribute) -> String {
    let local = attr.key.local_name();
    std::str::from_utf8(local.as_ref())
        .unwrap_or("")
        .to_string()
}

fn get_attr_value(attr: &quick_xml::events::attributes::Attribute) -> String {
    attr.unescape_value().unwrap_or_default().to_string()
}

fn parse_xml_manifest(xml: &str) -> Result<ManifestInfo, String> {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut reader = Reader::from_str(xml);

    let mut package_name = String::new();
    let mut version_name = String::new();
    let mut version_code = String::new();
    let mut min_sdk = String::new();
    let mut target_sdk = String::new();
    let mut compile_sdk = String::new();
    let mut debuggable = false;
    let mut allow_backup = false;
    let mut permissions: Vec<String> = Vec::new();

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let local_name = e.local_name();
                let tag = std::str::from_utf8(local_name.as_ref()).unwrap_or("");

                match tag {
                    "manifest" => {
                        for attr in e.attributes().flatten() {
                            let key = get_attr_name(&attr);
                            let val = get_attr_value(&attr);
                            match key.as_str() {
                                "package" => package_name = val,
                                "versionCode" => version_code = val,
                                "versionName" => version_name = val,
                                "compileSdkVersion" => compile_sdk = val,
                                _ => {}
                            }
                        }
                    }
                    "uses-sdk" => {
                        for attr in e.attributes().flatten() {
                            let key = get_attr_name(&attr);
                            let val = get_attr_value(&attr);
                            match key.as_str() {
                                "minSdkVersion" => min_sdk = val,
                                "targetSdkVersion" => target_sdk = val,
                                "compileSdkVersion" if compile_sdk.is_empty() => {
                                    compile_sdk = val
                                }
                                _ => {}
                            }
                        }
                    }
                    "uses-permission" => {
                        for attr in e.attributes().flatten() {
                            let key = get_attr_name(&attr);
                            if key == "name" {
                                let val = get_attr_value(&attr);
                                if !val.is_empty() {
                                    permissions.push(val);
                                }
                            }
                        }
                    }
                    "application" => {
                        for attr in e.attributes().flatten() {
                            let key = get_attr_name(&attr);
                            let val = get_attr_value(&attr);
                            match key.as_str() {
                                "debuggable" => {
                                    debuggable =
                                        val == "true" || val == "-1" || val == "0xffffffff"
                                }
                                "allowBackup" => {
                                    allow_backup =
                                        val == "true" || val == "-1" || val == "0xffffffff"
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    permissions.sort();
    permissions.dedup();

    Ok(ManifestInfo {
        package_name,
        version_name,
        version_code,
        min_sdk,
        target_sdk,
        compile_sdk,
        debuggable,
        allow_backup,
        permissions,
    })
}

async fn parse_signing_info(
    resources: &EmbeddedResources,
    file_path: &str,
) -> Result<SigningInfo, String> {
    let keytool = resources.keytool_path();
    if !keytool.exists() {
        return Err("keytool not found".to_string());
    }

    // Try 1: keytool -printcert -jarfile (works for v1/JAR signing)
    let output = crate::util::create_command(&keytool)
        .args(["-printcert", "-jarfile", file_path])
        .output()
        .await
        .map_err(|e| format!("keytool failed: {}", e))?;

    let mut stdout = String::from_utf8_lossy(&output.stdout).to_string();

    let has_cert_info = |s: &str| s.contains("Owner:") || s.contains("SHA256:");

    // Try 2: extract cert from META-INF/ and use -file
    if !has_cert_info(&stdout) {
        stdout = try_extract_cert_and_read(&keytool, file_path)
            .await
            .unwrap_or_default();
    }

    // Try 3: parse APK Signing Block v2/v3 to extract certificate
    if !has_cert_info(&stdout) {
        stdout = try_extract_v2_cert_and_read(&keytool, file_path)
            .await
            .unwrap_or_default();
    }

    if !has_cert_info(&stdout) {
        return Err("No signing info found".to_string());
    }

    let mut owner = String::new();
    let mut issuer = String::new();
    let mut serial_number = String::new();
    let mut valid_from = String::new();
    let mut valid_to = String::new();
    let mut sha256 = String::new();
    let mut sha1 = String::new();
    let mut md5 = String::new();
    let mut algorithm = String::new();

    let mut current_section = "";

    for line in stdout.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("Owner:") {
            owner = trimmed.trim_start_matches("Owner:").trim().to_string();
        } else if trimmed.starts_with("Issuer:") {
            issuer = trimmed.trim_start_matches("Issuer:").trim().to_string();
        } else if trimmed.starts_with("Serial number:") {
            serial_number = trimmed
                .trim_start_matches("Serial number:")
                .trim()
                .to_string();
        } else if trimmed.starts_with("Valid from:") {
            let parts = trimmed.trim_start_matches("Valid from:").trim();
            if let Some((from, to)) = parts.split_once("until:") {
                valid_from = from.trim().to_string();
                valid_to = to.trim().to_string();
            }
        } else if trimmed.starts_with("Signature algorithm name:") {
            algorithm = trimmed
                .trim_start_matches("Signature algorithm name:")
                .trim()
                .to_string();
        } else if trimmed.starts_with("SHA256:") {
            current_section = "sha256";
            sha256 = trimmed.trim_start_matches("SHA256:").trim().to_string();
        } else if trimmed.starts_with("SHA1:") {
            current_section = "sha1";
            sha1 = trimmed.trim_start_matches("SHA1:").trim().to_string();
        } else if trimmed.starts_with("MD5:") {
            current_section = "md5";
            md5 = trimmed.trim_start_matches("MD5:").trim().to_string();
        } else if !trimmed.is_empty()
            && trimmed.contains(':')
            && trimmed.chars().all(|c| c.is_ascii_hexdigit() || c == ':' || c == ' ')
        {
            match current_section {
                "sha256" => {
                    sha256.push_str(trimmed);
                }
                "sha1" => {
                    sha1.push_str(trimmed);
                }
                "md5" => {
                    md5.push_str(trimmed);
                }
                _ => {}
            }
        } else {
            current_section = "";
        }
    }

    Ok(SigningInfo {
        owner,
        issuer,
        serial_number,
        valid_from,
        valid_to,
        sha256,
        sha1,
        md5,
        algorithm,
    })
}

async fn try_extract_cert_and_read(
    keytool: &std::path::PathBuf,
    file_path: &str,
) -> Result<String, String> {
    let tmp_cert = std::env::temp_dir().join("adbqtools_cert.tmp");

    // Extract cert from ZIP synchronously to avoid Send issues
    {
        let file =
            std::fs::File::open(file_path).map_err(|e| format!("Cannot open file: {}", e))?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| format!("Cannot read ZIP: {}", e))?;

        let cert_name = (0..archive.len())
            .filter_map(|i| {
                let entry = archive.by_index(i).ok()?;
                let name = entry.name().to_string();
                if name.starts_with("META-INF/")
                    && (name.ends_with(".RSA")
                        || name.ends_with(".DSA")
                        || name.ends_with(".EC"))
                {
                    Some(name)
                } else {
                    None
                }
            })
            .next()
            .ok_or_else(|| "No certificate file found in META-INF/".to_string())?;

        let mut cert_entry = archive
            .by_name(&cert_name)
            .map_err(|e| format!("Cannot read cert: {}", e))?;

        let mut cert_bytes = Vec::new();
        cert_entry
            .read_to_end(&mut cert_bytes)
            .map_err(|e| format!("Cannot read cert data: {}", e))?;

        std::fs::write(&tmp_cert, &cert_bytes)
            .map_err(|e| format!("Cannot write temp cert: {}", e))?;
    }

    let output = crate::util::create_command(keytool)
        .args(["-printcert", "-file", &tmp_cert.to_string_lossy()])
        .output()
        .await
        .map_err(|e| format!("keytool failed: {}", e))?;

    std::fs::remove_file(&tmp_cert).ok();

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err("keytool -printcert -file failed".to_string())
    }
}

async fn try_extract_v2_cert_and_read(
    keytool: &std::path::PathBuf,
    file_path: &str,
) -> Result<String, String> {
    let tmp_cert = std::env::temp_dir().join("adbqtools_v2cert.der");

    // Extract certificate from APK Signing Block v2/v3
    // Only read the tail of the file (last 128KB is enough for EOCD + signing block)
    {
        use std::io::{Seek, SeekFrom};

        let mut file = std::fs::File::open(file_path)
            .map_err(|e| format!("Cannot open file: {}", e))?;
        let file_len = file
            .metadata()
            .map_err(|e| format!("Cannot read metadata: {}", e))?
            .len() as usize;

        // Read only the tail — signing block + central directory + EOCD
        // 128KB is more than enough for the signing block
        let tail_size = std::cmp::min(file_len, 128 * 1024);
        let tail_offset = file_len - tail_size;
        file.seek(SeekFrom::Start(tail_offset as u64))
            .map_err(|e| format!("Seek failed: {}", e))?;

        let mut tail = vec![0u8; tail_size];
        file.read_exact(&mut tail)
            .map_err(|e| format!("Read failed: {}", e))?;

        let cert_der = extract_v2_certificate(&tail, tail_offset)?;
        std::fs::write(&tmp_cert, &cert_der)
            .map_err(|e| format!("Cannot write temp cert: {}", e))?;
    }

    let output = crate::util::create_command(keytool)
        .args(["-printcert", "-file", &tmp_cert.to_string_lossy()])
        .output()
        .await
        .map_err(|e| format!("keytool failed: {}", e))?;

    std::fs::remove_file(&tmp_cert).ok();

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err("keytool -printcert -file failed for v2 cert".to_string())
    }
}

/// Parse APK Signing Block to extract the first X.509 certificate.
///
/// APK structure:  [Contents] [APK Signing Block] [Central Directory] [EOCD]
///
/// EOCD (End of Central Directory) is at the end of the file.
/// It contains the offset to the Central Directory.
/// The APK Signing Block is right before the Central Directory.
///
/// APK Signing Block format:
///   u64  block_size (without this field)
///   repeated {
///     u64  pair_size
///     u32  pair_id
///     bytes pair_data
///   }
///   u64  block_size (same value)
///   16 bytes magic "APK Sig Block 42"
///
/// For v2 (id=0x7109871a) and v3 (id=0xf05368c0), pair_data contains:
///   repeated signers {
///     u32 signer_size
///     signed_data {
///       u32 signed_data_size
///       digests { u32 len, ... }
///       certificates {
///         u32 total_len
///         repeated {
///           u32 cert_len
///           bytes cert_der  <-- this is the X.509 DER we want
///         }
///       }
///     }
///     ...
///   }
/// `data` is the tail of the APK file, starting at `base_offset` in the original file.
/// All absolute offsets (like cd_offset) are converted to local offsets by subtracting base_offset.
fn extract_v2_certificate(data: &[u8], base_offset: usize) -> Result<Vec<u8>, String> {
    let len = data.len();
    if len < 22 {
        return Err("File too small".to_string());
    }

    // Find EOCD (scan from end, EOCD signature = 0x06054b50)
    let eocd_pos = find_eocd(data).ok_or("EOCD not found")?;

    // Central Directory offset is at EOCD+16 (4 bytes LE) — this is an absolute file offset
    let cd_offset_abs = read_u32_le(data, eocd_pos + 16) as usize;
    let cd_offset = cd_offset_abs
        .checked_sub(base_offset)
        .ok_or("CD offset before tail buffer")?;
    if cd_offset > len {
        return Err("Invalid CD offset".to_string());
    }

    // APK Signing Block magic is 16 bytes before CD: "APK Sig Block 42"
    let magic_pos = cd_offset.checked_sub(16).ok_or("No room for magic")?;
    let magic = b"APK Sig Block 42";
    if data.len() < magic_pos + 16 || &data[magic_pos..magic_pos + 16] != magic {
        return Err("APK Signing Block magic not found".to_string());
    }

    // block_size (u64) is at magic_pos - 8
    let block_size_pos = magic_pos.checked_sub(8).ok_or("No room for block size")?;
    let block_size = read_u64_le(data, block_size_pos) as usize;

    // Block starts at cd_offset - block_size - 8
    let block_start = cd_offset
        .checked_sub(block_size + 8)
        .ok_or("Signing block extends before tail buffer")?;

    // Skip first 8 bytes (block_size at start), iterate pairs
    let mut pos = block_start + 8;
    let pairs_end = magic_pos - 8; // before the trailing block_size

    while pos + 12 <= pairs_end {
        let pair_size = read_u64_le(data, pos) as usize;
        let pair_id = read_u32_le(data, pos + 8);
        let pair_data_start = pos + 12;
        let pair_data_end = pos + 8 + pair_size;

        if pair_data_end > pairs_end {
            break;
        }

        // v2 = 0x7109871a, v3 = 0xf05368c0
        if pair_id == 0x7109871a || pair_id == 0xf05368c0 {
            if let Ok(cert) = extract_cert_from_signer_block(data, pair_data_start, pair_data_end)
            {
                return Ok(cert);
            }
        }

        pos = pair_data_end;
    }

    Err("No v2/v3 signer block found".to_string())
}

fn extract_cert_from_signer_block(
    data: &[u8],
    start: usize,
    _end: usize,
) -> Result<Vec<u8>, String> {
    // signers: length-prefixed sequence
    let mut pos = start;
    if pos + 4 > data.len() {
        return Err("truncated".to_string());
    }

    let signers_len = read_u32_le(data, pos) as usize;
    pos += 4;
    let signers_end = pos + signers_len;
    if signers_end > data.len() {
        return Err("truncated signers".to_string());
    }

    // First signer
    if pos + 4 > signers_end {
        return Err("no signer".to_string());
    }
    let signer_len = read_u32_le(data, pos) as usize;
    pos += 4;
    let _signer_end = pos + signer_len;

    // signed_data
    if pos + 4 > data.len() {
        return Err("truncated signed_data len".to_string());
    }
    let signed_data_len = read_u32_le(data, pos) as usize;
    pos += 4;
    let _signed_data_end = pos + signed_data_len;

    // digests (skip)
    if pos + 4 > data.len() {
        return Err("truncated digests".to_string());
    }
    let digests_len = read_u32_le(data, pos) as usize;
    pos += 4 + digests_len;

    // certificates
    if pos + 4 > data.len() {
        return Err("truncated certs len".to_string());
    }
    let certs_len = read_u32_le(data, pos) as usize;
    pos += 4;
    let certs_end = pos + certs_len;
    if certs_end > data.len() {
        return Err("truncated certs".to_string());
    }

    // First certificate
    if pos + 4 > certs_end {
        return Err("no cert".to_string());
    }
    let cert_len = read_u32_le(data, pos) as usize;
    pos += 4;
    if pos + cert_len > certs_end {
        return Err("truncated cert data".to_string());
    }

    Ok(data[pos..pos + cert_len].to_vec())
}

fn find_eocd(data: &[u8]) -> Option<usize> {
    let len = data.len();
    // EOCD can have a comment up to 65535 bytes, search backwards
    let search_start = if len > 65557 { len - 65557 } else { 0 };
    for i in (search_start..len.saturating_sub(3)).rev() {
        if data[i] == 0x50
            && data[i + 1] == 0x4b
            && data[i + 2] == 0x05
            && data[i + 3] == 0x06
        {
            return Some(i);
        }
    }
    None
}

fn read_u32_le(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

fn read_u64_le(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ])
}
