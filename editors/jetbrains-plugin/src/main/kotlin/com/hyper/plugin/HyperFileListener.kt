package com.hyper.plugin

import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.project.ProjectManager
import com.intellij.openapi.vfs.AsyncFileListener
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.openapi.vfs.newvfs.events.VFileContentChangeEvent
import com.intellij.openapi.vfs.newvfs.events.VFileEvent

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
        val project = ProjectManager.getInstance().openProjects.firstOrNull() ?: return

        try {
            val content = String(hyperFile.contentsToByteArray(), Charsets.UTF_8)
            val service = HyperTranspilerService.getInstance(project)
            val result = service.transpile(content, includeInjection = false)

            val outputName = hyperFile.nameWithoutExtension + ".py"
            val parent = hyperFile.parent ?: return
            val outputFile = parent.findOrCreateChildData(this, outputName)

            outputFile.getOutputStream(this).use { stream ->
                stream.write(result.code.toByteArray(Charsets.UTF_8))
            }

            LOG.debug("Generated $outputName")
        } catch (e: HyperTranspilerService.TranspileException) {
            LOG.warn("Failed to transpile ${hyperFile.name}: ${e.message}")
        } catch (e: Exception) {
            LOG.warn("Failed to generate Python for ${hyperFile.name}", e)
        }
    }
}
