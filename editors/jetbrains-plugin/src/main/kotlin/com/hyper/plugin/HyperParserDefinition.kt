package com.hyper.plugin

import com.hyper.plugin.lexer._HyperLexer
import com.hyper.plugin.parser.HyperParser
import com.hyper.plugin.psi.HyperTypes
import com.intellij.lang.ASTNode
import com.intellij.lang.ParserDefinition
import com.intellij.lang.PsiParser
import com.intellij.lexer.FlexAdapter
import com.intellij.lexer.Lexer
import com.intellij.openapi.project.Project
import com.intellij.psi.FileViewProvider
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile
import com.intellij.psi.tree.IFileElementType
import com.intellij.psi.tree.TokenSet

class HyperParserDefinition : ParserDefinition {

    companion object {
        val FILE = IFileElementType(HyperLanguage)
    }

    override fun createLexer(project: Project?): Lexer = FlexAdapter(_HyperLexer())

    override fun createParser(project: Project?): PsiParser = HyperParser()

    override fun getFileNodeType(): IFileElementType = FILE

    override fun getCommentTokens(): TokenSet = TokenSet.EMPTY

    override fun getStringLiteralElements(): TokenSet = TokenSet.EMPTY

    override fun getWhitespaceTokens(): TokenSet = TokenSet.WHITE_SPACE

    override fun createElement(node: ASTNode): PsiElement = HyperTypes.Factory.createElement(node)

    override fun createFile(viewProvider: FileViewProvider): PsiFile = HyperFile(viewProvider)
}
