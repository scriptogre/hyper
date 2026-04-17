package com.hyper.plugin

import com.intellij.openapi.Disposable
import com.intellij.openapi.editor.EditorFactory
import com.intellij.openapi.editor.colors.EditorColorsManager
import com.intellij.openapi.editor.event.DocumentEvent
import com.intellij.openapi.editor.event.DocumentListener
import com.intellij.openapi.editor.ex.EditorEx
import com.intellij.openapi.editor.highlighter.EditorHighlighterFactory
import com.intellij.openapi.editor.markup.HighlighterLayer
import com.intellij.openapi.editor.markup.HighlighterTargetArea
import com.intellij.openapi.editor.markup.TextAttributes
import com.intellij.openapi.editor.markup.EffectType
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
import com.intellij.util.Alarm
import com.intellij.util.AlarmFactory
import java.awt.Font
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

        // Tab 1: Compiled Python output with range highlighting
        val compiledPanel = HyperCompiledOutputPanel(project, toolWindow)
        val compiledContent = contentFactory.createContent(compiledPanel, "Python", false)
        toolWindow.contentManager.addContent(compiledContent)

        // Tab 2: Injection ranges debugging
        val rangesPanel = HyperRangesPanel(project, toolWindow)
        val rangesContent = contentFactory.createContent(rangesPanel, "Ranges", false)
        toolWindow.contentManager.addContent(rangesContent)

        // Tab 3: Actual injections with prefix/suffix
        val injectionsPanel = HyperInjectionsPanel(project, toolWindow)
        val injectionsContent = contentFactory.createContent(injectionsPanel, "Injections", false)
        toolWindow.contentManager.addContent(injectionsContent)
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
    private val pythonBorderColor = Color(0x4a, 0xd6, 0x4a)  // Bright green for Python ranges
    private val htmlBorderColor = Color(0xff, 0x99, 0x33)     // Bright orange for HTML ranges
    private val boilerplateColor = Color(0x10, 0x10, 0x10)   // Almost black (de-emphasized)

    // Use Disposer pattern for listener lifecycle management
    private var listenerDisposable: Disposable? = null

    // Debounce for updates on keystroke (300ms delay)
    private val updateAlarm = AlarmFactory.getInstance().create(Alarm.ThreadToUse.POOLED_THREAD, this)

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
                    """
                    The .py file generated on save that runs at runtime.

                    Visual indicators:
                    • Green underline: Python code from source
                    • Orange underline: HTML content from source
                    • Dark background: Generated boilerplate
                    """.trimIndent(),
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
            updateEditorText("# Open a .hyper file to see the compiled output", emptyList())
            return
        }

        val text = textEditor.document.text
        if (text.isBlank()) {
            updateEditorText("# Empty file", emptyList())
            return
        }

        try {
            val service = HyperTranspilerService.getInstance(project)
            // Use filename (without extension) as function name to match actual compilation
            val functionName = file.nameWithoutExtension
            val result = service.transpile(text, includeInjection = true, functionName = functionName)
            updateEditorText(result.code, result.ranges)

            // Set up document listener for live updates
            setupDocumentListener(textEditor)
        } catch (e: Exception) {
            updateEditorText("# Error: ${e.message}", emptyList())
        }
    }

    private fun setupDocumentListener(sourceEditor: com.intellij.openapi.editor.Editor) {
        // Dispose previous listener (this automatically removes it from the document)
        listenerDisposable?.let { Disposer.dispose(it) }
        listenerDisposable = null

        // Create a new disposable for this listener's lifecycle
        val newDisposable = Disposer.newDisposable(this, "HyperInspector document listener")

        val documentListener = object : DocumentListener {
            override fun documentChanged(event: DocumentEvent) {
                // Debounce: cancel previous pending update and schedule a new one
                updateAlarm.cancelAllRequests()
                updateAlarm.addRequest(::updateContent, 300)
            }
        }

        // Use the overload that takes a Disposable - listener is auto-removed on dispose
        sourceEditor.document.addDocumentListener(documentListener, newDisposable)
        listenerDisposable = newDisposable
    }

    private fun updateEditorText(text: String, ranges: List<HyperTranspilerService.Range> = emptyList()) {
        // Write actions must be on EDT, so wrap in invokeLater
        com.intellij.openapi.application.ApplicationManager.getApplication().invokeLater {
            com.intellij.openapi.application.ApplicationManager.getApplication().runWriteAction {
                document.setText(text)
            }

            // Highlight ranges by type: Python (green), HTML (blue), boilerplate (gray)
            val markupModel = editor.markupModel
            markupModel.removeAllHighlighters()

            val textLength = document.textLength
            if (textLength == 0) return@invokeLater

            val pythonAttributes = TextAttributes().apply {
                effectType = EffectType.LINE_UNDERSCORE
                effectColor = pythonBorderColor
            }
            val htmlAttributes = TextAttributes().apply {
                effectType = EffectType.LINE_UNDERSCORE
                effectColor = htmlBorderColor
            }
            val boilerplateAttributes = TextAttributes().apply {
                backgroundColor = boilerplateColor
            }

            if (ranges.isEmpty()) {
                // No ranges - everything is boilerplate
                markupModel.addRangeHighlighter(
                    0,
                    textLength,
                    HighlighterLayer.SELECTION - 1,
                    boilerplateAttributes,
                    HighlighterTargetArea.EXACT_RANGE
                )
                return@invokeLater
            }

            val sortedRanges = ranges.sortedBy { it.compiled_start }

            // Highlight ranges and gaps (boilerplate)
            var currentPos = 0
            for (range in sortedRanges) {
                val rangeStart = range.compiled_start.coerceIn(0, textLength)
                val rangeEnd = range.compiled_end.coerceIn(0, textLength)

                // Highlight gap before this range as boilerplate
                if (currentPos < rangeStart) {
                    markupModel.addRangeHighlighter(
                        currentPos,
                        rangeStart,
                        HighlighterLayer.SELECTION - 1,
                        boilerplateAttributes,
                        HighlighterTargetArea.EXACT_RANGE
                    )
                }

                // Highlight this range based on type
                if (rangeStart < rangeEnd) {
                    val attributes = when (range.type) {
                        "python" -> pythonAttributes
                        "html" -> htmlAttributes
                        else -> boilerplateAttributes
                    }
                    markupModel.addRangeHighlighter(
                        rangeStart,
                        rangeEnd,
                        HighlighterLayer.SELECTION - 1,
                        attributes,
                        HighlighterTargetArea.EXACT_RANGE
                    )
                }

                currentPos = maxOf(currentPos, rangeEnd)
            }

            // Highlight remaining text as boilerplate
            if (currentPos < textLength) {
                markupModel.addRangeHighlighter(
                    currentPos,
                    textLength,
                    HighlighterLayer.SELECTION - 1,
                    boilerplateAttributes,
                    HighlighterTargetArea.EXACT_RANGE
                )
            }
        }
    }

    override fun dispose() {
        // Clean up listeners via Disposer (auto-removes from document)
        listenerDisposable?.let { Disposer.dispose(it) }
        listenerDisposable = null

        updateAlarm.cancelAllRequests()
        editorFactory.releaseEditor(editor)
    }
}

class HyperRangesPanel(
    private val project: Project,
    toolWindow: ToolWindow
) : JPanel(BorderLayout()), Disposable {

    private val editorFactory = EditorFactory.getInstance()
    private val document = editorFactory.createDocument("")
    private val editor: EditorEx

    private val backgroundColor = Color(0x19, 0x1a, 0x1c)

    // Use Disposer pattern for listener lifecycle management
    private var listenerDisposable: Disposable? = null

    // Debounce for updates on keystroke (300ms delay)
    private val updateAlarm = AlarmFactory.getInstance().create(Alarm.ThreadToUse.POOLED_THREAD, this)

    init {
        editor = editorFactory.createViewer(document, project) as EditorEx
        editor.settings.apply {
            isLineNumbersShown = false
            isWhitespacesShown = false
            isFoldingOutlineShown = false
            additionalLinesCount = 0
            additionalColumnsCount = 0
            isCaretRowShown = false
        }

        // No syntax highlighting - just plain text
        editor.backgroundColor = backgroundColor
        val scheme = EditorColorsManager.getInstance().globalScheme.clone() as com.intellij.openapi.editor.colors.EditorColorsScheme
        scheme.setColor(com.intellij.openapi.editor.colors.EditorColors.GUTTER_BACKGROUND, backgroundColor)
        editor.colorsScheme = scheme

        // Use monospace font
        editor.colorsScheme.editorFontName = "JetBrains Mono"
        editor.colorsScheme.editorFontSize = 12

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
            updateRangesText("# Open a .hyper file to see injection ranges")
            return
        }

        val sourceText = textEditor.document.text
        if (sourceText.isBlank()) {
            updateRangesText("# Empty file")
            return
        }

        try {
            val service = HyperTranspilerService.getInstance(project)
            val functionName = file.nameWithoutExtension
            val result = service.transpile(sourceText, includeInjection = true, functionName = functionName)

            val formatted = formatRanges(sourceText, result.code, result.ranges)
            updateRangesText(formatted)

            // Set up document listener for live updates
            setupDocumentListener(textEditor)
        } catch (e: Exception) {
            updateRangesText("# Error: ${e.message}")
        }
    }

    private fun formatRanges(sourceText: String, compiledText: String, ranges: List<HyperTranspilerService.Range>): String {
        val sb = StringBuilder()

        sb.appendLine("INJECTION RANGES")
        sb.appendLine("━".repeat(80))
        sb.appendLine()

        if (ranges.isEmpty()) {
            sb.appendLine("No injection ranges found.")
            return sb.toString()
        }

        for ((index, range) in ranges.withIndex()) {
            // Extract source text
            val sourceSnippet = if (range.source_start < sourceText.length && range.source_end <= sourceText.length) {
                sourceText.substring(range.source_start, range.source_end)
            } else {
                "!!! OUT OF BOUNDS !!!"
            }

            // Extract compiled text
            val compiledSnippet = if (range.compiled_start < compiledText.length && range.compiled_end <= compiledText.length) {
                compiledText.substring(range.compiled_start, range.compiled_end)
            } else {
                "!!! OUT OF BOUNDS !!!"
            }

            // Determine if there are issues
            val issues = mutableListOf<String>()

            // Check for out of bounds
            if (range.source_end > sourceText.length) {
                issues.add("source_end exceeds source length")
            }
            if (range.compiled_end > compiledText.length) {
                issues.add("compiled_end exceeds compiled length")
            }

            // Check if source text looks split mid-word
            if (sourceSnippet.isNotEmpty() && !sourceSnippet.first().isWhitespace() &&
                range.source_start > 0 && !sourceText[range.source_start - 1].isWhitespace()) {
                issues.add("source text appears split mid-word")
            }

            // Check if source contains mixed content
            val hasHtmlTags = sourceSnippet.contains('<') || sourceSnippet.contains('>')
            val hasPythonSyntax = sourceSnippet.contains('(') || sourceSnippet.contains('[') || sourceSnippet.contains('{')
            if (range.type == "python" && hasHtmlTags) {
                issues.add("Python range contains HTML tags")
            }
            if (range.type == "html" && hasPythonSyntax && !sourceSnippet.contains('<')) {
                issues.add("HTML range contains Python syntax without tags")
            }

            // Format range header
            val typeLabel = "[${range.type.uppercase()}]"
            sb.appendLine("$typeLabel source[${range.source_start}:${range.source_end}] → compiled[${range.compiled_start}:${range.compiled_end}]")

            // Show snippets
            sb.appendLine("  source:   ${formatSnippet(sourceSnippet)}")
            sb.appendLine("  compiled: ${formatSnippet(compiledSnippet)}")

            // Show validation
            if (issues.isEmpty()) {
                if (sourceSnippet == compiledSnippet) {
                    sb.appendLine("  ✓ Match")
                } else {
                    sb.appendLine("  ℹ Transform (expected)")
                }
            } else {
                sb.appendLine("  ⚠ ISSUES:")
                for (issue in issues) {
                    sb.appendLine("    • $issue")
                }
            }

            if (index < ranges.size - 1) {
                sb.appendLine()
            }
        }

        return sb.toString()
    }

    private fun formatSnippet(text: String): String {
        // Escape and truncate if needed
        val escaped = text.replace("\n", "\\n").replace("\r", "\\r").replace("\t", "\\t")
        return "\"${escaped.take(60)}${if (escaped.length > 60) "..." else ""}\""
    }

    private fun setupDocumentListener(sourceEditor: com.intellij.openapi.editor.Editor) {
        listenerDisposable?.let { Disposer.dispose(it) }
        listenerDisposable = null

        val newDisposable = Disposer.newDisposable(this, "HyperRanges document listener")

        val documentListener = object : DocumentListener {
            override fun documentChanged(event: DocumentEvent) {
                updateAlarm.cancelAllRequests()
                updateAlarm.addRequest(::updateContent, 300)
            }
        }

        sourceEditor.document.addDocumentListener(documentListener, newDisposable)
        listenerDisposable = newDisposable
    }

    private fun updateRangesText(text: String) {
        com.intellij.openapi.application.ApplicationManager.getApplication().invokeLater {
            com.intellij.openapi.application.ApplicationManager.getApplication().runWriteAction {
                document.setText(text)
            }
        }
    }

    override fun dispose() {
        listenerDisposable?.let { Disposer.dispose(it) }
        listenerDisposable = null
        updateAlarm.cancelAllRequests()
        editorFactory.releaseEditor(editor)
    }
}

class HyperInjectionsPanel(
    private val project: Project,
    toolWindow: ToolWindow
) : JPanel(BorderLayout()), Disposable {

    private val editorFactory = EditorFactory.getInstance()
    private val document = editorFactory.createDocument("")
    private val editor: EditorEx

    private val backgroundColor = Color(0x19, 0x1a, 0x1c)

    private var listenerDisposable: Disposable? = null
    private val updateAlarm = AlarmFactory.getInstance().create(Alarm.ThreadToUse.POOLED_THREAD, this)

    init {
        editor = editorFactory.createViewer(document, project) as EditorEx
        editor.settings.apply {
            isLineNumbersShown = false
            isWhitespacesShown = false
            isFoldingOutlineShown = false
            additionalLinesCount = 0
            additionalColumnsCount = 0
            isCaretRowShown = false
        }

        editor.backgroundColor = backgroundColor
        val scheme = EditorColorsManager.getInstance().globalScheme.clone() as com.intellij.openapi.editor.colors.EditorColorsScheme
        scheme.setColor(com.intellij.openapi.editor.colors.EditorColors.GUTTER_BACKGROUND, backgroundColor)
        editor.colorsScheme = scheme
        editor.colorsScheme.editorFontName = "JetBrains Mono"
        editor.colorsScheme.editorFontSize = 12

        add(editor.component, BorderLayout.CENTER)

        project.messageBus.connect(this).subscribe(
            FileEditorManagerListener.FILE_EDITOR_MANAGER,
            object : FileEditorManagerListener {
                override fun selectionChanged(event: FileEditorManagerEvent) {
                    updateContent()
                }
            }
        )

        Disposer.register(toolWindow.disposable, this)
        updateContent()
    }

    private fun updateContent() {
        val textEditor = FileEditorManager.getInstance(project).selectedTextEditor ?: return
        val file = FileEditorManager.getInstance(project).selectedFiles.firstOrNull() ?: return

        if (!file.name.endsWith(".hyper")) {
            updateText("# Open a .hyper file to see injections")
            return
        }

        val sourceText = textEditor.document.text
        if (sourceText.isBlank()) {
            updateText("# Empty file")
            return
        }

        try {
            val service = HyperTranspilerService.getInstance(project)
            val functionName = file.nameWithoutExtension
            val result = service.transpile(sourceText, includeInjection = true, functionName = functionName)

            val formatted = formatInjections(sourceText, result)
            updateText(formatted)

            setupDocumentListener(textEditor)
        } catch (e: Exception) {
            updateText("# Error: ${e.message}\n\n${e.stackTraceToString()}")
        }
    }

    private fun formatInjections(sourceText: String, result: HyperTranspilerService.TranspileResult): String {
        val sb = StringBuilder()

        sb.appendLine("INJECTIONS (what the IDE receives)")
        sb.appendLine("━".repeat(80))
        sb.appendLine()

        val pythonInjections = result.pythonInjections
        val htmlInjections = result.htmlInjections

        sb.appendLine("Python injections: ${pythonInjections.size}")
        sb.appendLine("HTML injections: ${htmlInjections.size}")
        sb.appendLine()

        if (pythonInjections.isEmpty() && htmlInjections.isEmpty()) {
            sb.appendLine("No injections found!")
            sb.appendLine()
            sb.appendLine("This means the IDE won't apply any language highlighting.")
            sb.appendLine("Check if the transpiler is producing injections with needs_injection=true")
            return sb.toString()
        }

        // Show Python injections
        if (pythonInjections.isNotEmpty()) {
            sb.appendLine("─── PYTHON INJECTIONS ───")
            sb.appendLine()

            for ((index, inj) in pythonInjections.withIndex()) {
                val sourceSnippet = safeSubstring(sourceText, inj.start, inj.end)

                sb.appendLine("[${index + 1}] source[${inj.start}:${inj.end}] = ${formatSnippet(sourceSnippet, 40)}")
                sb.appendLine()
                sb.appendLine("    prefix (${inj.prefix.length} chars):")
                sb.appendLine("    ┌─────────────────────────────────────")
                for (line in inj.prefix.lines().take(8)) {
                    sb.appendLine("    │ ${line.take(60)}")
                }
                if (inj.prefix.lines().size > 8) {
                    sb.appendLine("    │ ... (${inj.prefix.lines().size - 8} more lines)")
                }
                sb.appendLine("    └─────────────────────────────────────")
                sb.appendLine()
                sb.appendLine("    suffix (${inj.suffix.length} chars):")
                sb.appendLine("    ┌─────────────────────────────────────")
                for (line in inj.suffix.lines().take(8)) {
                    sb.appendLine("    │ ${line.take(60)}")
                }
                if (inj.suffix.lines().size > 8) {
                    sb.appendLine("    │ ... (${inj.suffix.lines().size - 8} more lines)")
                }
                sb.appendLine("    └─────────────────────────────────────")
                sb.appendLine()

                // Show the concatenated result (what the IDE sees)
                sb.appendLine("    Virtual Python file fragment:")
                sb.appendLine("    ┌─────────────────────────────────────")
                val virtualContent = inj.prefix + sourceSnippet + inj.suffix
                for (line in virtualContent.lines().take(10)) {
                    sb.appendLine("    │ ${line.take(60)}")
                }
                if (virtualContent.lines().size > 10) {
                    sb.appendLine("    │ ... (${virtualContent.lines().size - 10} more lines)")
                }
                sb.appendLine("    └─────────────────────────────────────")
                sb.appendLine()
            }
        }

        // Show HTML injections
        if (htmlInjections.isNotEmpty()) {
            sb.appendLine("─── HTML INJECTIONS ───")
            sb.appendLine()

            for ((index, inj) in htmlInjections.withIndex()) {
                val sourceSnippet = safeSubstring(sourceText, inj.start, inj.end)
                sb.appendLine("[${index + 1}] source[${inj.start}:${inj.end}] = ${formatSnippet(sourceSnippet, 40)}")
                sb.appendLine("    prefix: ${formatSnippet(inj.prefix, 60)}")
                sb.appendLine("    suffix: ${formatSnippet(inj.suffix, 60)}")
                sb.appendLine()
            }
        }

        return sb.toString()
    }

    private fun safeSubstring(text: String, start: Int, end: Int): String {
        return if (start >= 0 && end <= text.length && start <= end) {
            text.substring(start, end)
        } else {
            "!!! OUT OF BOUNDS (${start}:${end} in ${text.length}) !!!"
        }
    }

    private fun formatSnippet(text: String, maxLen: Int): String {
        val escaped = text.replace("\n", "\\n").replace("\r", "\\r").replace("\t", "\\t")
        return "\"${escaped.take(maxLen)}${if (escaped.length > maxLen) "..." else ""}\""
    }

    private fun setupDocumentListener(sourceEditor: com.intellij.openapi.editor.Editor) {
        listenerDisposable?.let { Disposer.dispose(it) }
        listenerDisposable = null

        val newDisposable = Disposer.newDisposable(this, "HyperInjections document listener")

        val documentListener = object : DocumentListener {
            override fun documentChanged(event: DocumentEvent) {
                updateAlarm.cancelAllRequests()
                updateAlarm.addRequest(::updateContent, 300)
            }
        }

        sourceEditor.document.addDocumentListener(documentListener, newDisposable)
        listenerDisposable = newDisposable
    }

    private fun updateText(text: String) {
        com.intellij.openapi.application.ApplicationManager.getApplication().invokeLater {
            com.intellij.openapi.application.ApplicationManager.getApplication().runWriteAction {
                document.setText(text)
            }
        }
    }

    override fun dispose() {
        listenerDisposable?.let { Disposer.dispose(it) }
        listenerDisposable = null
        updateAlarm.cancelAllRequests()
        editorFactory.releaseEditor(editor)
    }
}
