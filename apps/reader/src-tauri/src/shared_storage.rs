//! Mandatory common large-resource storage policy and the Android mobile
//! bridge that backs it.
//!
//! Large resources (DynDon packs, model files) live under one organization
//! namespace (`mba.robin`) on a physical medium the user chooses (internal
//! storage or an SD card). Sharing is fixed at build time through `.env`
//! keys compiled in by `build.rs`; there is no app-private fallback and no
//! optional sharing. Foreground apps open the common files directly — no
//! background model service exists here.
//!
//! The Android access-recovery path is a small Tauri mobile plugin
//! (`mba.robin.enzime.SharedStoragePlugin`, Kotlin) registered from
//! `init()`: it enumerates actually mounted volumes and opens the system's
//! storage-permission screen on an explicit user action. Until access is
//! granted, storage enumeration fails with the actionable
//! `shared_storage_access_required` error; catalog browsing is unaffected.

use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;
#[cfg(target_os = "android")]
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

/// Organization storage namespace compiled in by `build.rs`.
pub const STORAGE_NAMESPACE: &str = env!("ENZIME_STORAGE_NAMESPACE");
/// Optional compiled-in desktop common root override ("" = platform default).
const COMPILED_PRIMARY_ROOT: &str = env!("ENZIME_SHARED_STORAGE_ROOT");
/// Optional compiled-in Android primary common root override ("" = default).
const COMPILED_PRIMARY_ROOT_ANDROID: &str = env!("ENZIME_SHARED_STORAGE_ROOT_ANDROID");
/// Optional compiled-in configured SD-card common root ("" = none).
const COMPILED_SDCARD_ROOT: &str = env!("ENZIME_SHARED_SDCARD_ROOT");

/// Stable error-code prefixes the frontend can match (see SHARE-2).
pub const ACCESS_REQUIRED: &str = "shared_storage_access_required";
pub const ROOT_NOT_SHARED: &str = "shared_storage_root_not_shared";
pub const BUSY: &str = "shared_storage_busy";

/// Physical medium holding a shared storage root.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageMedium {
    Internal,
    Sdcard,
}

impl StorageMedium {
    /// Friendly name shown to the user (SHARE-2 wording).
    pub fn label(self) -> &'static str {
        match self {
            StorageMedium::Internal => "Internal storage",
            StorageMedium::Sdcard => "SD card",
        }
    }

    /// §6.2 `storage_roots.kind` for this medium.
    pub fn root_kind(self) -> &'static str {
        match self {
            StorageMedium::Internal => "user_selected",
            StorageMedium::Sdcard => "removable",
        }
    }
}

/// One validated shared storage root on a physical medium.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedStorageRoot {
    pub path: PathBuf,
    pub medium: StorageMedium,
}

/// The compiled shared-storage policy. Held by `AppState`; the compiled
/// primary root and optional configured card are immutable, while the real
/// mounted roots are cached from the platform (Android volume enumeration).
pub struct SharedStoragePolicy {
    primary: SharedStorageRoot,
    configured_card: Option<SharedStorageRoot>,
    mounted_roots: Mutex<Vec<SharedStorageRoot>>,
}

fn validate_common_root(raw: &str, app_data_dir: &Path) -> Result<PathBuf, String> {
    let path = PathBuf::from(raw);
    if !path.is_absolute() {
        return Err(format!(
            "shared_storage_invalid_root: '{raw}' is not an absolute common storage path"
        ));
    }
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("shared_storage_invalid_root: '{raw}' has no namespace component"))?;
    if file_name != STORAGE_NAMESPACE {
        return Err(format!(
            "shared_storage_invalid_root: '{raw}' must end in the shared storage namespace \
             '{STORAGE_NAMESPACE}'"
        ));
    }
    reject_app_private(&path, app_data_dir)?;
    Ok(path)
}

fn reject_app_private(path: &Path, app_data_dir: &Path) -> Result<(), String> {
    if app_data_dir.is_absolute() && path.starts_with(app_data_dir) {
        return Err(format!(
            "shared_storage_invalid_root: '{}' is inside the app-private directory '{}'; \
             app-private locations can never be shared storage",
            path.display(),
            app_data_dir.display()
        ));
    }
    Ok(())
}

/// The namespace must be exactly one normal path component on every build
/// host: both slash kinds, `.`, `..`, drive prefixes and empty strings are
/// rejected so a configured root can never smuggle in extra components.
fn is_single_component(namespace: &str) -> bool {
    if namespace.is_empty()
        || namespace == "."
        || namespace == ".."
        || namespace.contains('/')
        || namespace.contains('\\')
    {
        return false;
    }
    let mut components = Path::new(namespace).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

impl SharedStoragePolicy {
    /// Build the compiled policy. `app_data_dir` is the resolved platform
    /// app-data directory, used only to derive the desktop default common
    /// primary (its parent plus the namespace) and to reject app-private
    /// descendants; Android defaults to `/storage/emulated/0/<namespace>`.
    pub fn new(app_data_dir: &Path) -> Result<Self, String> {
        if !is_single_component(STORAGE_NAMESPACE) {
            return Err(format!(
                "shared_storage_invalid_configuration: storage namespace \
                 '{STORAGE_NAMESPACE}' must be a single path component"
            ));
        }
        let primary_path = if cfg!(target_os = "android") {
            if COMPILED_PRIMARY_ROOT_ANDROID.is_empty() {
                PathBuf::from("/storage/emulated/0").join(STORAGE_NAMESPACE)
            } else {
                validate_common_root(COMPILED_PRIMARY_ROOT_ANDROID, app_data_dir)?
            }
        } else if COMPILED_PRIMARY_ROOT.is_empty() {
            // Desktop default: the platform data directory's parent plus the
            // org namespace — resolved independently of the app identifier.
            let parent = app_data_dir
                .parent()
                .ok_or_else(|| {
                    format!(
                        "shared_storage_invalid_configuration: app-data directory '{}' has no \
                         parent to derive the shared root from",
                        app_data_dir.display()
                    )
                })?
                .to_path_buf();
            reject_app_private(&parent, app_data_dir)?;
            parent.join(STORAGE_NAMESPACE)
        } else {
            validate_common_root(COMPILED_PRIMARY_ROOT, app_data_dir)?
        };
        let configured_card = if COMPILED_SDCARD_ROOT.is_empty() {
            None
        } else {
            Some(SharedStorageRoot {
                path: validate_common_root(COMPILED_SDCARD_ROOT, app_data_dir)?,
                medium: StorageMedium::Sdcard,
            })
        };
        Ok(Self {
            primary: SharedStorageRoot {
                path: primary_path,
                medium: StorageMedium::Internal,
            },
            configured_card,
            mounted_roots: Mutex::new(Vec::new()),
        })
    }

    /// The compiled primary common root.
    pub fn primary(&self) -> &SharedStorageRoot {
        &self.primary
    }

    /// The compiled optional configured card, if any.
    pub fn configured_card(&self) -> Option<&SharedStorageRoot> {
        self.configured_card.as_ref()
    }

    /// Enumerate the real shared roots currently available on this device,
    /// refreshing the mounted-roots cache. Namespace directories are created
    /// and canonical paths deduplicated by the caller (`storage_roots_list`)
    /// after this enumeration, so a fresh card may appear here twice (via the
    /// compiled configuration and volume enumeration) until the caller
    /// collapses it to one row. A card root appears only when its mount
    /// parent already exists (a mount point is never created).
    pub async fn roots(&self) -> Result<Vec<SharedStorageRoot>, String> {
        let mut roots = Vec::new();
        #[cfg(target_os = "android")]
        {
            let volumes = android_volumes().await?;
            if !volumes.access_granted {
                return Err(format!(
                    "{ACCESS_REQUIRED}: allow shared storage access so EnZIME can use the \
                     common storage location, then return to the app. Tap 'Allow shared \
                     storage access' to open the system storage settings."
                ));
            }
            // The compiled Android override wins; otherwise the primary
            // volume directory plus the shared namespace.
            let primary_dir = volumes
                .primary_directory
                .filter(|dir| !dir.trim().is_empty())
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/storage/emulated/0"));
            let primary_path = if COMPILED_PRIMARY_ROOT_ANDROID.is_empty() {
                primary_dir.join(STORAGE_NAMESPACE)
            } else {
                self.primary.path.clone()
            };
            roots.push(SharedStorageRoot {
                path: primary_path,
                medium: StorageMedium::Internal,
            });
            for dir in volumes.removable_directories {
                let dir = PathBuf::from(dir);
                // Only actually mounted removable volumes are real roots.
                if dir.is_dir() {
                    roots.push(SharedStorageRoot {
                        path: dir.join(STORAGE_NAMESPACE),
                        medium: StorageMedium::Sdcard,
                    });
                }
            }
        }
        #[cfg(not(target_os = "android"))]
        {
            // Desktop: the compiled common primary on the local filesystem.
            roots.push(self.primary.clone());
        }
        // The configured card participates on every platform, but only when
        // its mount parent (the card's mount point) already exists.
        if let Some(card) = &self.configured_card {
            let mount_parent = card
                .path
                .parent()
                .ok_or_else(|| "shared_storage_invalid_root: configured card has no mount parent".to_string())?;
            if mount_parent.is_dir() {
                roots.push(card.clone());
            }
        }
        // Collapse roots that already canonicalize to the same path. Paths
        // that do not canonicalize yet (namespace directory not created) are
        // kept: `storage_roots_list` creates the namespace directories and
        // performs the authoritative canonical deduplication afterwards.
        let mut seen: Vec<PathBuf> = Vec::new();
        roots.retain(|root| {
            match std::fs::canonicalize(&root.path) {
                Ok(canonical) => {
                    if seen.contains(&canonical) {
                        false
                    } else {
                        seen.push(canonical);
                        true
                    }
                }
                Err(_) => true, // not yet created; the caller dedups after creation
            }
        });
        if let Ok(mut cache) = self.mounted_roots.lock() {
            *cache = roots.clone();
        }
        Ok(roots)
    }

    /// Require that `candidate` (an already-canonical storage-root path)
    /// matches a shared root that is current under this build's policy.
    /// Returns its medium, or an actionable error. Historical
    /// `storage_roots` rows cannot authorize new private acquisitions.
    pub async fn require_shared_root(&self, candidate: &Path) -> Result<StorageMedium, String> {
        let roots = self.roots().await?;
        for root in &roots {
            let Ok(canonical) = std::fs::canonicalize(&root.path) else {
                continue;
            };
            if canonical == candidate {
                return Ok(root.medium);
            }
        }
        Err(format!(
            "{ROOT_NOT_SHARED}: '{}' is not a shared storage location in this build. Choose a \
             destination from the shared storage list (internal storage or an SD card) and \
             compute the plan again.",
            candidate.display()
        ))
    }
}

// ===========================================================================
// Android mobile plugin bridge
// ===========================================================================

#[cfg(target_os = "android")]
static PLUGIN: OnceLock<tauri::plugin::PluginHandle<tauri::Wry>> = OnceLock::new();

/// Android-only response of the mobile plugin's `volumes` command; the
/// desktop build has no mobile volumes and never names this type.
#[cfg(target_os = "android")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VolumesResponse {
    #[serde(default)]
    primary_directory: Option<String>,
    #[serde(default)]
    removable_directories: Vec<String>,
    #[serde(default)]
    access_granted: bool,
}

/// Registers the Android Kotlin plugin class `mba.robin.enzime.
/// SharedStoragePlugin` with Tauri's mobile plugin boundary and keeps the
/// resulting `PluginHandle` for the lifetime of the process. Desktop builds
/// register nothing: local shared storage needs no access prompt.
pub fn init() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new("shared-storage")
        .setup(|_app, api| {
            #[cfg(target_os = "android")]
            {
                let handle = api
                    .register_android_plugin("mba.robin.enzime", "SharedStoragePlugin")
                    .map_err(|e| format!("shared_storage_plugin_registration_failed: {e:?}"))?;
                let _ = PLUGIN.set(handle);
            }
            #[cfg(not(target_os = "android"))]
            {
                let _ = api;
            }
            Ok(())
        })
        .build()
}

/// Query the Android plugin for the primary volume directory, actually
/// mounted removable volume directories, and whether shared-file access is
/// granted. API 30+ answers from `StorageVolume`/`isExternalStorageManager`;
/// older supported versions derive volumes from `getExternalFilesDirs` and
/// legacy permissions (see SharedStoragePlugin.kt).
#[cfg(target_os = "android")]
async fn android_volumes() -> Result<VolumesResponse, String> {
    let handle = PLUGIN.get().ok_or_else(|| {
        format!("{ACCESS_REQUIRED}: the shared-storage mobile plugin is not initialized")
    })?;
    handle
        .run_mobile_plugin_async::<VolumesResponse>("volumes", serde_json::json!({}))
        .await
        .map_err(|e| format!("shared_storage_volumes_failed: {e:?}"))
}

/// Open the system's shared-storage access recovery flow. Android delegates
/// to the mobile plugin, which opens the storage-permission screen; desktop
/// needs no access prompt and succeeds immediately. The §14.4 Tauri command
/// `shared_storage_request_access` (ipc/library.rs) delegates here.
pub async fn request_access() -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        let handle = PLUGIN.get().ok_or_else(|| {
            "shared_storage_request_failed: the shared-storage mobile plugin is not initialized"
                .to_string()
        })?;
        handle
            .run_mobile_plugin_async::<serde_json::Value>(
                "requestAccess",
                serde_json::json!({}),
            )
            .await
            .map_err(|e| format!("shared_storage_request_failed: {e:?}"))?;
    }
    Ok(())
}

// ===========================================================================
// Cross-process acquisition lease
// ===========================================================================

/// An exclusive lease on `<root>/.acquisition.lock` held for the duration of
/// an acquisition (plan or install), serializing acquisition writes between
/// cooperating ecosystem apps that share the common root. The lock is
/// released when the guard is dropped.
pub struct AcquisitionLease(std::fs::File);

impl AcquisitionLease {
    /// Try to acquire the lease without blocking. A held lock returns an
    /// actionable busy error instead of waiting.
    pub fn acquire(root: &Path) -> Result<Self, String> {
        let path = root.join(".acquisition.lock");
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .read(true)
            .open(&path)
            .map_err(|e| {
                format!(
                    "{BUSY}: cannot open the acquisition lock at '{}' ({e}); verify the \
                     shared storage location is writable and try again.",
                    path.display()
                )
            })?;
        fs2::FileExt::try_lock_exclusive(&file).map_err(|_| {
            format!(
                "{BUSY}: another app is currently acquiring resources in this shared storage \
                 location. Wait for it to finish, then try again."
            )
        })?;
        Ok(Self(file))
    }
}
