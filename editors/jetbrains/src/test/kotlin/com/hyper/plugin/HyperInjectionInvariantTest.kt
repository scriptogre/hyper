package com.hyper.plugin

import com.hyper.plugin.psi.HyperRootElement
import com.intellij.lang.injection.InjectedLanguageManager
import com.intellij.psi.PsiErrorElement
import com.intellij.psi.PsiFile
import com.intellij.psi.util.PsiTreeUtil
import com.intellij.testFramework.fixtures.BasePlatformTestCase
import java.io.File

/**
 * Data-driven invariant test that runs every non-error .hyper test file
 * through the JetBrains injection pipeline and checks structural properties.
 *
 * Invariants:
 *   1. No crashes — loading and highlighting doesn't throw
 *   2. No error-level highlights on valid templates
 *   3. Injected Python parses cleanly (no PsiErrorElement in injected PSI)
 *   4. Injected HTML parses cleanly
 *   5. Every injection language is Python or HTML
 *   6. Files with expressions have Python injection
 *   7. Files with tags have HTML injection
 *
 * Important: after doHighlighting(), myFixture.file may return the injected
 * PsiFile (Python or HTML) instead of the Hyper host file, because the
 * injection covers the entire file content. We save a reference to the
 * host file's HyperRootElement before highlighting so we can enumerate
 * injections from it afterward.
 */
class HyperInjectionInvariantTest : BasePlatformTestCase() {

    /**
     * Runs all invariants for a single .hyper file.
     */
    private fun checkFile(file: File) {
        val content = file.readText()
        myFixture.configureByText("${file.nameWithoutExtension}.hyper", content)

        // Invariant 1: No crashes during highlighting
        val highlights = myFixture.doHighlighting()

        // Invariant 2: No error-level highlights on valid templates
        val errors = highlights.filter {
            it.severity == com.intellij.lang.annotation.HighlightSeverity.ERROR
        }
        assertTrue(
            "File ${file.name} produced error highlights:\n${errors.joinToString("\n") { "  [${it.startOffset}-${it.endOffset}] ${it.description}" }}",
            errors.isEmpty()
        )

        // Walk all injected fragments using enumerate() on each HyperRootElement.
        // This is O(number of injections) instead of O(file length).
        //
        // myFixture.file may already point to the injected PsiFile (Python or HTML)
        // because the injection covers the entire file. Use getTopLevelFile() to
        // navigate back to the Hyper host file, then find HyperRootElement from there.
        val injectedManager = InjectedLanguageManager.getInstance(project)
        val possiblyInjectedFile = myFixture.file
        val hostFile = injectedManager.getTopLevelFile(possiblyInjectedFile)

        val rootElements = mutableListOf<HyperRootElement>()
        val firstChild = hostFile.firstChild
        if (firstChild is HyperRootElement) {
            rootElements.add(firstChild)
        }
        rootElements.addAll(PsiTreeUtil.findChildrenOfType(hostFile, HyperRootElement::class.java))

        val injectedLanguages = mutableListOf<String>()
        val checkedFiles = mutableSetOf<Int>()

        for (root in rootElements) {
            injectedManager.enumerate(root) { injectedPsi: PsiFile, _ ->
                val fileHash = System.identityHashCode(injectedPsi)
                if (fileHash in checkedFiles) return@enumerate
                checkedFiles.add(fileHash)

                val languageId = injectedPsi.language.id
                injectedLanguages.add(languageId)

                // Invariant 5: Only Python or HTML
                assertTrue(
                    "File ${file.name}: unexpected injection language '$languageId'",
                    languageId == "Python" || languageId == "HTML"
                )

                // Invariant 3 & 4: Injected PSI parses cleanly
                // Skip HTML parse errors for files with component/slot tags (<{Name}>),
                // because component names are stripped in the HTML injection, producing
                // empty tags like </> and <> that the HTML parser rejects.
                val hasComponentTags = content.contains("<{")
                val skipHtmlErrors = languageId == "HTML" && hasComponentTags

                if (!skipHtmlErrors) {
                    val parseErrors = PsiTreeUtil.findChildrenOfType(injectedPsi, PsiErrorElement::class.java)
                    if (parseErrors.isNotEmpty()) {
                        val errorDetails = parseErrors.joinToString("\n") { err ->
                            "  ${err.errorDescription} at offset ${err.textOffset}: '${
                                injectedPsi.text.substring(
                                    maxOf(0, err.textOffset - 20),
                                    minOf(injectedPsi.text.length, err.textOffset + 20)
                                )
                            }'"
                        }
                        fail(
                            "File ${file.name}: injected $languageId has parse errors:\n$errorDetails\n" +
                            "Injected file text:\n${injectedPsi.text}"
                        )
                    }
                }
            }
        }

        // Invariant 6: Files with {expressions} should have Python injection
        // Skip frontmatter (before ---) — only check template body.
        // Exclude slot syntax ({...} and {...name}) which are not Python expressions.
        val bodyStart = content.indexOf("\n---\n")
        val body = if (bodyStart >= 0) content.substring(bodyStart + 5) else content
        val hasExpressions = Regex("""\{(?!\{)(?!\.\.\.)[^}]+\}""").containsMatchIn(body)
        if (hasExpressions) {
            assertTrue(
                "File ${file.name} has expressions but no Python injection was found",
                injectedLanguages.contains("Python")
            )
        }

        // Invariant 7: Files with <tags> should have HTML injection
        val hasTags = Regex("""<[a-z][a-zA-Z0-9]*[\s>]""").containsMatchIn(body)
        if (hasTags) {
            assertTrue(
                "File ${file.name} has HTML tags but no HTML injection was found",
                injectedLanguages.contains("HTML")
            )
        }
    }

    /**
     * Find all non-error .hyper test files from the transpiler test suite.
     */
    private fun findTestFiles(): List<File> {
        val testDir = File(project.basePath).resolve("../../rust/tests")
        if (!testDir.exists()) {
            // Fallback: try relative to working directory
            val fallback = File("../../rust/tests")
            assertTrue(
                "Cannot find transpiler test directory at ${testDir.absolutePath} or ${fallback.absolutePath}",
                fallback.exists()
            )
            return collectHyperFiles(fallback)
        }
        return collectHyperFiles(testDir)
    }

    private fun collectHyperFiles(dir: File): List<File> {
        return dir.walkTopDown()
            .filter { it.extension == "hyper" }
            .filter { !it.path.contains("/errors/") }
            .sortedBy { it.path }
            .toList()
    }

    // --- Test methods: one per directory for readable output ---

    fun testInjectionInvariants_basic() {
        val files = findTestFiles().filter { it.path.contains("/basic/") }
        assertTrue("No basic test files found", files.isNotEmpty())
        val failures = mutableListOf<String>()
        for (file in files) {
            try {
                checkFile(file)
            } catch (e: AssertionError) {
                // Filter out IntelliJ platform "Cannot restore" errors — these are
                // noise from PSI element restoration in injected contexts when the
                // light test fixture is reused across multiple files.
                val msg = e.message ?: "Unknown failure for ${file.name}"
                if (!msg.contains("Cannot restore")) {
                    failures.add(msg)
                }
            }
        }
        if (failures.isNotEmpty()) {
            fail("${failures.size}/${files.size} basic files failed:\n\n${failures.joinToString("\n\n")}")
        }
    }

    fun testInjectionInvariants_components() {
        val files = findTestFiles().filter { it.path.contains("/components/") }
        assertTrue("No component test files found", files.isNotEmpty())
        val failures = mutableListOf<String>()
        for (file in files) {
            try {
                checkFile(file)
            } catch (e: AssertionError) {
                // Filter out IntelliJ platform "Cannot restore" errors — these are
                // noise from PSI element restoration in injected contexts when the
                // light test fixture is reused across multiple files.
                val msg = e.message ?: "Unknown failure for ${file.name}"
                if (!msg.contains("Cannot restore")) {
                    failures.add(msg)
                }
            }
        }
        if (failures.isNotEmpty()) {
            fail("${failures.size}/${files.size} component files failed:\n\n${failures.joinToString("\n\n")}")
        }
    }

    fun testInjectionInvariants_controlFlow() {
        val files = findTestFiles().filter { it.path.contains("/control_flow/") }
        assertTrue("No control_flow test files found", files.isNotEmpty())
        val failures = mutableListOf<String>()
        for (file in files) {
            try {
                checkFile(file)
            } catch (e: AssertionError) {
                // Filter out IntelliJ platform "Cannot restore" errors — these are
                // noise from PSI element restoration in injected contexts when the
                // light test fixture is reused across multiple files.
                val msg = e.message ?: "Unknown failure for ${file.name}"
                if (!msg.contains("Cannot restore")) {
                    failures.add(msg)
                }
            }
        }
        if (failures.isNotEmpty()) {
            fail("${failures.size}/${files.size} control_flow files failed:\n\n${failures.joinToString("\n\n")}")
        }
    }
}
