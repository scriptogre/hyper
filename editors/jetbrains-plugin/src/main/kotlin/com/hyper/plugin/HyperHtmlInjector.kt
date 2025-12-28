package com.hyper.plugin

import com.hyper.plugin.psi.HyperHtmlLine
import com.intellij.lang.Language
import com.intellij.lang.injection.MultiHostInjector
import com.intellij.lang.injection.MultiHostRegistrar
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.util.TextRange
import com.intellij.psi.PsiElement

class HyperHtmlInjector : MultiHostInjector {

    companion object {
        private val LOG = Logger.getInstance(HyperHtmlInjector::class.java)
        private val HTML_LANGUAGE: Language? by lazy {
            Language.findLanguageByID("HTML")
        }
    }

    override fun getLanguagesToInject(registrar: MultiHostRegistrar, context: PsiElement) {
        if (context !is HyperHtmlLine) return
        val html = HTML_LANGUAGE ?: return

        val text = context.text
        if (text.isBlank()) return

        val segments = findHtmlSegments(text)
        LOG.info("HTML injection: text='$text', segments=$segments")
        if (segments.isEmpty()) return

        registrar.startInjecting(html)
        for ((start, end) in segments) {
            LOG.info("  Adding HTML place: [$start, $end] = '${text.substring(start, end)}'")
            registrar.addPlace(null, null, context, TextRange(start, end))
        }
        registrar.doneInjecting()
    }

    private fun findHtmlSegments(text: String): List<Pair<Int, Int>> {
        val segments = mutableListOf<Pair<Int, Int>>()
        var segmentStart = 0
        var braceDepth = 0

        for ((i, c) in text.withIndex()) {
            when {
                c == '{' && braceDepth == 0 -> {
                    if (i > segmentStart) segments.add(segmentStart to i)
                    braceDepth = 1
                }
                c == '{' -> braceDepth++
                c == '}' && braceDepth > 0 -> {
                    braceDepth--
                    if (braceDepth == 0) segmentStart = i + 1
                }
            }
        }

        if (braceDepth == 0 && segmentStart < text.length) {
            segments.add(segmentStart to text.length)
        }

        return segments
    }

    override fun elementsToInjectIn(): MutableList<out Class<out PsiElement>> {
        return mutableListOf(HyperHtmlLine::class.java)
    }
}