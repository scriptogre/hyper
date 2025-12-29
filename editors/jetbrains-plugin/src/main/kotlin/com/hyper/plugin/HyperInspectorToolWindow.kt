package com.hyper.plugin

import com.intellij.openapi.Disposable
import com.intellij.openapi.editor.EditorFactory
import com.intellij.openapi.editor.ScrollType
import com.intellij.openapi.editor.colors.EditorColorsManager
import com.intellij.openapi.editor.event.CaretEvent
import com.intellij.openapi.editor.event.CaretListener
import com.intellij.openapi.editor.ex.EditorEx
import com.intellij.openapi.editor.highlighter.EditorHighlighterFactory
import com.intellij.openapi.editor.markup.HighlighterLayer
import com.intellij.openapi.editor.markup.HighlighterTargetArea
import com.intellij.openapi.editor.markup.TextAttributes
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.fileEditor.FileEditorManagerEvent
import com.intellij.openapi.fileEditor.FileEditorManagerListener
import com.intellij.openapi.fileTypes.FileTypeManager
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.Disposer
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.icons.AllIcons
import com.intellij.openapi.ui.Messages
import com.intellij.ui.content.ContentFactory
import java.awt.BorderLayout
import java.awt.Color
import java.awt.Cursor
import java.awt.FlowLayout
import java.awt.event.MouseAdapter
import java.awt.event.MouseEvent
import javax.swing.JLabel
import javax.swing.JPanel

class HyperInspectorToolWindowFactory : ToolWindowFactory {
    override fun createToolWindowContent(project: Project, toolWindow: ToolWindow) {
        val contentFactory = ContentFactory.getInstance()

        // Tab 1: Virtual File (with injection highlighting)
        val virtualFilePanel = HyperVirtualFilePanel(project, toolWindow)
        val virtualFileContent = contentFactory.createContent(virtualFilePanel, "IDE View", false)
        toolWindow.contentManager.addContent(virtualFileContent)

        // Tab 2: Compiled Output
        val compiledPanel = HyperCompiledOutputPanel(project, toolWindow)
        val compiledContent = contentFactory.createContent(compiledPanel, "Compiled", false)
        toolWindow.contentManager.addContent(compiledContent)
    }
}

class HyperVirtualFilePanel(
    private val project: Project,
    toolWindow: ToolWindow
) : JPanel(BorderLayout()), Disposable {

    private val editorFactory = EditorFactory.getInstance()
    private val document = editorFactory.createDocument("")
    private val editor: EditorEx

    private val backgroundColor = Color(0x19, 0x1a, 0x1c)
    private val generatedHighlightColor = Color(0x2d, 0x3a, 0x2d)  // Subtle green tint for generated code

    // Mapping from source offset ranges to virtual file offset ranges
    private var sourceToVirtualMappings = listOf<SourceMapping>()
    private var currentCaretListener: CaretListener? = null
    private var currentSourceEditor: com.intellij.openapi.editor.Editor? = null

    private data class SourceMapping(
        val sourceStart: Int,
        val sourceEnd: Int,
        val virtualStart: Int,
        val virtualEnd: Int
    )

    init {
        editor = editorFactory.createViewer(document, project) as EditorEx
        editor.settings.apply {
            isLineNumbersShown = true
            isWhitespacesShown = false
            isFoldingOutlineShown = false
            additionalLinesCount = 0
            additionalColumnsCount = 0
            isCaretRowShown = false
        }

        // Set Python syntax highlighting
        val pythonFileType = FileTypeManager.getInstance().getFileTypeByExtension("py")
        val highlighter = EditorHighlighterFactory.getInstance().createEditorHighlighter(
            project,
            pythonFileType
        )
        editor.highlighter = highlighter

        // Set dark background
        editor.backgroundColor = backgroundColor
        val scheme = EditorColorsManager.getInstance().globalScheme.clone() as com.intellij.openapi.editor.colors.EditorColorsScheme
        scheme.setColor(com.intellij.openapi.editor.colors.EditorColors.GUTTER_BACKGROUND, backgroundColor)
        editor.colorsScheme = scheme

        // Add help icon in header
        val headerPanel = JPanel(FlowLayout(FlowLayout.RIGHT, 4, 2))
        headerPanel.isOpaque = false
        val helpIcon = JLabel(AllIcons.General.ContextHelp)
        helpIcon.cursor = Cursor.getPredefinedCursor(Cursor.HAND_CURSOR)
        helpIcon.addMouseListener(object : MouseAdapter() {
            override fun mouseClicked(e: MouseEvent) {
                Messages.showInfoMessage(
                    project,
                    "The Python that the IDE sees for autocomplete and go-to-definition.",
                    "IDE View"
                )
            }
        })
        headerPanel.add(helpIcon)
        add(headerPanel, BorderLayout.NORTH)

        add(editor.component, BorderLayout.CENTER)

        // Listen for file changes
        project.messageBus.connect(this).subscribe(
            FileEditorManagerListener.FILE_EDITOR_MANAGER,
            object : FileEditorManagerListener {
                override fun selectionChanged(event: FileEditorManagerEvent) {
                    updateContent()
                }
            }
        )

        // Register for disposal
        Disposer.register(toolWindow.disposable, this)

        // Initial update
        updateContent()
    }

    private fun updateContent() {
        val textEditor = FileEditorManager.getInstance(project).selectedTextEditor ?: return
        val file = FileEditorManager.getInstance(project).selectedFiles.firstOrNull() ?: return

        if (!file.name.endsWith(".hyper")) {
            updateEditorText("# Open a .hyper file to see the Python virtual file", emptyList())
            return
        }

        val text = textEditor.document.text
        if (text.isBlank()) {
            updateEditorText("# Empty file", emptyList())
            return
        }

        try {
            val service = HyperTranspilerService.getInstance(project)
            val result = service.transpile(text, includeInjection = true)
            val assemblyResult = assemblePythonVirtualFile(text, result)
            updateEditorText(assemblyResult.content, assemblyResult.generatedRanges)

            // Update source mappings and set up caret listener
            sourceToVirtualMappings = assemblyResult.sourceMappings
            setupCaretListener(textEditor)
        } catch (e: Exception) {
            updateEditorText("# Error: ${e.message}", emptyList())
            sourceToVirtualMappings = emptyList()
        }
    }

    private fun setupCaretListener(sourceEditor: com.intellij.openapi.editor.Editor) {
        // Remove old listener if exists
        currentCaretListener?.let { listener ->
            currentSourceEditor?.caretModel?.removeCaretListener(listener)
        }

        val caretListener = object : CaretListener {
            override fun caretPositionChanged(event: CaretEvent) {
                val sourceOffset = event.caret?.offset ?: return
                scrollToSourcePosition(sourceOffset)
            }
        }

        sourceEditor.caretModel.addCaretListener(caretListener)
        currentCaretListener = caretListener
        currentSourceEditor = sourceEditor

        // Initial scroll to current position
        scrollToSourcePosition(sourceEditor.caretModel.offset)
    }

    private fun scrollToSourcePosition(sourceOffset: Int) {
        // Find the mapping that contains this source offset
        val mapping = sourceToVirtualMappings.find { mapping ->
            sourceOffset >= mapping.sourceStart && sourceOffset < mapping.sourceEnd
        } ?: return

        // Calculate relative position within the source range
        val relativePos = (sourceOffset - mapping.sourceStart).toDouble() /
            (mapping.sourceEnd - mapping.sourceStart).coerceAtLeast(1)

        // Map to virtual file position
        val virtualOffset = mapping.virtualStart +
            ((mapping.virtualEnd - mapping.virtualStart) * relativePos).toInt()

        // Scroll the viewer to that position
        if (virtualOffset in 0 until document.textLength) {
            editor.caretModel.moveToOffset(virtualOffset)
            editor.scrollingModel.scrollToCaret(ScrollType.CENTER)
        }
    }

    private data class TextRange(val start: Int, val end: Int)

    private data class AssemblyResult(
        val content: String,
        val generatedRanges: List<TextRange>,
        val sourceMappings: List<SourceMapping>
    )

    private fun assemblePythonVirtualFile(
        sourceText: String,
        result: HyperTranspilerService.TranspileResult
    ): AssemblyResult {
        val pythonPieces = result.injections?.python
            ?: return AssemblyResult("# No Python segments", emptyList(), emptyList())

        val content = StringBuilder()
        val generatedRanges = mutableListOf<TextRange>()
        val sourceMappings = mutableListOf<SourceMapping>()

        for (piece in pythonPieces) {
            if (piece.start >= 0 && piece.end <= sourceText.length && piece.start <= piece.end) {
                val source = sourceText.substring(piece.start, piece.end)

                // Track prefix range
                if (piece.prefix.isNotEmpty()) {
                    val prefixStart = content.length
                    content.append(piece.prefix)
                    generatedRanges.add(TextRange(prefixStart, content.length))
                }

                // Track source mapping (source position → virtual position)
                val virtualStart = content.length
                content.append(source)
                val virtualEnd = content.length
                sourceMappings.add(SourceMapping(piece.start, piece.end, virtualStart, virtualEnd))

                // Track suffix range
                if (piece.suffix.isNotEmpty()) {
                    val suffixStart = content.length
                    content.append(piece.suffix)
                    generatedRanges.add(TextRange(suffixStart, content.length))
                }
            }
        }
        return AssemblyResult(content.toString(), generatedRanges, sourceMappings)
    }

    private fun updateEditorText(text: String, generatedRanges: List<TextRange>) {
        com.intellij.openapi.application.ApplicationManager.getApplication().runWriteAction {
            document.setText(text)
        }

        // Apply highlights for generated code
        val markupModel = editor.markupModel
        markupModel.removeAllHighlighters()

        val attributes = TextAttributes().apply {
            backgroundColor = generatedHighlightColor
        }

        for (range in generatedRanges) {
            if (range.start < range.end && range.end <= document.textLength) {
                markupModel.addRangeHighlighter(
                    range.start,
                    range.end,
                    HighlighterLayer.ADDITIONAL_SYNTAX,
                    attributes,
                    HighlighterTargetArea.EXACT_RANGE
                )
            }
        }
    }

    override fun dispose() {
        // Clean up caret listener
        currentCaretListener?.let { listener ->
            currentSourceEditor?.caretModel?.removeCaretListener(listener)
        }
        currentCaretListener = null
        currentSourceEditor = null

        editorFactory.releaseEditor(editor)
    }
}

class HyperCompiledOutputPanel(
    private val project: Project,
    toolWindow: ToolWindow
) : JPanel(BorderLayout()), Disposable {

    private val editorFactory = EditorFactory.getInstance()
    private val document = editorFactory.createDocument("")
    private val editor: EditorEx

    private val backgroundColor = Color(0x19, 0x1a, 0x1c)

    init {
        editor = editorFactory.createViewer(document, project) as EditorEx
        editor.settings.apply {
            isLineNumbersShown = true
            isWhitespacesShown = false
            isFoldingOutlineShown = false
            additionalLinesCount = 0
            additionalColumnsCount = 0
            isCaretRowShown = false
        }

        // Set Python syntax highlighting
        val pythonFileType = FileTypeManager.getInstance().getFileTypeByExtension("py")
        val highlighter = EditorHighlighterFactory.getInstance().createEditorHighlighter(
            project,
            pythonFileType
        )
        editor.highlighter = highlighter

        // Set dark background
        editor.backgroundColor = backgroundColor
        val scheme = EditorColorsManager.getInstance().globalScheme.clone() as com.intellij.openapi.editor.colors.EditorColorsScheme
        scheme.setColor(com.intellij.openapi.editor.colors.EditorColors.GUTTER_BACKGROUND, backgroundColor)
        editor.colorsScheme = scheme

        // Add help icon in header
        val headerPanel = JPanel(FlowLayout(FlowLayout.RIGHT, 4, 2))
        headerPanel.isOpaque = false
        val helpIcon = JLabel(AllIcons.General.ContextHelp)
        helpIcon.cursor = Cursor.getPredefinedCursor(Cursor.HAND_CURSOR)
        helpIcon.addMouseListener(object : MouseAdapter() {
            override fun mouseClicked(e: MouseEvent) {
                Messages.showInfoMessage(
                    project,
                    "The .py file generated on save that runs at runtime.",
                    "Compiled Output"
                )
            }
        })
        headerPanel.add(helpIcon)
        add(headerPanel, BorderLayout.NORTH)

        add(editor.component, BorderLayout.CENTER)

        // Listen for file changes
        project.messageBus.connect(this).subscribe(
            FileEditorManagerListener.FILE_EDITOR_MANAGER,
            object : FileEditorManagerListener {
                override fun selectionChanged(event: FileEditorManagerEvent) {
                    updateContent()
                }
            }
        )

        // Register for disposal
        Disposer.register(toolWindow.disposable, this)

        // Initial update
        updateContent()
    }

    private fun updateContent() {
        val textEditor = FileEditorManager.getInstance(project).selectedTextEditor ?: return
        val file = FileEditorManager.getInstance(project).selectedFiles.firstOrNull() ?: return

        if (!file.name.endsWith(".hyper")) {
            updateEditorText("# Open a .hyper file to see the compiled output")
            return
        }

        val text = textEditor.document.text
        if (text.isBlank()) {
            updateEditorText("# Empty file")
            return
        }

        try {
            val service = HyperTranspilerService.getInstance(project)
            val result = service.transpile(text, includeInjection = false)
            updateEditorText(result.code)
        } catch (e: Exception) {
            updateEditorText("# Error: ${e.message}")
        }
    }

    private fun updateEditorText(text: String) {
        com.intellij.openapi.application.ApplicationManager.getApplication().runWriteAction {
            document.setText(text)
        }
    }

    override fun dispose() {
        editorFactory.releaseEditor(editor)
    }
}
