package com.hyper.plugin

import com.hyper.plugin.psi.HyperRootElement
import com.intellij.lang.Language
import com.intellij.lang.injection.MultiHostInjector
import com.intellij.lang.injection.MultiHostRegistrar
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.progress.ProcessCanceledException
import com.intellij.openapi.util.TextRange
import com.intellij.psi.PsiElement

/**
 * Injects Python and HTML into .hyper files.
 * Python injection enables go-to-definition, autocomplete etc.
 * HTML injection enables HTML syntax highlighting.
 */
class HyperLanguageInjector : MultiHostInjector {

    companion object {
        private val LOG = Logger.getInstance(HyperLanguageInjector::class.java)
        private val PYTHON_LANGUAGE: Language? by lazy {
            Language.findLanguageByID("Python")
        }
        private val HTML_LANGUAGE: Language? by lazy {
            Language.findLanguageByID("HTML")
        }
    }

    override fun getLanguagesToInject(registrar: MultiHostRegistrar, context: PsiElement) {
        if (context !is HyperRootElement) return
        val project = context.project

        try {
            val service = HyperTranspilerService.getInstance(project)
            val text = context.text
            val textLength = text.length
            val result = service.transpile(text, includeInjection = true)

            // Inject Python
            val python = PYTHON_LANGUAGE
            val pythonPieces = result.injections?.python
            if (python != null && !pythonPieces.isNullOrEmpty()) {
                val validPythonPieces = pythonPieces.filter { piece ->
                    piece.start >= 0 && piece.end <= textLength && piece.start <= piece.end
                }
                if (validPythonPieces.isNotEmpty()) {
                    registrar.startInjecting(python)
                    for (piece in validPythonPieces) {
                        val range = TextRange(piece.start, piece.end)
                        registrar.addPlace(piece.prefix, piece.suffix, context, range)
                    }
                    registrar.doneInjecting()
                    LOG.debug("Python injection: ${validPythonPieces.size} pieces")
                }
            }

            // Inject HTML
            val html = HTML_LANGUAGE
            val htmlPieces = result.injections?.html
            if (html != null && !htmlPieces.isNullOrEmpty()) {
                val validHtmlPieces = htmlPieces.filter { piece ->
                    piece.start >= 0 && piece.end <= textLength && piece.start < piece.end
                }
                if (validHtmlPieces.isNotEmpty()) {
                    registrar.startInjecting(html)
                    for (piece in validHtmlPieces) {
                        val range = TextRange(piece.start, piece.end)
                        registrar.addPlace(null, null, context, range)
                    }
                    registrar.doneInjecting()
                    LOG.debug("HTML injection: ${validHtmlPieces.size} pieces")
                }
            } else if (html == null) {
                LOG.info("HTML language not available in this IDE")
            }

        } catch (e: ProcessCanceledException) {
            throw e
        } catch (e: HyperTranspilerService.TranspileException) {
            LOG.debug("Transpile error: ${e.message}")
        } catch (e: Exception) {
            LOG.warn("Unexpected error during injection", e)
        }
    }

    override fun elementsToInjectIn(): MutableList<out Class<out PsiElement>> {
        return mutableListOf(HyperRootElement::class.java)
    }
}
