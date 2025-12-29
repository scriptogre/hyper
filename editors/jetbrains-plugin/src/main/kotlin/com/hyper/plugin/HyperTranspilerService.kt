package com.hyper.plugin

import com.intellij.openapi.components.Service
import com.intellij.openapi.components.service
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import java.io.File
import java.io.OutputStreamWriter
import java.nio.file.Files
import java.nio.file.StandardCopyOption
import java.util.concurrent.TimeUnit

/**
 * Service that calls the Rust hyper CLI to transpile .hyper to Python.
 * Uses a bundled binary extracted from plugin resources, with fallback to system PATH.
 * Caches results to avoid duplicate transpiler calls from multiple injectors.
 */
@Service(Service.Level.PROJECT)
class HyperTranspilerService(private val project: Project) {

    companion object {
        private val LOG = Logger.getInstance(HyperTranspilerService::class.java)
        private val json = Json { ignoreUnknownKeys = true }

        fun getInstance(project: Project): HyperTranspilerService = project.service()

        // Cache the extracted binary path across all projects
        @Volatile
        private var extractedBinaryPath: String? = null

        private fun getPlatformBinaryName(): String {
            val os = System.getProperty("os.name").lowercase()
            val arch = System.getProperty("os.arch").lowercase()

            val osName = when {
                os.contains("mac") || os.contains("darwin") -> "darwin"
                os.contains("win") -> "windows"
                os.contains("linux") -> "linux"
                else -> "unknown"
            }

            val archName = when {
                arch.contains("aarch64") || arch.contains("arm64") -> "arm64"
                arch.contains("amd64") || arch.contains("x86_64") -> "x64"
                else -> "x64"
            }

            val ext = if (osName == "windows") ".exe" else ""
            return "hyper-$osName-$archName$ext"
        }
    }

    // Cache: content hash -> result (single entry, replaced on new content)
    @Volatile
    private var cachedContentHash: Int = 0
    @Volatile
    private var cachedResult: TranspileResult? = null

    @Serializable
    data class Mapping(
        val gen_line: Int,
        val gen_col: Int,
        val src_line: Int,
        val src_col: Int
    )

    @Serializable
    data class PythonInjection(
        val start: Int,
        val end: Int,
        val prefix: String,
        val suffix: String
    )

    @Serializable
    data class HtmlInjection(
        val start: Int,
        val end: Int
    )

    @Serializable
    data class Injections(
        val python: List<PythonInjection> = emptyList(),
        val html: List<HtmlInjection> = emptyList()
    )

    @Serializable
    data class TranspileResult(
        val code: String,
        val mappings: List<Mapping>,
        val injections: Injections? = null
    )

    fun transpile(content: String, includeInjection: Boolean = false): TranspileResult {
        // Check cache first (only for injection mode which is called by multiple injectors)
        if (includeInjection) {
            val contentHash = content.hashCode()
            val cached = cachedResult
            if (cached != null && cachedContentHash == contentHash) {
                LOG.debug("Using cached transpile result")
                return cached
            }
        }

        val hyperPath = findHyperBinary()
            ?: throw TranspileException("Could not find 'hyper' binary. No bundled binary for this platform and none found in PATH.")

        LOG.debug("Using hyper binary: $hyperPath")

        val args = mutableListOf(hyperPath, "generate", "--stdin", "--json")
        if (includeInjection) {
            args.add("--injection")
        }

        val processBuilder = ProcessBuilder(args)
            .redirectErrorStream(true)  // Merge stderr into stdout to avoid separate buffer

        val process = processBuilder.start()

        // Write input and close stdin in a separate thread to avoid deadlock
        val writeThread = Thread {
            try {
                process.outputStream.bufferedWriter().use { writer ->
                    writer.write(content)
                }
            } catch (e: Exception) {
                LOG.debug("Error writing to process stdin", e)
            }
        }
        writeThread.start()

        // Read output while process runs (prevents buffer deadlock)
        val output = process.inputStream.bufferedReader().readText()

        // Wait for write thread to finish
        writeThread.join(1000)

        val completed = process.waitFor(10, TimeUnit.SECONDS)
        if (!completed) {
            process.destroyForcibly()
            throw TranspileException("Transpiler timed out")
        }

        if (process.exitValue() != 0) {
            throw TranspileException("Transpiler failed: $output")
        }

        val result = json.decodeFromString<TranspileResult>(output)

        // Cache the result for injection mode
        if (includeInjection) {
            cachedContentHash = content.hashCode()
            cachedResult = result
        }

        return result
    }

    private fun findHyperBinary(): String? {
        // First, try to use cached extracted binary
        extractedBinaryPath?.let { path ->
            if (File(path).canExecute()) {
                return path
            }
        }

        // Try to extract bundled binary
        extractBundledBinary()?.let { path ->
            extractedBinaryPath = path
            return path
        }

        // Fallback to system locations
        return findSystemBinary()
    }

    private fun extractBundledBinary(): String? {
        val binaryName = getPlatformBinaryName()
        val resourcePath = "/bin/$binaryName"

        val inputStream = javaClass.getResourceAsStream(resourcePath)
        if (inputStream == null) {
            LOG.info("No bundled binary found at $resourcePath")
            return null
        }

        return try {
            // Extract to a temp directory that persists across IDE restarts
            val cacheDir = File(System.getProperty("java.io.tmpdir"), "hyper-plugin")
            cacheDir.mkdirs()

            val targetFile = File(cacheDir, binaryName)

            // Only extract if not already present or if plugin was updated
            // For simplicity, always extract (it's fast)
            inputStream.use { stream ->
                Files.copy(stream, targetFile.toPath(), StandardCopyOption.REPLACE_EXISTING)
            }

            // Make executable on Unix
            if (!System.getProperty("os.name").lowercase().contains("win")) {
                targetFile.setExecutable(true)
            }

            LOG.info("Extracted bundled binary to: ${targetFile.absolutePath}")
            targetFile.absolutePath
        } catch (e: Exception) {
            LOG.warn("Failed to extract bundled binary", e)
            null
        }
    }

    private fun findSystemBinary(): String? {
        val homeDir = System.getProperty("user.home")

        // Check common locations
        val candidates = listOf(
            "$homeDir/.cargo/bin/hyper",
            "$homeDir/.local/bin/hyper",
            "/usr/local/bin/hyper",
            "/opt/homebrew/bin/hyper",
        )

        for (path in candidates) {
            if (File(path).canExecute()) {
                LOG.info("Found system binary at: $path")
                return path
            }
        }

        // Try PATH using platform-appropriate command
        val isWindows = System.getProperty("os.name").lowercase().contains("win")
        val cmd = if (isWindows) listOf("where", "hyper") else listOf("which", "hyper")
        return try {
            val process = ProcessBuilder(cmd).start()
            if (process.waitFor(2, TimeUnit.SECONDS) && process.exitValue() == 0) {
                process.inputStream.bufferedReader().readLine()?.trim()
            } else null
        } catch (e: Exception) {
            null
        }
    }

    class TranspileException(message: String) : Exception(message)
}
