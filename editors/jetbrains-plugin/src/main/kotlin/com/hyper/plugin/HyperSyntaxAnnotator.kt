package com.hyper.plugin

import com.hyper.plugin.psi.HyperTypes
import com.intellij.lang.annotation.AnnotationHolder
import com.intellij.lang.annotation.Annotator
import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.openapi.editor.DefaultLanguageHighlighterColors
import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.openapi.editor.colors.TextAttributesKey.createTextAttributesKey
import com.intellij.openapi.util.TextRange
import com.intellij.psi.PsiElement

/**
 * Annotator for sub-line syntax highlighting in .hyper files.
 *
 * The Hyper lexer is line-based (each line = one token), so the lexer-based
 * syntax highlighter can only color entire lines. This annotator adds
 * fine-grained highlighting for:
 *   - Expression braces { }
 *   - Inline comments (# after code on the same line)
 *
 * HTML tag highlighting is intentionally NOT done here — it's handled by
 * the HTML language injection via HyperLanguageInjector.
 */
class HyperSyntaxAnnotator : Annotator {

    companion object {
        val EXPRESSION_BRACE = createTextAttributesKey(
            "HYPER_EXPRESSION_BRACE", DefaultLanguageHighlighterColors.KEYWORD
        )
        val INLINE_COMMENT = createTextAttributesKey(
            "HYPER_INLINE_COMMENT", DefaultLanguageHighlighterColors.LINE_COMMENT
        )
    }

    override fun annotate(element: PsiElement, holder: AnnotationHolder) {
        val node = element.node ?: return
        val elementType = node.elementType

        // Process composite elements (from the parser) that contain sub-elements
        if (elementType != HyperTypes.HTML_LINE &&
            elementType != HyperTypes.CONTROL_LINE &&
            elementType != HyperTypes.PYTHON_LINE) {
            return
        }

        val text = element.text
        val base = element.textRange.startOffset
        val len = text.length

        var i = 0
        var exprDepth = 0
        var inString = false
        var stringChar = ' '

        while (i < len) {
            val ch = text[i]

            // --- String tracking (skip contents) ---
            if (inString) {
                if (ch == '\\' && i + 1 < len) {
                    i += 2
                    continue
                }
                if (ch == stringChar) {
                    inString = false
                }
                i++
                continue
            }
            if (ch == '"' || ch == '\'') {
                inString = true
                stringChar = ch
                i++
                continue
            }

            // --- Escaped braces {{ and }} ---
            if (ch == '{' && i + 1 < len && text[i + 1] == '{') {
                i += 2
                continue
            }
            if (ch == '}' && i + 1 < len && text[i + 1] == '}') {
                i += 2
                continue
            }

            // --- Expression braces ---
            if (ch == '{' && exprDepth == 0) {
                highlight(holder, base + i, base + i + 1, EXPRESSION_BRACE)
                exprDepth++
                i++
                continue
            }
            if (ch == '{' && exprDepth > 0) {
                exprDepth++
                i++
                continue
            }
            if (ch == '}' && exprDepth > 0) {
                exprDepth--
                if (exprDepth == 0) {
                    highlight(holder, base + i, base + i + 1, EXPRESSION_BRACE)
                }
                i++
                continue
            }

            // Skip content inside expressions
            if (exprDepth > 0) {
                i++
                continue
            }

            // --- Skip HTML tags (don't interfere with HTML language injection) ---
            if (ch == '<') {
                // Advance past the tag entirely
                var j = i + 1
                var d = 0
                var inStr = false
                var strCh = ' '
                while (j < len) {
                    val c = text[j]
                    if (inStr) {
                        if (c == '\\' && j + 1 < len) { j += 2; continue }
                        if (c == strCh) inStr = false
                        j++
                        continue
                    }
                    if (c == '"' || c == '\'') { inStr = true; strCh = c; j++; continue }
                    if (c == '{') {
                        if (d == 0) highlight(holder, base + j, base + j + 1, EXPRESSION_BRACE)
                        d++; j++; continue
                    }
                    if (c == '}') {
                        d--
                        if (d == 0) highlight(holder, base + j, base + j + 1, EXPRESSION_BRACE)
                        j++; continue
                    }
                    if (c == '>' && d <= 0) { j++; break }
                    j++
                }
                i = j
                continue
            }

            // --- Inline comments: # ... ---
            if (ch == '#') {
                var lineEnd = len
                for (k in i until len) {
                    if (text[k] == '\n' || text[k] == '\r') {
                        lineEnd = k
                        break
                    }
                }
                highlight(holder, base + i, base + lineEnd, INLINE_COMMENT)
                i = lineEnd
                continue
            }

            i++
        }
    }

    private fun highlight(holder: AnnotationHolder, start: Int, end: Int, key: TextAttributesKey) {
        if (start >= end) return
        holder.newSilentAnnotation(HighlightSeverity.INFORMATION)
            .range(TextRange(start, end))
            .textAttributes(key)
            .create()
    }
}
