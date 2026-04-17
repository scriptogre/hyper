package com.hyper.plugin

import com.intellij.codeInsight.daemon.impl.HighlightInfo
import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.testFramework.fixtures.BasePlatformTestCase
import java.io.File

/**
 * Tests that HyperSyntaxAnnotator produces the correct TextAttributesKey
 * highlights for keywords, comments, component tags, and slot tags.
 *
 * Important: Hyper's MultiHostInjector injects Python over the entire file,
 * which replaces the document/PSI text with the generated Python output.
 * As a result, `myFixture.file.text` returns the injected Python, not
 * the original .hyper source. We read the source text directly from the
 * test data file and use it to extract text at the annotator's offsets
 * (which are always in the original .hyper coordinate space).
 *
 * The IntelliJ platform logs "Cannot restore" errors when the light test
 * fixture teardown encounters stale injected PSI elements. This test class
 * suppresses those specific errors since they are platform infrastructure
 * noise, not plugin bugs.
 */
class HyperAnnotatorTest : BasePlatformTestCase() {

    override fun getTestDataPath(): String = "src/test/testData"

    /**
     * Read the original .hyper source text from the test data file.
     *
     * We cannot use myFixture.file.text because the injection pipeline
     * replaces it with the generated Python content.
     */
    private fun sourceText(relativePath: String): String {
        return File(testDataPath, relativePath).readText()
    }

    /**
     * Find all highlights with a specific TextAttributesKey and return
     * the text they cover, using the original source text for extraction.
     *
     * We match only on `forcedTextAttributesKey` because that is what
     * `holder.newSilentAnnotation(...).textAttributes(key).create()` sets.
     * Matching on `type.attributesKey` is too broad and picks up unrelated
     * IntelliJ-internal highlights.
     */
    private fun highlightedTexts(
        highlights: List<HighlightInfo>,
        key: TextAttributesKey,
        source: String
    ): List<String> {
        return highlights
            .filter { it.forcedTextAttributesKey == key }
            .filter { it.startOffset >= 0 && it.endOffset <= source.length }
            .map { source.substring(it.startOffset, it.endOffset) }
    }

    fun testControlKeywords() {
        myFixture.configureByFile("annotation/keywords.hyper")
        val source = sourceText("annotation/keywords.hyper")
        val highlights = myFixture.doHighlighting()
        val keywords = highlightedTexts(highlights, HyperSyntaxHighlighter.KEYWORD, source)

        // Every control keyword in the fixture should be highlighted.
        // Note: "end" is highlighted by the lexer-level highlighter (KEYWORD_KEYS
        // for END_LINE_TOKEN), not the annotator. Lexer highlights may not appear
        // in forcedTextAttributesKey. We only check annotator-produced keywords here.
        val expected = listOf(
            "for", "in",
            "if", "elif", "else",
            "while",
            "match", "case", "case",
            "try", "except", "finally",
            "with",
        )
        for (kw in expected) {
            assertTrue(
                "Expected keyword '$kw' to be highlighted, got: $keywords",
                keywords.contains(kw)
            )
        }

        // No non-keyword text should be highlighted as a keyword
        val validKeywords = setOf("for", "in", "if", "elif", "else", "while",
            "match", "case", "try", "except", "finally", "with", "async")
        for (text in keywords) {
            assertTrue(
                "Unexpected keyword highlight: '$text'",
                text in validKeywords
            )
        }
    }

    fun testInlineComments() {
        myFixture.configureByFile("annotation/comments.hyper")
        val source = sourceText("annotation/comments.hyper")
        val highlights = myFixture.doHighlighting()
        val comments = highlightedTexts(highlights, HyperSyntaxAnnotator.INLINE_COMMENT, source)

        // Inline comments (after structural elements)
        assertTrue(
            "Expected '# After tag close' to be highlighted as inline comment, got: $comments",
            comments.any { it.contains("After tag close") }
        )
        assertTrue(
            "Expected '# After paragraph' to be highlighted as inline comment, got: $comments",
            comments.any { it.contains("After paragraph") }
        )

        // Top-level comment is a COMMENT_TOKEN (lexer-level), not INLINE_COMMENT.
        // Make sure inline comment detection doesn't claim it.
        assertFalse(
            "Top-level comment should not be INLINE_COMMENT",
            comments.any { it.contains("Top-level") }
        )
    }

    /**
     * Tests component tags, slot tags, and tag punctuation using
     * the components.hyper fixture file.
     */
    fun testComponentAndSlotTags() {
        myFixture.configureByFile("annotation/components.hyper")
        val source = sourceText("annotation/components.hyper")
        val highlights = myFixture.doHighlighting()

        // Component names
        val componentNames = highlightedTexts(highlights, HyperSyntaxAnnotator.COMPONENT_NAME, source)
        assertTrue(
            "Expected 'Header' as component name, got: $componentNames",
            componentNames.contains("Header")
        )
        assertTrue(
            "Expected 'Card' as component name, got: $componentNames",
            componentNames.contains("Card")
        )

        // Tag punctuation (< { } > for component tags)
        val punctuation = highlightedTexts(highlights, HyperSyntaxAnnotator.TAG_PUNCTUATION, source)
        assertTrue(
            "Expected '{' in tag punctuation, got: $punctuation",
            punctuation.contains("{")
        )
        assertTrue(
            "Expected '}' in tag punctuation, got: $punctuation",
            punctuation.contains("}")
        )

        // Slot keyword "..."
        val slotKeywords = highlightedTexts(highlights, HyperSyntaxAnnotator.SLOT_KEYWORD, source)
        assertTrue(
            "Expected '...' as slot keyword, got: $slotKeywords",
            slotKeywords.contains("...")
        )

        // Slot name
        val slotNames = highlightedTexts(highlights, HyperSyntaxAnnotator.SLOT_NAME, source)
        assertTrue(
            "Expected 'sidebar' as slot name, got: $slotNames",
            slotNames.contains("sidebar")
        )
    }
}
