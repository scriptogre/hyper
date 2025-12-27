package com.hyper.plugin

import com.hyper.plugin.psi.HyperRootElement
import com.intellij.lang.Language
import com.intellij.lang.injection.MultiHostInjector
import com.intellij.lang.injection.MultiHostRegistrar
import com.intellij.psi.PsiElement

class HyperLanguageInjector : MultiHostInjector {

    companion object {
        private val PYTHON_LANGUAGE: Language? by lazy {
            Language.findLanguageByID("Python")
        }
    }

    override fun getLanguagesToInject(registrar: MultiHostRegistrar, context: PsiElement) {
        if (context !is HyperRootElement) return
        val python = PYTHON_LANGUAGE ?: return

        registrar.startInjecting(python)

        HyperVirtualFileGenerator.generate(context) { prefix, suffix, element, range ->
            // If range is empty and prefix/suffix just add newline, addPlace handles it.
            // But addPlace requires a valid range inside the host.
            // My generator provides range relative to RootElement (context).
            registrar.addPlace(prefix, suffix, context, range)
        }

        registrar.doneInjecting()
    }

    override fun elementsToInjectIn(): MutableList<out Class<out PsiElement>> {
        return mutableListOf(HyperRootElement::class.java)
    }
}