package com.hyper.plugin

import com.intellij.lang.annotation.AnnotationHolder
import com.intellij.lang.annotation.ExternalAnnotator
import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.openapi.diagnostic.Logger
import com.intellij.openapi.editor.Document
import com.intellij.openapi.util.TextRange
import com.intellij.psi.PsiFile

/**
 * External annotator that runs the Hyper transpiler and shows errors as red squiggles.
 * Uses the ExternalAnnotator pattern: collectInformation (EDT) -> doAnnotate (background) -> apply (EDT).
 */
class HyperExternalAnnotator : ExternalAnnotator<HyperExternalAnnotator.Info, HyperExternalAnnotator.Result>() {

    companion object {
        private val LOG = Logger.getInstance(HyperExternalAnnotator::class.java)
    }

    data class Info(
        val content: String,
        val fileName: String?,
        val document: Document
    )

    data class ErrorAnnotation(
        val message: String,
        val startOffset: Int,
        val endOffset: Int
    )

    data class Result(
        val errors: List<ErrorAnnotation>
    )

    override fun collectInformation(file: PsiFile): Info? {
        if (file.fileType !is HyperFileType) return null
        val document = file.viewProvider.document ?: return null
        val fileName = file.virtualFile?.nameWithoutExtension
        return Info(file.text, fileName, document)
    }

    override fun doAnnotate(info: Info): Result {
        val project = com.intellij.openapi.project.ProjectManager.getInstance().openProjects.firstOrNull()
            ?: return Result(emptyList())

        return try {
            val service = HyperTranspilerService.getInstance(project)
            // Just compile — if it succeeds, no errors to show
            service.transpile(info.content, includeInjection = false, functionName = info.fileName)
            Result(emptyList())
        } catch (e: HyperTranspilerService.TranspileException) {
            val errors = mutableListOf<ErrorAnnotation>()

            val line = e.line
            val col = e.col
            val endLine = e.endLine
            val endCol = e.endCol

            if (line != null && col != null) {
                val document = info.document
                val lineCount = document.lineCount

                // Convert 0-based line/col to document offset
                if (line < lineCount) {
                    val startOffset = document.getLineStartOffset(line) + col
                    val endOffset = if (endLine != null && endCol != null && endLine < lineCount) {
                        document.getLineStartOffset(endLine) + endCol
                    } else {
                        // Default to end of line
                        document.getLineEndOffset(line).coerceAtMost(document.textLength)
                    }

                    // Ensure valid range
                    val safeStart = startOffset.coerceIn(0, document.textLength)
                    val safeEnd = endOffset.coerceIn(safeStart, document.textLength)

                    // If start == end, extend to end of line for visibility
                    val finalEnd = if (safeStart == safeEnd) {
                        document.getLineEndOffset(line).coerceIn(safeStart, document.textLength)
                    } else {
                        safeEnd
                    }

                    errors.add(ErrorAnnotation(
                        message = e.message ?: "Transpiler error",
                        startOffset = safeStart,
                        endOffset = finalEnd
                    ))
                }
            } else {
                // No position info — annotate the first line
                errors.add(ErrorAnnotation(
                    message = e.message ?: "Transpiler error",
                    startOffset = 0,
                    endOffset = info.document.getLineEndOffset(0).coerceAtMost(info.document.textLength)
                ))
            }

            Result(errors)
        } catch (e: Exception) {
            LOG.debug("Unexpected error during annotation", e)
            Result(emptyList())
        }
    }

    override fun apply(file: PsiFile, result: Result, holder: AnnotationHolder) {
        for (error in result.errors) {
            val range = TextRange(error.startOffset, error.endOffset)
            holder.newAnnotation(HighlightSeverity.ERROR, error.message)
                .range(range)
                .create()
        }
    }
}
