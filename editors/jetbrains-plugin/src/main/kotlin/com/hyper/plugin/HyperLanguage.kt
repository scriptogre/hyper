package com.hyper.plugin

import com.intellij.lang.Language

object HyperLanguage : Language("Hyper") {
    override fun getDisplayName(): String = "Hyper"
    override fun isCaseSensitive(): Boolean = true
}
