package com.hyper.plugin

import com.intellij.openapi.Disposable
import com.intellij.openapi.components.Service
import com.intellij.openapi.components.service
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.Disposer
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import java.io.DataInputStream
import java.io.DataOutputStream
import java.io.File
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.nio.file.Files
import java.nio.file.StandardCopyOption
import java.util.concurrent.TimeUnit
import java.util.concurrent.locks.ReentrantLock
import kotlin.concurrent.withLock

/**
 * Service that calls the Rust hyper CLI to transpile .hyper to Python.
 * Uses daemon mode for instant responses (no process spawn overhead).
 * Falls back to per-request mode if daemon unavailable.
 */
@Service(Service.Level.PROJECT)
class HyperTranspilerService(private val project: Project) : Disposable {

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

    // Daemon process management
    private var daemonProcess: Process? = null
    private var daemonInput: DataInputStream? = null
    private var daemonOutput: DataOutputStream? = null
    private val daemonLock = ReentrantLock()
    private var daemonFailed = false

    // Cache: content hash -> result (single entry, replaced on new content)
    @Volatile
    private var cachedContentHash: Int = 0
    @Volatile
    private var cachedResult: TranspileResult? = null

    init {
        Disposer.register(project, this)
    }

    override fun dispose() {
        shutdownDaemon()
    }

    @Serializable
    data class Mapping(
        val gen_line: Int,
        val gen_col: Int,
        val src_line: Int,
        val src_col: Int
    )

    @Serializable
    data class Range(
        val type: String,  // "python" or "html"
        val source_start: Int,
        val source_end: Int,
        val compiled_start: Int,
        val compiled_end: Int
    )

    @Serializable
    data class InjectionJson(
        val type: String,
        val start: Int,
        val end: Int,
        val prefix: String,
        val suffix: String
    )

    @Serializable
    data class TranspileResultJson(
        val compiled: String,
        val mappings: List<Mapping>,
        val ranges: List<Range>? = null,
        val injections: List<InjectionJson>? = null
    )

    /**
     * Computed injection with prefix/suffix for JetBrains language injection.
     * Computed from Range + compiled code.
     */
    data class Injection(
        val start: Int,       // source start (UTF-16)
        val end: Int,         // source end (UTF-16)
        val prefix: String,
        val suffix: String,
        val type: String      // "python" or "html"
    )

    /**
     * Result with computed injections for IDE use.
     */
    data class TranspileResult(
        val code: String,
        val mappings: List<Mapping>,
        val ranges: List<Range>,
        val injections: List<Injection>
    ) {
        val pythonInjections: List<Injection>
            get() = injections.filter { it.type == "python" }

        val htmlInjections: List<Injection>
            get() = injections.filter { it.type == "html" }
    }

    @Serializable
    private data class DaemonRequest(
        val content: String,
        val injection: Boolean = false,
        val name: String? = null
    )

    /**
     * Parse JSON response - injections are computed by the transpiler.
     */
    private fun parseResponse(jsonString: String): TranspileResult {
        val parsed = json.decodeFromString<TranspileResultJson>(jsonString)
        val ranges = parsed.ranges ?: emptyList()

        // Use injections from transpiler (already computed with prefix/suffix)
        val injections = parsed.injections?.map { inj ->
            Injection(
                start = inj.start,
                end = inj.end,
                prefix = inj.prefix,
                suffix = inj.suffix,
                type = inj.type
            )
        } ?: emptyList()

        return TranspileResult(
            code = parsed.compiled,
            mappings = parsed.mappings,
            ranges = ranges,
            injections = injections
        )
    }

    fun transpile(content: String, includeInjection: Boolean = false, functionName: String? = null): TranspileResult {
        // Check cache first (only for injection mode which is called by multiple injectors)
        if (includeInjection) {
            val contentHash = content.hashCode()
            val cached = cachedResult
            if (cached != null && cachedContentHash == contentHash) {
                LOG.debug("Using cached transpile result")
                return cached
            }
        }

        // Try daemon mode first (much faster)
        if (!daemonFailed) {
            try {
                val result = transpileViaDaemon(content, includeInjection, functionName)
                if (includeInjection) {
                    cachedContentHash = content.hashCode()
                    cachedResult = result
                }
                return result
            } catch (e: Exception) {
                LOG.warn("Daemon mode failed, falling back to per-request mode", e)
                daemonFailed = true
                shutdownDaemon()
            }
        }

        // Fallback to per-request mode
        return transpileViaProcess(content, includeInjection, functionName)
    }

    private fun transpileViaDaemon(content: String, includeInjection: Boolean, functionName: String?): TranspileResult {
        daemonLock.withLock {
            ensureDaemonRunning()

            val output = daemonOutput ?: throw TranspileException("Daemon not available")
            val input = daemonInput ?: throw TranspileException("Daemon not available")

            // Build request
            val request = DaemonRequest(
                content = content,
                injection = includeInjection,
                name = functionName
            )
            val requestJson = json.encodeToString(DaemonRequest.serializer(), request)
            val requestBytes = requestJson.toByteArray(Charsets.UTF_8)

            // Write length-prefixed request
            val lenBuf = ByteBuffer.allocate(4).order(ByteOrder.BIG_ENDIAN)
            lenBuf.putInt(requestBytes.size)
            output.write(lenBuf.array())
            output.write(requestBytes)
            output.flush()

            // Read response with timeout (10 seconds)
            val executor = java.util.concurrent.Executors.newSingleThreadExecutor()
            try {
                val future = executor.submit<String> {
                    val respLenBytes = ByteArray(4)
                    input.readFully(respLenBytes)
                    val respLen = ByteBuffer.wrap(respLenBytes).order(ByteOrder.BIG_ENDIAN).getInt()

                    val respBytes = ByteArray(respLen)
                    input.readFully(respBytes)
                    String(respBytes, Charsets.UTF_8)
                }

                val respJson = future.get(10, TimeUnit.SECONDS)
                return parseResponse(respJson)
            } catch (e: java.util.concurrent.TimeoutException) {
                // Daemon hung, restart it next time
                shutdownDaemon()
                throw TranspileException("Daemon response timed out")
            } catch (e: Exception) {
                shutdownDaemon()
                throw TranspileException("Daemon communication failed: ${e.message}")
            } finally {
                executor.shutdown()
            }
        }
    }

    private fun ensureDaemonRunning() {
        val process = daemonProcess
        if (process != null && process.isAlive) {
            return
        }

        val hyperPath = findHyperBinary()
            ?: throw TranspileException("Could not find 'hyper' binary")

        LOG.info("Starting hyper daemon: $hyperPath")

        val processBuilder = ProcessBuilder(hyperPath, "generate", "--daemon")
        val newProcess = processBuilder.start()

        val newInput = DataInputStream(newProcess.inputStream)
        val newOutput = DataOutputStream(newProcess.outputStream)

        // Read ready message with timeout (5 seconds)
        val executor = java.util.concurrent.Executors.newSingleThreadExecutor()
        try {
            val future = executor.submit<String> {
                val readyLenBytes = ByteArray(4)
                newInput.readFully(readyLenBytes)
                val readyLen = ByteBuffer.wrap(readyLenBytes).order(ByteOrder.BIG_ENDIAN).getInt()
                val readyBytes = ByteArray(readyLen)
                newInput.readFully(readyBytes)
                String(readyBytes, Charsets.UTF_8)
            }

            val readyMsg = future.get(5, TimeUnit.SECONDS)
            LOG.info("Daemon ready: $readyMsg")

            daemonProcess = newProcess
            daemonInput = newInput
            daemonOutput = newOutput
        } catch (e: java.util.concurrent.TimeoutException) {
            newProcess.destroyForcibly()
            throw TranspileException("Daemon startup timed out")
        } catch (e: Exception) {
            newProcess.destroyForcibly()
            throw TranspileException("Daemon startup failed: ${e.message}")
        } finally {
            executor.shutdown()
        }
    }

    private fun shutdownDaemon() {
        daemonLock.withLock {
            try {
                daemonOutput?.close()
                daemonInput?.close()
                daemonProcess?.destroyForcibly()
            } catch (e: Exception) {
                LOG.debug("Error shutting down daemon", e)
            }
            daemonProcess = null
            daemonInput = null
            daemonOutput = null
        }
    }

    private fun transpileViaProcess(content: String, includeInjection: Boolean, functionName: String?): TranspileResult {
        val hyperPath = findHyperBinary()
            ?: throw TranspileException("Could not find 'hyper' binary. No bundled binary for this platform and none found in PATH.")

        LOG.debug("Using hyper binary (per-request): $hyperPath")

        val args = mutableListOf(hyperPath, "generate", "--stdin", "--json")
        if (includeInjection) {
            args.add("--injection")
        }
        if (functionName != null) {
            args.add("--name")
            args.add(functionName)
        }

        val processBuilder = ProcessBuilder(args)
            .redirectErrorStream(true)

        val process = processBuilder.start()

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

        val output = process.inputStream.bufferedReader().readText()
        writeThread.join(1000)

        val completed = process.waitFor(10, TimeUnit.SECONDS)
        if (!completed) {
            process.destroyForcibly()
            throw TranspileException("Transpiler timed out")
        }

        if (process.exitValue() != 0) {
            throw TranspileException("Transpiler failed: $output")
        }

        val result = parseResponse(output)

        if (includeInjection) {
            cachedContentHash = content.hashCode()
            cachedResult = result
        }

        return result
    }

    private fun findHyperBinary(): String? {
        extractedBinaryPath?.let { path ->
            if (File(path).canExecute()) {
                return path
            }
        }

        extractBundledBinary()?.let { path ->
            extractedBinaryPath = path
            return path
        }

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
            val cacheDir = File(System.getProperty("java.io.tmpdir"), "hyper-plugin")
            cacheDir.mkdirs()

            val targetFile = File(cacheDir, binaryName)

            inputStream.use { stream ->
                Files.copy(stream, targetFile.toPath(), StandardCopyOption.REPLACE_EXISTING)
            }

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
