package com.hyper.plugin

import com.hyper.plugin.psi.HyperTypes
import com.intellij.lang.annotation.AnnotationHolder
import com.intellij.lang.annotation.Annotator
import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.openapi.editor.DefaultLanguageHighlighterColors
import com.intellij.openapi.editor.colors.EditorColorsManager
import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.openapi.editor.colors.TextAttributesKey.createTextAttributesKey
import com.intellij.openapi.editor.markup.TextAttributes
import com.intellij.openapi.util.TextRange
import com.intellij.psi.PsiElement

/**
 * Annotator for sub-line syntax highlighting in .hyper files.
 *
 * The Hyper lexer is line-based (each line = one token), so the lexer-based
 * syntax highlighter can only color entire lines. This annotator adds
 * fine-grained highlighting for:
 *   - Control flow keywords (for, in, if, elif, else, etc.)
 *   - Inline comments (# after code on the same line)
 *
 * Expression brace highlighting is handled by HyperExternalAnnotator using
 * transpiler data. HTML tag highlighting is handled by HyperLanguageInjector.
 */
class HyperSyntaxAnnotator : Annotator {

    companion object {
        val EXPRESSION_BRACE = createTextAttributesKey(
            "HYPER_EXPRESSION_BRACE", DefaultLanguageHighlighterColors.KEYWORD
        )
        val INLINE_COMMENT = createTextAttributesKey(
            "HYPER_INLINE_COMMENT", DefaultLanguageHighlighterColors.LINE_COMMENT
        )
        // Fallback keys chosen to render in all IDEs including PyCharm CE
        // (which doesn't have HTML/MARKUP_TAG defined). Users can still
        // customize HYPER_* keys in Settings → Color Scheme.
        val TAG_PUNCTUATION = createTextAttributesKey(
            // Astro-style: `<`, `>`, `/>` and `{`, `}` use the keyword color (orange in Darcula).
            "HYPER_TAG_PUNCTUATION", DefaultLanguageHighlighterColors.KEYWORD
        )
        val COMPONENT_NAME = createTextAttributesKey(
            // Astro-style: component names use the function declaration color (yellow in Darcula).
            "HYPER_COMPONENT_NAME", DefaultLanguageHighlighterColors.FUNCTION_DECLARATION
        )
        val SLOT_KEYWORD = createTextAttributesKey(
            "HYPER_SLOT_KEYWORD", DefaultLanguageHighlighterColors.KEYWORD
        )
        val SLOT_NAME = createTextAttributesKey(
            "HYPER_SLOT_NAME", DefaultLanguageHighlighterColors.INSTANCE_FIELD
        )

        // Keywords that start a control line, matched at the beginning (after indent)
        private val CONTROL_KEYWORDS = listOf(
            "async for ", "async with ",
            "for ", "if ", "elif ", "else:", "while ", "match ", "with ",
            "try:", "except:", "except ", "finally:",
        )

        // Secondary keywords within control lines
        private val FOR_IN_REGEX = Regex("""(?<=\s)in\s""")
        private val CASE_REGEX = Regex("""^\s*case\s""")
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

        // Highlight control flow keywords on CONTROL_LINE tokens
        if (elementType == HyperTypes.CONTROL_LINE) {
            highlightControlKeywords(text, base, holder)
        }

        var i = 0
        var exprDepth = 0
        var inString = false
        var stringChar = ' '

        // Track "after structural" state to match the tokenizer's comment detection.
        // A # is only an inline comment when:
        //   1. after_structural is true (last structural element was a tag close, expression close, or line start)
        //   2. All text since that structural element is whitespace
        //   3. There is at least some whitespace (buffer is non-empty)
        var afterStructural = true
        var allWhitespaceSinceStructural = true
        var hasTextSinceStructural = false

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
                afterStructural = false
                i++
                continue
            }

            // --- Track expression depth (for comment detection) ---
            if (ch == '{' && i + 1 < len && text[i + 1] == '{') {
                i += 2
                continue
            }
            if (ch == '}' && i + 1 < len && text[i + 1] == '}') {
                i += 2
                continue
            }
            if (ch == '{') {
                exprDepth++
                i++
                continue
            }
            if (ch == '}' && exprDepth > 0) {
                exprDepth--
                if (exprDepth == 0) {
                    // Expression close is a structural boundary
                    afterStructural = true
                    allWhitespaceSinceStructural = true
                    hasTextSinceStructural = false
                }
                i++
                continue
            }

            // Skip content inside expressions
            if (exprDepth > 0) {
                i++
                continue
            }

            // --- HTML / component / slot tags ---
            if (ch == '<') {
                val afterLt = if (i + 1 < len && text[i + 1] == '/') i + 2 else i + 1

                if (afterLt < len && text[afterLt] == '{') {
                    // Component or slot tag: <{...}> or </{...}>
                    // Find the matching `}`
                    var braceEnd = afterLt + 1
                    var bd = 1
                    while (braceEnd < len && bd > 0) {
                        if (text[braceEnd] == '{') bd++
                        if (text[braceEnd] == '}') bd--
                        if (bd > 0) braceEnd++
                    }

                    // Find the `>` that closes the tag
                    var tagEnd = braceEnd + 1
                    var td = 0
                    var tInStr = false
                    var tStrCh = ' '
                    while (tagEnd < len) {
                        val tc = text[tagEnd]
                        if (tInStr) {
                            if (tc == '\\' && tagEnd + 1 < len) { tagEnd += 2; continue }
                            if (tc == tStrCh) tInStr = false
                            tagEnd++; continue
                        }
                        if (tc == '"' || tc == '\'') { tInStr = true; tStrCh = tc; tagEnd++; continue }
                        if (tc == '{') { td++; tagEnd++; continue }
                        if (tc == '}') { td--; tagEnd++; continue }
                        if (tc == '>' && td <= 0) { tagEnd++; break }
                        tagEnd++
                    }

                    // `<` or `</`
                    enforce(holder, base + i, base + afterLt, TAG_PUNCTUATION)
                    // `{`
                    enforce(holder, base + afterLt, base + afterLt + 1, TAG_PUNCTUATION)
                    // `}`
                    if (braceEnd < len) {
                        enforce(holder, base + braceEnd, base + braceEnd + 1, TAG_PUNCTUATION)
                    }
                    // `>` or `/>`
                    if (tagEnd > 0 && tagEnd <= len) {
                        val gtStart = if (tagEnd >= 2 && text[tagEnd - 2] == '/') tagEnd - 2 else tagEnd - 1
                        enforce(holder, base + gtStart, base + tagEnd, TAG_PUNCTUATION)
                    }

                    // Name inside braces
                    val nameStart = afterLt + 1
                    val nameEnd = braceEnd
                    if (nameStart < nameEnd) {
                        val nameText = text.substring(nameStart, nameEnd)
                        if (nameText.startsWith("...")) {
                            enforce(holder, base + nameStart, base + nameStart + 3, SLOT_KEYWORD)
                            if (nameStart + 3 < nameEnd) {
                                enforce(holder, base + nameStart + 3, base + nameEnd, SLOT_NAME)
                            }
                        } else {
                            enforce(holder, base + nameStart, base + nameEnd, COMPONENT_NAME)
                        }
                    }

                    i = tagEnd
                } else {
                    // Regular HTML tag — skip past it
                    var j = i + 1
                    var d = 0
                    var inStr2 = false
                    var strCh2 = ' '
                    while (j < len) {
                        val c = text[j]
                        if (inStr2) {
                            if (c == '\\' && j + 1 < len) { j += 2; continue }
                            if (c == strCh2) inStr2 = false
                            j++; continue
                        }
                        if (c == '"' || c == '\'') { inStr2 = true; strCh2 = c; j++; continue }
                        if (c == '{') { d++; j++; continue }
                        if (c == '}') { d--; j++; continue }
                        if (c == '>' && d <= 0) { j++; break }
                        j++
                    }
                    i = j
                }
                afterStructural = true
                allWhitespaceSinceStructural = true
                hasTextSinceStructural = false
                continue
            }

            // --- Inline comments: # ... ---
            // Only treat # as comment when after a structural element and all
            // intervening text is whitespace (matching the tokenizer's rule).
            if (ch == '#' && afterStructural && allWhitespaceSinceStructural && hasTextSinceStructural) {
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

            // Track text accumulation for comment detection
            if (ch.isWhitespace()) {
                hasTextSinceStructural = true
            } else {
                afterStructural = false
            }

            i++
        }
    }

    /**
     * Highlight Python keywords within a control line.
     * The lexer gives us the whole line as one token; this picks out
     * just the keyword portions (for, in, if, elif, else, while, etc.).
     */
    private fun highlightControlKeywords(text: String, base: Int, holder: AnnotationHolder) {
        val trimmed = text.trimStart()
        val indent = text.length - trimmed.length

        // Match the leading keyword
        for (kw in CONTROL_KEYWORDS) {
            if (trimmed.startsWith(kw)) {
                // Highlight just the keyword part (without trailing space/colon)
                val kwText = kw.trimEnd(' ', ':')
                highlight(holder, base + indent, base + indent + kwText.length, HyperSyntaxHighlighter.KEYWORD)

                // For "async for" / "async with", also highlight the second keyword
                if (kwText == "async for" || kwText == "async with") {
                    highlight(holder, base + indent, base + indent + 5, HyperSyntaxHighlighter.KEYWORD) // "async"
                    highlight(holder, base + indent + 6, base + indent + kwText.length, HyperSyntaxHighlighter.KEYWORD) // "for"/"with"
                }

                break
            }
        }

        // "for ... in ..." — highlight the "in" keyword
        if (trimmed.startsWith("for ") || trimmed.startsWith("async for ")) {
            val match = FOR_IN_REGEX.find(text)
            if (match != null) {
                val inStart = match.range.first
                highlight(holder, base + inStart, base + inStart + 2, HyperSyntaxHighlighter.KEYWORD)
            }
        }

        // "case ..." inside match blocks
        val caseMatch = CASE_REGEX.find(text)
        if (caseMatch != null) {
            highlight(holder, base + indent, base + indent + 4, HyperSyntaxHighlighter.KEYWORD) // "case"
        }
    }

    private fun highlight(holder: AnnotationHolder, start: Int, end: Int, key: TextAttributesKey) {
        if (start >= end) return
        holder.newSilentAnnotation(HighlightSeverity.INFORMATION)
            .range(TextRange(start, end))
            .textAttributes(key)
            .create()
    }

    /**
     * Apply a TextAttributesKey via `enforcedTextAttributes`. Required for
     * tag punctuation/names inside the injection host: the regular
     * `textAttributes(key)` path resolves keys against the injected language's
     * color scheme, which can render them invisible. Enforcing pre-resolved
     * attributes bypasses that.
     */
    private fun enforce(holder: AnnotationHolder, start: Int, end: Int, key: TextAttributesKey) {
        if (start >= end) return
        val attrs = resolveAttributes(key) ?: return
        holder.newSilentAnnotation(HighlightSeverity.INFORMATION)
            .range(TextRange(start, end))
            .enforcedTextAttributes(attrs)
            .create()
    }

    /**
     * Resolve TextAttributes by walking the fallback chain from a key until
     * we find attributes with an actual foreground/background color.
     */
    private fun resolveAttributes(key: TextAttributesKey): TextAttributes? {
        val scheme = EditorColorsManager.getInstance().globalScheme
        var current: TextAttributesKey? = key
        while (current != null) {
            val attrs = scheme.getAttributes(current)
            if (attrs != null && (attrs.foregroundColor != null || attrs.backgroundColor != null)) {
                return attrs
            }
            current = current.fallbackAttributeKey
        }
        return null
    }
}
