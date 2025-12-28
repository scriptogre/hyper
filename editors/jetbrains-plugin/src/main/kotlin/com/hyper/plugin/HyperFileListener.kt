package com.hyper.plugin

import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.vfs.AsyncFileListener
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.openapi.vfs.newvfs.events.VFileContentChangeEvent
import com.intellij.openapi.vfs.newvfs.events.VFileEvent
import java.io.File
import java.io.OutputStreamWriter
import java.util.concurrent.TimeUnit

/**
 * Listens for .hyper file saves and generates corresponding .py files.
 */
class HyperFileListener : AsyncFileListener {

    companion object {
        private val LOG = Logger.getInstance(HyperFileListener::class.java)
    }

    override fun prepareChange(events: MutableList<out VFileEvent>): AsyncFileListener.ChangeApplier? {
        val hyperFiles = events
            .filterIsInstance<VFileContentChangeEvent>()
            .map { it.file }
            .filter { it.extension == "hyper" }

        if (hyperFiles.isEmpty()) return null

        return object : AsyncFileListener.ChangeApplier {
            override fun afterVfsChange() {
                for (file in hyperFiles) {
                    generatePythonFile(file)
                }
            }
        }
    }

    private fun generatePythonFile(hyperFile: VirtualFile) {
        try {
            val content = String(hyperFile.contentsToByteArray(), Charsets.UTF_8)
            val outputName = hyperFile.nameWithoutExtension + ".py"
            val parent = hyperFile.parent ?: return

            val pythonCode = transpile(content) ?: return

            val outputFile = parent.findOrCreateChildData(this, outputName)
            outputFile.getOutputStream(this).use { stream ->
                stream.write(pythonCode.toByteArray(Charsets.UTF_8))
            }

            LOG.debug("Generated $outputName")
        } catch (e: Exception) {
            LOG.warn("Failed to generate Python for ${hyperFile.name}", e)
        }
    }

    private fun transpile(content: String): String? {
        val hyperPath = findHyperBinary() ?: return null

        return try {
            val process = ProcessBuilder(hyperPath, "generate", "--stdin")
                .redirectErrorStream(false)
                .start()

            OutputStreamWriter(process.outputStream, Charsets.UTF_8).use { writer ->
                writer.write(content)
            }

            val completed = process.waitFor(10, TimeUnit.SECONDS)
            if (!completed) {
                process.destroyForcibly()
                return null
            }

            if (process.exitValue() == 0) {
                process.inputStream.bufferedReader().readText()
            } else null
        } catch (e: Exception) {
            LOG.debug("Transpile failed", e)
            null
        }
    }

    private fun findHyperBinary(): String? {
        val homeDir = System.getProperty("user.home")
        val candidates = listOf(
            "$homeDir/.cargo/bin/hyper",
            "$homeDir/.local/bin/hyper",
            "/usr/local/bin/hyper",
            "/opt/homebrew/bin/hyper",
        )

        for (path in candidates) {
            if (File(path).canExecute()) return path
        }

        return try {
            val process = ProcessBuilder("which", "hyper").start()
            if (process.waitFor(2, TimeUnit.SECONDS) && process.exitValue() == 0) {
                process.inputStream.bufferedReader().readLine()?.trim()
            } else null
        } catch (e: Exception) {
            null
        }
    }
}
