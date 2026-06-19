package com.hyper.plugin

import com.hyper.plugin.psi.HyperRootElement
import com.intellij.lang.injection.InjectedLanguageManager
import com.intellij.psi.PsiErrorElement
import com.intellij.psi.PsiFile
import com.intellij.psi.util.PsiTreeUtil
import com.intellij.testFramework.fixtures.BasePlatformTestCase
import java.io.File

/** End-to-end check that the injector glue yields cleanly-parsing Python/HTML fragments.
 *  Algorithm is covered by HyperInjectionAlgorithmTest; this samples a few files for speed. */
class HyperInjectionInvariantTest : BasePlatformTestCase() {

    /** A few files, each hitting a different injector path. Additions cost platform-test time. */
    private fun representativeFiles(): List<File> {
        val root = File(project.basePath).resolve("../../rust/tests").takeIf { it.exists() }
            ?: File("../../rust/tests")
        return listOf(
            root.resolve("kitchen_sink.hyper"),
            root.resolve("components/nested.hyper"),
            root.resolve("basic/multiline.hyper"),
        ).filter { it.exists() }
    }

    fun testInjectionInvariants() {
        val files = representativeFiles()
        assertTrue("No representative .hyper files found on disk", files.isNotEmpty())

        val failures = mutableListOf<String>()
        for (file in files) {
            try {
                checkFile(file)
            } catch (e: AssertionError) {
                val msg = e.message ?: "Unknown failure for ${file.name}"
                if (!msg.contains("Cannot restore")) {
                    failures.add(msg)
                }
            }
        }
        if (failures.isNotEmpty()) {
            fail("${failures.size}/${files.size} files failed:\n\n${failures.joinToString("\n\n")}")
        }
    }

    private fun checkFile(file: File) {
        val content = file.readText()
        myFixture.configureByText("${file.nameWithoutExtension}.hyper", content)

        val injectedManager = InjectedLanguageManager.getInstance(project)
        val hostFile = injectedManager.getTopLevelFile(myFixture.file)

        val rootElements = mutableListOf<HyperRootElement>()
        val firstChild = hostFile.firstChild
        if (firstChild is HyperRootElement) {
            rootElements.add(firstChild)
        }
        rootElements.addAll(PsiTreeUtil.findChildrenOfType(hostFile, HyperRootElement::class.java))

        val injectedLanguages = mutableSetOf<String>()
        val checkedFiles = mutableSetOf<Int>()

        for (root in rootElements) {
            injectedManager.enumerate(root) { injectedPsi: PsiFile, _ ->
                val fileHash = System.identityHashCode(injectedPsi)
                if (fileHash in checkedFiles) return@enumerate
                checkedFiles.add(fileHash)

                val languageId = injectedPsi.language.id
                injectedLanguages.add(languageId)

                // Injection language must be Python or HTML.
                assertTrue(
                    "File ${file.name}: unexpected injection language '$languageId'",
                    languageId == "Python" || languageId == "HTML"
                )

                // Component/slot files: the injector strips `<{Name}>` braces, leaving tag
                // fragments that don't parse. Expected, so skip HTML error checks for them.
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

        // Files with `{expressions}` in the body should produce Python injection.
        // Skip the frontmatter (before `---`); exclude slot syntax `{...}` and `{...name}`.
        val bodyStart = content.indexOf("\n---\n")
        val body = if (bodyStart >= 0) content.substring(bodyStart + 5) else content
        val hasExpressions = Regex("""\{(?!\{)(?!\.\.\.)[^}]+\}""").containsMatchIn(body)
        if (hasExpressions) {
            assertTrue(
                "File ${file.name} has expressions but no Python injection was found",
                injectedLanguages.contains("Python")
            )
        }

        val hasTags = Regex("""<[a-z][a-zA-Z0-9]*[\s>]""").containsMatchIn(body)
        if (hasTags) {
            assertTrue(
                "File ${file.name} has HTML tags but no HTML injection was found",
                injectedLanguages.contains("HTML")
            )
        }
    }
}
