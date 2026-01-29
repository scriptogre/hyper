package com.hyper.plugin

import com.hyper.plugin.lexer._HyperLexer
import com.hyper.plugin.psi.HyperTypes
import com.intellij.lexer.FlexAdapter
import com.intellij.lexer.Lexer
import com.intellij.openapi.editor.DefaultLanguageHighlighterColors
import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.openapi.editor.colors.TextAttributesKey.createTextAttributesKey
import com.intellij.openapi.fileTypes.SyntaxHighlighter
import com.intellij.openapi.fileTypes.SyntaxHighlighterBase
import com.intellij.openapi.fileTypes.SyntaxHighlighterFactory
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.psi.tree.IElementType

class HyperSyntaxHighlighter : SyntaxHighlighterBase() {

    companion object {
        val KEYWORD = createTextAttributesKey("HYPER_KEYWORD", DefaultLanguageHighlighterColors.KEYWORD)
        val COMMENT = createTextAttributesKey("HYPER_COMMENT", DefaultLanguageHighlighterColors.LINE_COMMENT)
        val SEPARATOR = createTextAttributesKey("HYPER_SEPARATOR", DefaultLanguageHighlighterColors.BRACES)
        val BAD_CHARACTER = createTextAttributesKey("HYPER_BAD_CHARACTER", com.intellij.openapi.editor.HighlighterColors.BAD_CHARACTER)

        private val KEYWORD_KEYS = arrayOf(KEYWORD)
        private val COMMENT_KEYS = arrayOf(COMMENT)
        private val SEPARATOR_KEYS = arrayOf(SEPARATOR)
        private val BAD_CHAR_KEYS = arrayOf(BAD_CHARACTER)
        private val EMPTY_KEYS = emptyArray<TextAttributesKey>()
    }

    override fun getHighlightingLexer(): Lexer = FlexAdapter(_HyperLexer())

    override fun getTokenHighlights(tokenType: IElementType?): Array<TextAttributesKey> {
        return when (tokenType) {
            HyperTypes.CONTROL_LINE_TOKEN, HyperTypes.END_LINE_TOKEN -> KEYWORD_KEYS
            HyperTypes.COMMENT_TOKEN -> COMMENT_KEYS
            HyperTypes.SEPARATOR_TOKEN -> SEPARATOR_KEYS
            com.intellij.psi.TokenType.BAD_CHARACTER -> BAD_CHAR_KEYS
            else -> EMPTY_KEYS
        }
    }
}

class HyperSyntaxHighlighterFactory : SyntaxHighlighterFactory() {
    override fun getSyntaxHighlighter(project: Project?, virtualFile: VirtualFile?): SyntaxHighlighter {
        return HyperSyntaxHighlighter()
    }
}
