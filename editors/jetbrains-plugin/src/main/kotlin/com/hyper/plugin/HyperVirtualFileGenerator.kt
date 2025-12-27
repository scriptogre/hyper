package com.hyper.plugin

import com.hyper.plugin.psi.HyperRootElement
import com.hyper.plugin.psi.HyperTypes
import com.intellij.openapi.util.TextRange
import com.intellij.psi.PsiElement

object HyperVirtualFileGenerator {

    // Pattern to match type annotations: "name: Type" or "name: Type = value"
    private val TYPE_ANNOTATION_PATTERN = Regex("""^([a-zA-Z_][a-zA-Z0-9_]*)\s*:\s*(.+)$""")

    // Patterns to detect await as a keyword (not in comments/strings)
    private val AWAIT_PATTERNS = listOf(
        Regex("""^await\s"""),
        Regex("""[=(\[,:]\s*await\s"""),
    )

    fun generate(
        context: HyperRootElement,
        callback: (prefix: String, suffix: String, element: PsiElement, range: TextRange) -> Unit
    ) {
        // First pass: find structure (leading content and parameters, body start)
        val leadingContent = mutableListOf<LeadingContent>()
        val parameters = mutableListOf<ParameterWithRange>()
        val bodyStartIndex = findStructure(context, leadingContent, parameters)
        val isAsync = hasAwaitKeyword(context)

        // Output leading content (comments and empty lines) before the function
        for (content in leadingContent) {
            when (content) {
                is LeadingContent.Comment -> callback("", "\n", context, content.range)
                is LeadingContent.EmptyLine -> callback("", "\n", context, TextRange.from(content.offset, 0))
            }
        }

        // Generate function signature with parameters linked to source
        // Important: combine def keyword with first parameter's prefix to avoid
        // emitting an empty range at position 0 (which would conflict with comments)
        val defKeyword = if (isAsync) "async def" else "def"
        if (parameters.isNotEmpty()) {
            parameters.forEachIndexed { index, param ->
                val prefix = if (index == 0) "$defKeyword __hyper_template__(" else ", "
                val suffix = if (index == parameters.lastIndex) "):\n" else ""
                callback(prefix, suffix, context, param.range)
            }
        } else {
            // No parameters - use an empty range at the body start position
            val bodyStartOffset = if (bodyStartIndex > 0) {
                // Find the actual offset of the first body element
                var offset = 0
                var idx = 0
                var child = context.firstChild
                while (child != null && idx < bodyStartIndex) {
                    offset = child.textRangeInParent.endOffset
                    idx++
                    child = child.nextSibling
                }
                offset
            } else {
                0
            }
            callback("$defKeyword __hyper_template__():\n", "", context, TextRange.from(bodyStartOffset, 0))
        }

        // Now generate the body with +1 base indent (inside the function)
        var indentLevel = 1
        val indent = "    "
        val blockStack = mutableListOf<String>()

        var index = 0
        var child = context.firstChild
        while (child != null) {
            // Skip leading comments and parameters (already handled above)
            if (index < bodyStartIndex) {
                index++
                child = child.nextSibling
                continue
            }

            val type = child.node.elementType
            val text = child.text
            val rangeInParent = child.textRangeInParent

            // Find content bounds (strip leading whitespace and trailing newline)
            var start = 0
            while (start < text.length && (text[start] == ' ' || text[start] == '\t')) start++
            var end = text.length
            while (end > start && (text[end - 1] == '\n' || text[end - 1] == '\r')) end--

            if (start >= end) {
                callback("", "\n", context, TextRange.from(rangeInParent.startOffset, 0))
                index++
                child = child.nextSibling
                continue
            }

            val contentRange = TextRange.from(rangeInParent.startOffset + start, end - start)
            val trimmed = text.substring(start, end)

            when (type) {
                HyperTypes.CONTROL_LINE -> {
                    // Dedent keywords: else, elif, except, finally, case
                    val isDedent = trimmed.startsWith("else") ||
                                   trimmed.startsWith("elif") ||
                                   trimmed.startsWith("except") ||
                                   trimmed.startsWith("finally")
                    val isCase = trimmed.startsWith("case")

                    when {
                        isDedent -> {
                            val printIndent = (indentLevel - 1).coerceAtLeast(1)
                            callback(indent.repeat(printIndent), "\n", context, contentRange)
                        }
                        isCase -> {
                            if (blockStack.lastOrNull() == "case") {
                                blockStack.removeLast()
                                indentLevel--
                            }
                            callback(indent.repeat(indentLevel), "\n", context, contentRange)
                            blockStack.add("case")
                            indentLevel++
                        }
                        else -> {
                            callback(indent.repeat(indentLevel), "\n", context, contentRange)
                            val blockType = if (trimmed.startsWith("match")) "match" else "block"
                            blockStack.add(blockType)
                            indentLevel++
                        }
                    }
                }
                HyperTypes.END_LINE -> {
                    while (blockStack.lastOrNull() == "case") {
                        blockStack.removeLast()
                        indentLevel--
                    }
                    if (blockStack.isNotEmpty()) {
                        blockStack.removeLast()
                        indentLevel--
                    }
                    indentLevel = indentLevel.coerceAtLeast(1)
                    callback(indent.repeat(indentLevel) + "pass\n", "", context, TextRange.from(rangeInParent.startOffset, 0))
                }
                HyperTypes.HTML_LINE -> {
                    callback(indent.repeat(indentLevel) + "t\"\"\"", "\"\"\"\n", context, contentRange)
                }
                HyperTypes.PYTHON_LINE -> {
                    callback(indent.repeat(indentLevel), "\n", context, contentRange)
                }
            }

            index++
            child = child.nextSibling
        }
    }

    private sealed class LeadingContent {
        data class Comment(val range: TextRange) : LeadingContent()
        // For empty lines, we track the position but use zero-length range
        // since the newline itself is added as suffix
        data class EmptyLine(val offset: Int) : LeadingContent()
    }
    private data class ParameterWithRange(val range: TextRange)

    private fun findStructure(
        context: HyperRootElement,
        leadingContent: MutableList<LeadingContent>,
        parameters: MutableList<ParameterWithRange>
    ): Int {
        var index = 0
        var child = context.firstChild
        var seenParameter = false

        while (child != null) {
            val type = child.node.elementType
            val text = child.text
            val trimmed = text.trim()
            val rangeInParent = child.textRangeInParent

            // Calculate content range (strip whitespace)
            var start = 0
            while (start < text.length && (text[start] == ' ' || text[start] == '\t')) start++
            var end = text.length
            while (end > start && (text[end - 1] == '\n' || text[end - 1] == '\r')) end--
            val contentRange = TextRange.from(rangeInParent.startOffset + start, end - start)

            // Empty lines before first parameter - preserve them
            if (trimmed.isEmpty() && !seenParameter) {
                leadingContent.add(LeadingContent.EmptyLine(rangeInParent.startOffset))
                index++
                child = child.nextSibling
                continue
            }

            // Comments before parameters - collect them
            if (!seenParameter && trimmed.startsWith("#")) {
                leadingContent.add(LeadingContent.Comment(contentRange))
                index++
                child = child.nextSibling
                continue
            }

            // Check if this is a type annotation (parameter declaration)
            if (type == HyperTypes.PYTHON_LINE) {
                val match = TYPE_ANNOTATION_PATTERN.matchEntire(trimmed)
                if (match != null) {
                    seenParameter = true
                    parameters.add(ParameterWithRange(contentRange))
                    index++
                    child = child.nextSibling
                    continue
                }
            }

            // Not a parameter declaration - body starts here
            break
        }

        return index
    }

    private fun hasAwaitKeyword(context: HyperRootElement): Boolean {
        var child = context.firstChild
        while (child != null) {
            val type = child.node.elementType
            if (type == HyperTypes.PYTHON_LINE) {
                val text = child.text.trim()
                if (!text.startsWith("#")) {
                    for (pattern in AWAIT_PATTERNS) {
                        if (pattern.containsMatchIn(text)) {
                            return true
                        }
                    }
                }
            }
            child = child.nextSibling
        }
        return false
    }
}