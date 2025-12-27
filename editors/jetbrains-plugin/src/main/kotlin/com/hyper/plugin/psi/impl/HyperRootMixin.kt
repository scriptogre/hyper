package com.hyper.plugin.psi.impl

import com.intellij.extapi.psi.ASTWrapperPsiElement
import com.intellij.lang.ASTNode
import com.intellij.psi.LiteralTextEscaper
import com.intellij.psi.PsiLanguageInjectionHost
import com.intellij.psi.impl.source.tree.injected.InjectionBackgroundSuppressor

abstract class HyperRootMixin(node: ASTNode) : ASTWrapperPsiElement(node), PsiLanguageInjectionHost, InjectionBackgroundSuppressor {

    override fun isValidHost(): Boolean {
        return true
    }

    override fun updateText(text: String): PsiLanguageInjectionHost {
        // This is a simplified implementation. 
        // In a real scenario, replacing text in a composite element is complex.
        // For now, we assume the document update handles the AST change.
        return this
    }

    override fun createLiteralTextEscaper(): LiteralTextEscaper<out PsiLanguageInjectionHost> {
        return LiteralTextEscaper.createSimple(this)
    }
}
