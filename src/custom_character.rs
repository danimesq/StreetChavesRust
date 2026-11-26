//! Platform-neutral custom-character package validation and activation.
//!
//! The game core consumes package bytes supplied by a host. Native hosts may
//! read a directory from disk; browser/WASM hosts may supply files selected by
//! the user. No filesystem, DOM, SDL, or JavaScript type enters this module.

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const MANIFEST_FILE: &str = "character.json";
pub const CURRENT_SCHEMA_VERSION: u64 = 1;
pub const MAX_PACKAGE_BYTES: usize = 32 * 1024 * 1024;
pub const MAX_ASSET_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_PACKAGE_FILES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageFile {
    pub path: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharacterManifest {
    pub schema_version: u64,
    pub id: String,
    pub display_name: String,
    pub version: String,
    pub author: Option<String>,
    pub fighter: Value,
    pub assets: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomCharacterPackage {
    pub manifest: CharacterManifest,
    files: BTreeMap<String, Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageError {
    EmptyPackage,
    TooManyFiles {
        actual: usize,
        maximum: usize,
    },
    PackageTooLarge {
        actual: usize,
        maximum: usize,
    },
    AssetTooLarge {
        path: String,
        actual: usize,
        maximum: usize,
    },
    UnsafePath(String),
    DuplicatePath(String),
    MissingManifest,
    InvalidManifest(String),
    UnsupportedSchema(u64),
    InvalidCharacterId(String),
    MissingAsset {
        id: String,
        path: String,
    },
    DuplicateCharacterId(String),
    CharacterNotFound(String),
}

impl std::fmt::Display for PackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPackage => write!(f, "package contains no files"),
            Self::TooManyFiles { actual, maximum } => {
                write!(f, "package has {actual} files; maximum is {maximum}")
            }
            Self::PackageTooLarge { actual, maximum } => {
                write!(f, "package is {actual} bytes; maximum is {maximum}")
            }
            Self::AssetTooLarge {
                path,
                actual,
                maximum,
            } => write!(f, "asset {path:?} is {actual} bytes; maximum is {maximum}"),
            Self::UnsafePath(path) => write!(f, "unsafe package path: {path:?}"),
            Self::DuplicatePath(path) => write!(f, "duplicate package path: {path:?}"),
            Self::MissingManifest => write!(f, "package is missing {MANIFEST_FILE}"),
            Self::InvalidManifest(reason) => write!(f, "invalid character manifest: {reason}"),
            Self::UnsupportedSchema(version) => {
                write!(f, "unsupported character schema version: {version}")
            }
            Self::InvalidCharacterId(id) => write!(f, "invalid character id: {id:?}"),
            Self::MissingAsset { id, path } => {
                write!(f, "asset {id:?} references missing file {path:?}")
            }
            Self::DuplicateCharacterId(id) => {
                write!(f, "character package {id:?} is already installed")
            }
            Self::CharacterNotFound(id) => write!(f, "character package {id:?} is not installed"),
        }
    }
}
impl std::error::Error for PackageError {}

impl CustomCharacterPackage {
    /// Validate package files supplied by any host, including browser/WASM.
    /// Paths are package-relative; traversal and absolute paths are rejected.
    pub fn from_files(files: impl IntoIterator<Item = PackageFile>) -> Result<Self, PackageError> {
        let mut indexed = BTreeMap::new();
        let mut total_bytes = 0usize;
        let mut count = 0usize;
        for file in files {
            count += 1;
            if count > MAX_PACKAGE_FILES {
                return Err(PackageError::TooManyFiles {
                    actual: count,
                    maximum: MAX_PACKAGE_FILES,
                });
            }
            validate_relative_path(&file.path)?;
            if file.path != MANIFEST_FILE && file.bytes.len() > MAX_ASSET_BYTES {
                return Err(PackageError::AssetTooLarge {
                    path: file.path,
                    actual: file.bytes.len(),
                    maximum: MAX_ASSET_BYTES,
                });
            }
            total_bytes = total_bytes.saturating_add(file.bytes.len());
            if total_bytes > MAX_PACKAGE_BYTES {
                return Err(PackageError::PackageTooLarge {
                    actual: total_bytes,
                    maximum: MAX_PACKAGE_BYTES,
                });
            }
            let path = file.path;
            if indexed.insert(path.clone(), file.bytes).is_some() {
                return Err(PackageError::DuplicatePath(path));
            }
        }
        if indexed.is_empty() {
            return Err(PackageError::EmptyPackage);
        }
        let manifest_bytes = indexed
            .get(MANIFEST_FILE)
            .ok_or(PackageError::MissingManifest)?;
        let manifest_json: Value = serde_json::from_slice(manifest_bytes)
            .map_err(|error| PackageError::InvalidManifest(error.to_string()))?;
        let manifest = parse_manifest(&manifest_json)?;
        for (id, path) in &manifest.assets {
            if !indexed.contains_key(path) {
                return Err(PackageError::MissingAsset {
                    id: id.clone(),
                    path: path.clone(),
                });
            }
        }
        Ok(Self {
            manifest,
            files: indexed,
        })
    }

    /// Asset bytes are decoded by the native or browser presentation host.
    pub fn asset_bytes(&self, asset_id: &str) -> Option<&[u8]> {
        let path = self.manifest.assets.get(asset_id)?;
        self.files.get(path).map(Vec::as_slice)
    }

    pub fn file_paths(&self) -> impl Iterator<Item = &str> {
        self.files.keys().map(String::as_str)
    }
}

#[derive(Debug, Clone, Default)]
pub struct CustomCharacterRegistry {
    packages: BTreeMap<String, CustomCharacterPackage>,
    enabled: BTreeSet<String>,
}

impl CustomCharacterRegistry {
    pub fn install(&mut self, package: CustomCharacterPackage) -> Result<(), PackageError> {
        let id = package.manifest.id.clone();
        if self.packages.contains_key(&id) {
            return Err(PackageError::DuplicateCharacterId(id));
        }
        self.packages.insert(id, package);
        Ok(())
    }

    pub fn enable(&mut self, id: &str) -> Result<(), PackageError> {
        if !self.packages.contains_key(id) {
            return Err(PackageError::CharacterNotFound(id.to_owned()));
        }
        self.enabled.insert(id.to_owned());
        Ok(())
    }

    pub fn disable(&mut self, id: &str) -> Result<(), PackageError> {
        if !self.packages.contains_key(id) {
            return Err(PackageError::CharacterNotFound(id.to_owned()));
        }
        self.enabled.remove(id);
        Ok(())
    }

    pub fn is_enabled(&self, id: &str) -> bool {
        self.enabled.contains(id)
    }
    pub fn get(&self, id: &str) -> Option<&CustomCharacterPackage> {
        self.packages.get(id)
    }
    pub fn enabled(&self) -> impl Iterator<Item = &CustomCharacterPackage> {
        self.enabled.iter().filter_map(|id| self.packages.get(id))
    }
}

fn parse_manifest(value: &Value) -> Result<CharacterManifest, PackageError> {
    let object = value
        .as_object()
        .ok_or_else(|| PackageError::InvalidManifest("root must be a JSON object".to_owned()))?;
    let schema_version = object
        .get("schema_version")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            PackageError::InvalidManifest("schema_version must be an unsigned integer".to_owned())
        })?;
    if schema_version != CURRENT_SCHEMA_VERSION {
        return Err(PackageError::UnsupportedSchema(schema_version));
    }
    let string_field = |name: &str| -> Result<String, PackageError> {
        object
            .get(name)
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| PackageError::InvalidManifest(format!("{name} must be a string")))
    };
    let id = string_field("id")?;
    if id.is_empty()
        || id.len() > 64
        || !id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
    {
        return Err(PackageError::InvalidCharacterId(id));
    }
    let fighter = object
        .get("fighter")
        .filter(|v| v.is_object())
        .cloned()
        .ok_or_else(|| PackageError::InvalidManifest("fighter must be a JSON object".to_owned()))?;
    let asset_values = object
        .get("assets")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            PackageError::InvalidManifest(
                "assets must be an object of asset-id to path mappings".to_owned(),
            )
        })?;
    let mut assets = BTreeMap::new();
    for (asset_id, path_value) in asset_values {
        if asset_id.is_empty() || asset_id.len() > 64 {
            return Err(PackageError::InvalidManifest(format!(
                "invalid asset id {asset_id:?}"
            )));
        }
        let path = path_value
            .as_str()
            .ok_or_else(|| {
                PackageError::InvalidManifest(format!("asset {asset_id:?} path must be a string"))
            })?
            .to_owned();
        validate_relative_path(&path)?;
        assets.insert(asset_id.clone(), path);
    }
    Ok(CharacterManifest {
        schema_version,
        id,
        display_name: string_field("display_name")?,
        version: string_field("version")?,
        author: object
            .get("author")
            .and_then(Value::as_str)
            .map(str::to_owned),
        fighter,
        assets,
    })
}

fn validate_relative_path(path: &str) -> Result<(), PackageError> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.contains(':')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || path.chars().any(char::is_control)
    {
        return Err(PackageError::UnsafePath(path.to_owned()));
    }
    Ok(())
}

/// Native directory adapter. WASM hosts pass selected file bytes to from_files.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_directory(
    root: impl AsRef<std::path::Path>,
) -> Result<CustomCharacterPackage, PackageError> {
    use std::fs;
    use std::path::Path;
    fn visit(
        root: &Path,
        current: &Path,
        files: &mut Vec<PackageFile>,
    ) -> Result<(), PackageError> {
        let entries =
            fs::read_dir(current).map_err(|e| PackageError::InvalidManifest(e.to_string()))?;
        for entry in entries {
            let entry = entry.map_err(|e| PackageError::InvalidManifest(e.to_string()))?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|e| PackageError::InvalidManifest(e.to_string()))?;
            if metadata.file_type().is_symlink() {
                return Err(PackageError::UnsafePath(path.display().to_string()));
            }
            if metadata.is_dir() {
                visit(root, &path, files)?;
            } else if metadata.is_file() {
                let relative = path
                    .strip_prefix(root)
                    .map_err(|_| PackageError::UnsafePath(path.display().to_string()))?;
                let relative = relative
                    .to_str()
                    .ok_or_else(|| PackageError::UnsafePath(path.display().to_string()))?
                    .replace(std::path::MAIN_SEPARATOR, "/");
                let bytes =
                    fs::read(&path).map_err(|e| PackageError::InvalidManifest(e.to_string()))?;
                files.push(PackageFile {
                    path: relative,
                    bytes,
                });
            }
        }
        Ok(())
    }
    let root = root.as_ref();
    let mut files = Vec::new();
    visit(root, root, &mut files)?;
    CustomCharacterPackage::from_files(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn files() -> Vec<PackageFile> {
        vec![
            PackageFile { path: MANIFEST_FILE.to_owned(), bytes: br#"{
                "schema_version":1,"id":"chapolin","display_name":"Chapolin Colorado",
                "author":"StreetChaves custom character example","version":"1.0.0",
                "fighter":{"attributes":{"walk_speed":9.0,"sprite_height":199},"moves":[],"animations":{}},
                "assets":{"fighter_strip":"assets/chapolin.png"}
            }"#.to_vec() },
            PackageFile { path:"assets/chapolin.png".to_owned(), bytes:vec![1,2,3] },
        ]
    }
    #[test]
    fn validates_and_activates_package_from_host_supplied_files() {
        let package = CustomCharacterPackage::from_files(files()).unwrap();
        assert_eq!(package.manifest.id, "chapolin");
        assert_eq!(package.asset_bytes("fighter_strip"), Some(&[1, 2, 3][..]));
        let mut registry = CustomCharacterRegistry::default();
        registry.install(package).unwrap();
        assert!(!registry.is_enabled("chapolin"));
        registry.enable("chapolin").unwrap();
        assert!(registry.is_enabled("chapolin"));
        registry.disable("chapolin").unwrap();
        assert!(!registry.is_enabled("chapolin"));
    }
    #[test]
    fn rejects_path_traversal() {
        let mut input = files();
        input[1].path = "../outside.png".to_owned();
        assert_eq!(
            CustomCharacterPackage::from_files(input).unwrap_err(),
            PackageError::UnsafePath("../outside.png".to_owned())
        );
    }
    #[test]
    fn rejects_missing_assets() {
        let mut input = files();
        input.pop();
        assert!(matches!(
            CustomCharacterPackage::from_files(input),
            Err(PackageError::MissingAsset { .. })
        ));
    }
    #[test]
    fn rejects_duplicate_ids() {
        let package = CustomCharacterPackage::from_files(files()).unwrap();
        let mut registry = CustomCharacterRegistry::default();
        registry.install(package.clone()).unwrap();
        assert_eq!(
            registry.install(package).unwrap_err(),
            PackageError::DuplicateCharacterId("chapolin".to_owned())
        );
    }
    #[test]
    fn rejects_unknown_schema() {
        let mut input = files();
        let text = String::from_utf8(input[0].bytes.clone())
            .unwrap()
            .replace("\"schema_version\":1", "\"schema_version\":2");
        input[0].bytes = text.into_bytes();
        assert_eq!(
            CustomCharacterPackage::from_files(input).unwrap_err(),
            PackageError::UnsupportedSchema(2)
        );
    }
}
