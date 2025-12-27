package com.hyper.plugin

import com.intellij.openapi.fileTypes.LanguageFileType
import javax.swing.Icon

object HyperFileType : LanguageFileType(HyperLanguage) {
    override fun getName(): String = "Hyper"
    override fun getDescription(): String = "Hyper template file"
    override fun getDefaultExtension(): String = "hyper"
    override fun getIcon(): Icon? = HyperIcons.FILE
}
