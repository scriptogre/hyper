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
            // Use filename as function name for consistent naming
            val fileName = context.containingFile?.virtualFile?.nameWithoutExtension
            val result = service.transpile(text, includeInjection = true, functionName = fileName)

            // Inject Python
            val python = PYTHON_LANGUAGE
            val pythonInjections = result.pythonInjections
            if (python != null && pythonInjections.isNotEmpty()) {
                val validPythonInjections = pythonInjections.filter { inj ->
                    inj.start >= 0 && inj.end <= textLength && inj.start <= inj.end
                }
                if (validPythonInjections.isNotEmpty()) {
                    registrar.startInjecting(python)
                    for (inj in validPythonInjections) {
                        val range = TextRange(inj.start, inj.end)
                        registrar.addPlace(inj.prefix, inj.suffix, context, range)
                    }
                    registrar.doneInjecting()
                    LOG.debug("Python injection: ${validPythonInjections.size} pieces")
                }
            }

            // Inject HTML
            val html = HTML_LANGUAGE
            val htmlInjections = result.htmlInjections
            if (html != null && htmlInjections.isNotEmpty()) {
                val validHtmlInjections = htmlInjections.filter { inj ->
                    inj.start >= 0 && inj.end <= textLength && inj.start < inj.end
                }
                if (validHtmlInjections.isNotEmpty()) {
                    registrar.startInjecting(html)
                    for (inj in validHtmlInjections) {
                        val range = TextRange(inj.start, inj.end)
                        registrar.addPlace(inj.prefix, inj.suffix, context, range)
                    }
                    registrar.doneInjecting()
                    LOG.debug("HTML injection: ${validHtmlInjections.size} pieces")
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
