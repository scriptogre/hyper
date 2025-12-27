package com.hyper.plugin

import com.intellij.extapi.psi.PsiFileBase
import com.intellij.openapi.fileTypes.FileType
import com.intellij.psi.FileViewProvider

class HyperFile(viewProvider: FileViewProvider) : PsiFileBase(viewProvider, HyperLanguage) {
    override fun getFileType(): FileType = HyperFileType
    override fun toString(): String = "Hyper File"
}
