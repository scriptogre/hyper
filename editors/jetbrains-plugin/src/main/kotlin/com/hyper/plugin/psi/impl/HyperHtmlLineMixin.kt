package com.hyper.plugin.psi.impl

import com.intellij.extapi.psi.ASTWrapperPsiElement
import com.intellij.lang.ASTNode
import com.intellij.psi.impl.source.tree.injected.InjectionBackgroundSuppressor

// InjectionBackgroundSuppressor prevents the green background on injected Python code
abstract class HyperHtmlLineMixin(node: ASTNode) : ASTWrapperPsiElement(node), InjectionBackgroundSuppressor