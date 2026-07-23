package com.hyper.plugin

import com.intellij.testFramework.fixtures.BasePlatformTestCase
import java.io.File

class HyperZeroBuildTest : BasePlatformTestCase() {
    fun testPluginDoesNotRegisterGeneratedPythonFeatures() {
        val descriptor = File("src/main/resources/META-INF/plugin.xml").readText()

        assertFalse(descriptor.contains("HyperFileListener"))
        assertFalse(descriptor.contains("HyperFileNestingProvider"))
    }
}
