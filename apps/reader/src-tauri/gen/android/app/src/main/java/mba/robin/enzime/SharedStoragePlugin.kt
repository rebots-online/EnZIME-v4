// SharedStoragePlugin — Android side of the mandatory common storage policy.
//
// Registered from Rust by shared_storage::init() (plugin name
// "shared-storage") through Tauri's mobile plugin boundary. It provides the
// two @Command entry points the native side invokes:
//
//   volumes      → the primary shared volume directory, the actually mounted
//                  removable volume directories, and whether shared-file
//                  access is granted right now.
//   requestAccess → opens the system's storage permission screen on an
//                  explicit user action (API 30+ all-files screen; legacy
//                  runtime read/write permission below API 30).
//
// There is no background runner or service: both commands always resolve or
// reject the Invoke, and nothing runs after the activity is gone.

package mba.robin.enzime

import android.Manifest
import android.app.Activity
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.Settings
import android.util.Log
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.PluginManager

@TauriPlugin
class SharedStoragePlugin(private val activity: Activity) : Plugin(activity) {

    companion object {
        private const val TAG = "SharedStorage"
    }

    /**
     * Enumerate real shared volumes and the current access state. The
     * response shape is consumed by SharedStoragePolicy::roots():
     *   { primaryDirectory: String|null, removableDirectories: [String],
     *     accessGranted: Boolean }
     */
    @Command
    fun volumes(invoke: Invoke) {
        try {
            val result = JSObject()
            val removable = JSArray()
            if (Build.VERSION.SDK_INT >= 30) {
                result.put("accessGranted", Environment.isExternalStorageManager())
                // StorageVolume.directory is the real mount point (API 30+);
                // only volumes actually in the mounted state are real roots.
                val manager = activity.getSystemService(Activity.STORAGE_SERVICE)
                    as? android.os.storage.StorageManager
                if (manager != null) {
                    result.put(
                        "primaryDirectory",
                        Environment.getExternalStorageDirectory()?.absolutePath
                    )
                    for (volume in manager.storageVolumes) {
                        if (!volume.isRemovable || volume.state != Environment.MEDIA_MOUNTED) {
                            continue
                        }
                        val directory = volume.directory ?: continue
                        removable.put(directory.absolutePath)
                    }
                } else {
                    result.put("primaryDirectory", Environment.getExternalStorageDirectory()?.absolutePath)
                }
            } else {
                // Below API 30, shared-file access is the legacy read/write
                // permission pair, and volumes are derived from each
                // external files dir (/storage/<volume>/Android/data/<pkg>/files).
                result.put("accessGranted", legacyAccessGranted())
                result.put(
                    "primaryDirectory",
                    Environment.getExternalStorageDirectory()?.absolutePath
                )
                val externalFilesDirs = activity.getExternalFilesDirs(null)
                for (dir in externalFilesDirs) {
                    if (dir == null) continue
                    val mount = volumeRootOf(dir.absolutePath) ?: continue
                    if (mount.absolutePath == Environment.getExternalStorageDirectory()?.absolutePath) {
                        continue // the primary emulated volume, not a card
                    }
                    // Only a mounted card is a real removable root.
                    if (mount.isDirectory && Environment.MEDIA_MOUNTED == Environment.getExternalStorageState(mount)) {
                        removable.put(mount.absolutePath)
                    }
                }
            }
            result.put("removableDirectories", removable)
            invoke.resolve(result)
        } catch (e: Exception) {
            invoke.reject("Could not enumerate shared storage volumes: ${e.message}", "volumes_failed", e)
        }
    }

    /**
     * Open the real system recovery flow for shared-file access. API 30+
     * targets this app's all-files entry on the system screen, falling back
     * to the global all-files screen; legacy versions request the actual
     * read/write permission pair through the plugin manager's launcher.
     * Always resolves or rejects; the permission outcome is observed by the
     * next `volumes` call after the user returns.
     */
    @Command
    fun requestAccess(invoke: Invoke) {
        try {
            if (Build.VERSION.SDK_INT >= 30) {
                val packageUri = Uri.parse("package:${activity.packageName}")
                val intent = Intent(
                    Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION,
                    packageUri
                )
                if (intent.resolveActivity(activity.packageManager) != null) {
                    activity.startActivity(intent)
                } else {
                    // OEM builds without the per-app screen: the system
                    // all-files list.
                    activity.startActivity(Intent(Settings.ACTION_MANAGE_ALL_FILES_ACCESS_PERMISSION))
                }
                invoke.resolve()
            } else {
                PluginManager.requestPermissions(
                    arrayOf(
                        Manifest.permission.READ_EXTERNAL_STORAGE,
                        Manifest.permission.WRITE_EXTERNAL_STORAGE
                    )
                ) { permissions ->
                    // The system dialog finished; report what is true now
                    // rather than claiming success.
                    val granted = permissions.values.all { it }
                    if (granted) {
                        invoke.resolve()
                    } else {
                        invoke.reject(
                            "Shared storage permission was not granted. Tap 'Allow shared storage access' again and allow access in system settings.",
                            "access_denied"
                        )
                    }
                }
            }
        } catch (e: Exception) {
            invoke.reject("Could not open the shared storage settings screen: ${e.message}", "request_access_failed", e)
        }
    }

    private fun legacyAccessGranted(): Boolean =
        activity.checkSelfPermission(Manifest.permission.READ_EXTERNAL_STORAGE) == PackageManager.PERMISSION_GRANTED &&
            activity.checkSelfPermission(Manifest.permission.WRITE_EXTERNAL_STORAGE) == PackageManager.PERMISSION_GRANTED

    /** /storage/<volume>/Android/data/<pkg>/files → /storage/<volume> */
    private fun volumeRootOf(externalFilesPath: String): java.io.File? {
        val marker = "/Android/data/"
        val index = externalFilesPath.indexOf(marker)
        if (index <= 0) return null
        return java.io.File(externalFilesPath.substring(0, index))
    }
}
