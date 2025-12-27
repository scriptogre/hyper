package com.hyper.plugin.actions

import com.hyper.plugin.HyperFile
import com.hyper.plugin.HyperVirtualFileGenerator
import com.hyper.plugin.psi.HyperRootElement
import com.intellij.openapi.actionSystem.ActionUpdateThread
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.intellij.openapi.application.runWriteAction
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.vfs.VfsUtil
import com.intellij.psi.util.PsiTreeUtil

class ShowVirtualPythonAction : AnAction() {

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val psiFile = e.getData(CommonDataKeys.PSI_FILE) as? HyperFile ?: return
        val virtualFile = e.getData(CommonDataKeys.VIRTUAL_FILE) ?: return
        val rootElement = PsiTreeUtil.findChildOfType(psiFile, HyperRootElement::class.java) ?: return

        val sb = StringBuilder()
        HyperVirtualFileGenerator.generate(rootElement) { prefix, suffix, _, range ->
            sb.append(prefix)
            if (range.length > 0) {
                sb.append(rootElement.text.substring(range.startOffset, range.endOffset))
            }
            sb.append(suffix)
        }

        val content = sb.toString()
        if (content.isEmpty()) return

        val parentDir = virtualFile.parent ?: return
        val outputName = virtualFile.nameWithoutExtension + ".virtual.py"

        runWriteAction {
            val outputFile = parentDir.findOrCreateChildData(this, outputName)
            VfsUtil.saveText(outputFile, content)
            FileEditorManager.getInstance(project).openFile(outputFile, true)
        }
    }

    override fun update(e: AnActionEvent) {
        val psiFile = e.getData(CommonDataKeys.PSI_FILE)
        e.presentation.isEnabledAndVisible = psiFile is HyperFile
    }

    override fun getActionUpdateThread() = ActionUpdateThread.BGT
}
