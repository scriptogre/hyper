package com.hyper.plugin

import com.intellij.openapi.components.Service
import com.intellij.openapi.components.service
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.Project
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import java.io.File
import java.io.OutputStreamWriter
import java.util.concurrent.TimeUnit

/**
 * Service that calls the Rust hyper CLI to transpile .hyper to Python.
 */
@Service(Service.Level.PROJECT)
class HyperTranspilerService(private val project: Project) {

    companion object {
        private val LOG = Logger.getInstance(HyperTranspilerService::class.java)
        private val json = Json { ignoreUnknownKeys = true }

        fun getInstance(project: Project): HyperTranspilerService = project.service()
    }

    @Serializable
    data class SourceMapping(
        val gen_line: Int,
        val gen_col: Int,
        val src_line: Int,
        val src_col: Int
    )

    @Serializable
    data class PythonPiece(
        val prefix: String,
        val suffix: String,
        val src_start: Int,
        val src_end: Int
    )

    @Serializable
    data class TranspileResult(
        val python_code: String,
        val source_mappings: List<SourceMapping>,
        val python_pieces: List<PythonPiece>? = null
    )

    fun transpile(content: String, includeInjection: Boolean = false): TranspileResult {
        val hyperPath = findHyperBinary()
            ?: throw TranspileException("Could not find 'hyper' binary. Install it or add it to PATH.")

        LOG.info("Using hyper binary: $hyperPath")

        val args = mutableListOf(hyperPath, "generate", "--stdin", "--json")
        if (includeInjection) {
            args.add("--injection")
        }

        val processBuilder = ProcessBuilder(args)
            .redirectErrorStream(false)

        val process = processBuilder.start()

        OutputStreamWriter(process.outputStream, Charsets.UTF_8).use { writer ->
            writer.write(content)
        }

        val completed = process.waitFor(10, TimeUnit.SECONDS)
        if (!completed) {
            process.destroyForcibly()
            throw TranspileException("Transpiler timed out")
        }

        if (process.exitValue() != 0) {
            val error = process.errorStream.bufferedReader().readText()
            throw TranspileException("Transpiler failed: $error")
        }

        val output = process.inputStream.bufferedReader().readText()
        return json.decodeFromString<TranspileResult>(output)
    }

    private fun findHyperBinary(): String? {
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
                return path
            }
        }

        // Try PATH
        return try {
            val process = ProcessBuilder("which", "hyper").start()
            if (process.waitFor(2, TimeUnit.SECONDS) && process.exitValue() == 0) {
                process.inputStream.bufferedReader().readLine()?.trim()
            } else null
        } catch (e: Exception) {
            null
        }
    }

    class TranspileException(message: String) : Exception(message)
}
