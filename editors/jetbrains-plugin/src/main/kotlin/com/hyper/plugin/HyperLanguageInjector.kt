package com.hyper.plugin

import com.hyper.plugin.psi.HyperRootElement
import com.intellij.lang.Language
import com.intellij.lang.injection.MultiHostInjector
import com.intellij.lang.injection.MultiHostRegistrar
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.progress.ProcessCanceledException
import com.intellij.openapi.util.TextRange
import com.intellij.psi.PsiElement

class HyperLanguageInjector : MultiHostInjector {

    companion object {
        private val LOG = Logger.getInstance(HyperLanguageInjector::class.java)
        private val PYTHON_LANGUAGE: Language? by lazy {
            Language.findLanguageByID("Python")
        }
    }

    override fun getLanguagesToInject(registrar: MultiHostRegistrar, context: PsiElement) {
        if (context !is HyperRootElement) return
        val python = PYTHON_LANGUAGE ?: run {
            LOG.warn("Python language not found")
            return
        }
        val project = context.project

        try {
            val service = HyperTranspilerService.getInstance(project)
            LOG.info("Transpiling ${context.text.length} chars...")
            val start = System.currentTimeMillis()
            val result = service.transpile(context.text, includeInjection = true)
            LOG.info("Transpile took ${System.currentTimeMillis() - start}ms, got ${result.python_pieces?.size ?: 0} python pieces")

            val pythonPieces = result.python_pieces
            if (pythonPieces.isNullOrEmpty()) {
                LOG.warn("No python pieces returned")
                return
            }

            registrar.startInjecting(python)
            for (piece in pythonPieces) {
                val range = TextRange(piece.src_start, piece.src_end)
                registrar.addPlace(piece.prefix, piece.suffix, context, range)
            }
            registrar.doneInjecting()
            LOG.info("Python injection complete with ${pythonPieces.size} pieces")

        } catch (e: ProcessCanceledException) {
            LOG.info("Injection cancelled")
            throw e
        } catch (e: HyperTranspilerService.TranspileException) {
            LOG.warn("Failed to transpile .hyper file: ${e.message}")
        } catch (e: Exception) {
            LOG.warn("Unexpected error during .hyper transpilation", e)
        }
    }

    override fun elementsToInjectIn(): MutableList<out Class<out PsiElement>> {
        return mutableListOf(HyperRootElement::class.java)
    }
}