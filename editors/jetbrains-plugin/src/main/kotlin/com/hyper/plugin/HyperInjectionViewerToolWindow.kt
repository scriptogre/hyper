package com.hyper.plugin

import com.intellij.openapi.editor.colors.EditorColorsManager
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.fileEditor.FileEditorManagerEvent
import com.intellij.openapi.fileEditor.FileEditorManagerListener
import com.intellij.openapi.project.Project
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.ui.JBColor
import com.intellij.ui.components.JBScrollPane
import com.intellij.ui.content.ContentFactory
import java.awt.BorderLayout
import java.awt.Color
import javax.swing.*
import javax.swing.text.SimpleAttributeSet
import javax.swing.text.StyleConstants

class HyperInjectionViewerToolWindowFactory : ToolWindowFactory {
    override fun createToolWindowContent(project: Project, toolWindow: ToolWindow) {
        val panel = HyperInjectionViewerPanel(project)
        val content = ContentFactory.getInstance().createContent(panel, "", false)
        toolWindow.contentManager.addContent(content)
    }
}

class HyperInjectionViewerPanel(private val project: Project) : JPanel(BorderLayout()) {
    private val textPane = JTextPane()
    private val prefixColor = JBColor(Color(200, 230, 200), Color(50, 80, 50))  // Light/dark green
    private val sourceColor = JBColor(Color.WHITE, Color(43, 43, 43))  // Normal background
    private val htmlLabelColor = JBColor(Color(230, 200, 200), Color(80, 50, 50))  // Light/dark red

    init {
        textPane.isEditable = false
        textPane.font = EditorColorsManager.getInstance().globalScheme.getFont(null)
        add(JBScrollPane(textPane), BorderLayout.CENTER)

        // Listen for file changes
        project.messageBus.connect().subscribe(
            FileEditorManagerListener.FILE_EDITOR_MANAGER,
            object : FileEditorManagerListener {
                override fun selectionChanged(event: FileEditorManagerEvent) {
                    updateContent()
                }
            }
        )

        // Initial update
        updateContent()
    }

    private fun updateContent() {
        val editor = FileEditorManager.getInstance(project).selectedTextEditor ?: return
        val file = FileEditorManager.getInstance(project).selectedFiles.firstOrNull() ?: return

        if (!file.name.endsWith(".hyper")) {
            textPane.text = "(Open a .hyper file to see injection preview)"
            return
        }

        val text = editor.document.text
        if (text.isBlank()) {
            textPane.text = "(Empty file)"
            return
        }

        try {
            val service = HyperTranspilerService.getInstance(project)
            val result = service.transpile(text, includeInjection = true)
            displayInjections(text, result)
        } catch (e: Exception) {
            textPane.text = "Error: ${e.message}"
        }
    }

    private fun displayInjections(sourceText: String, result: HyperTranspilerService.TranspileResult) {
        val doc = textPane.styledDocument

        // Clear existing content
        doc.remove(0, doc.length)

        val prefixStyle = SimpleAttributeSet().apply {
            StyleConstants.setBackground(this, prefixColor)
        }
        val sourceStyle = SimpleAttributeSet().apply {
            StyleConstants.setBackground(this, sourceColor)
        }
        val labelStyle = SimpleAttributeSet().apply {
            StyleConstants.setBold(this, true)
            StyleConstants.setForeground(this, JBColor.GRAY)
        }
        val htmlLabelStyle = SimpleAttributeSet().apply {
            StyleConstants.setBold(this, true)
            StyleConstants.setBackground(this, htmlLabelColor)
        }

        // Python virtual file
        doc.insertString(doc.length, "═══ VIRTUAL PYTHON FILE ═══\n", labelStyle)
        doc.insertString(doc.length, "Background: ", labelStyle)
        doc.insertString(doc.length, " PREFIX/SUFFIX (added by plugin) ", prefixStyle)
        doc.insertString(doc.length, " ", sourceStyle)
        doc.insertString(doc.length, " SOURCE (from .hyper) ", sourceStyle)
        doc.insertString(doc.length, "\n\n", labelStyle)

        val pythonPieces = result.injections?.python ?: emptyList()
        for (piece in pythonPieces) {
            if (piece.start >= 0 && piece.end <= sourceText.length && piece.start <= piece.end) {
                val source = sourceText.substring(piece.start, piece.end)
                doc.insertString(doc.length, piece.prefix, prefixStyle)
                doc.insertString(doc.length, source, sourceStyle)
                doc.insertString(doc.length, piece.suffix, prefixStyle)
            }
        }

        // HTML virtual file
        doc.insertString(doc.length, "\n\n═══ VIRTUAL HTML FILE ═══\n", htmlLabelStyle)

        val htmlPieces = result.injections?.html ?: emptyList()
        for (piece in htmlPieces) {
            if (piece.start >= 0 && piece.end <= sourceText.length && piece.start < piece.end) {
                val source = sourceText.substring(piece.start, piece.end)
                doc.insertString(doc.length, source, sourceStyle)
            }
        }
    }
}
