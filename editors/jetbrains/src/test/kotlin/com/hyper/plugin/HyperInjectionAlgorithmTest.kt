package com.hyper.plugin

import com.hyper.plugin.HyperTranspilerService.Segment
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** Unit tests for buildInjections, the Kotlin port of the transpiler's old
 *  compute_injections (replaces the structural cases from injection_tests.rs). */
class HyperInjectionAlgorithmTest {

    @Test
    fun pythonInjectionsCarryCompiledPrefixAndTailSuffix() {
        // One Python segment covers "name" in source "<b>{name}</b>".
        val compiled = "from hyper import html\n\n@html\ndef T():\n    yield f'<b>{name}</b>'\n"
        val nameSourceStart = 4
        val nameSourceEnd = 8
        val nameCompiledStart = compiled.indexOf("{name}") + 1
        val nameCompiledEnd = nameCompiledStart + "name".length

        val segments = listOf(
            Segment(
                type = "python",
                source_start = nameSourceStart,
                source_end = nameSourceEnd,
                compiled_start = nameCompiledStart,
                compiled_end = nameCompiledEnd
            )
        )

        val injections = HyperTranspilerService.buildInjections(compiled, segments)
        val python = injections.filter { it.type == "python" }
        assertEquals(1, python.size)

        val py = python[0]
        // Source coordinates pass straight through from the segment.
        assertEquals(nameSourceStart, py.start)
        assertEquals(nameSourceEnd, py.end)
        // Prefix = compiled code up to the segment start.
        assertEquals(compiled.substring(0, nameCompiledStart), py.prefix)
        // Last (and only) segment carries the tail of compiled code as its suffix.
        assertEquals(compiled.substring(nameCompiledEnd), py.suffix)
        // Reconstruct: prefix + source slice + suffix == compiled code.
        val source = "<b>{name}</b>"
        val virtual = py.prefix + source.substring(py.start, py.end) + py.suffix
        assertEquals(compiled, virtual)
    }

    @Test
    fun multiplePythonSegmentsOnlyLastHasSuffix() {
        // Each prefix is the gap since the previous compiled_end; only the last gets a suffix.
        val compiled = "AAA<x>BBB<y>CCC"

        val segments = listOf(
            Segment(
                type = "python",
                source_start = 0, source_end = 1,
                compiled_start = 4, compiled_end = 5  // covers "x"
            ),
            Segment(
                type = "python",
                source_start = 2, source_end = 3,
                compiled_start = 10, compiled_end = 11 // covers "y"
            )
        )

        val python = HyperTranspilerService.buildInjections(compiled, segments)
            .filter { it.type == "python" }
        assertEquals(2, python.size)

        assertEquals("AAA<", python[0].prefix)
        assertEquals("", python[0].suffix)
        assertEquals(">BBB<", python[1].prefix)
        assertEquals(">CCC", python[1].suffix)
    }

    @Test
    fun pythonSegmentsSortByCompiledStart() {
        // Out-of-order input must be sorted by compiled_start before walking,
        // because prefix computation is sequential along the compiled code.
        val compiled = "AAA<x>BBB<y>CCC"

        val segments = listOf(
            Segment(
                type = "python",
                source_start = 2, source_end = 3,
                compiled_start = 10, compiled_end = 11
            ),
            Segment(
                type = "python",
                source_start = 0, source_end = 1,
                compiled_start = 4, compiled_end = 5
            )
        )

        val python = HyperTranspilerService.buildInjections(compiled, segments)
            .filter { it.type == "python" }
        assertEquals("AAA<", python[0].prefix)
        assertEquals(">BBB<", python[1].prefix)
    }

    @Test
    fun segmentsMarkedNoInjectionAreSkipped() {
        // Closing component-tag names get a Python segment for highlighting only,
        // with needs_injection=false. They must not appear in the injection list.
        val compiled = "AAA<x>BBB"

        val segments = listOf(
            Segment(
                type = "python",
                source_start = 0, source_end = 1,
                compiled_start = 4, compiled_end = 5
            ),
            Segment(
                type = "python",
                source_start = 5, source_end = 6,
                compiled_start = 0, compiled_end = 0,
                needs_injection = false
            )
        )

        val python = HyperTranspilerService.buildInjections(compiled, segments)
            .filter { it.type == "python" }
        assertEquals(1, python.size)
        assertEquals(0, python[0].start)
    }

    @Test
    fun htmlInjectionsHaveEmptyPrefixAndSuffixByDefault() {
        val compiled = "anything"

        val segments = listOf(
            Segment(
                type = "html",
                source_start = 0, source_end = 5,
                compiled_start = 0, compiled_end = 0
            ),
            Segment(
                type = "html",
                source_start = 6, source_end = 12,
                compiled_start = 0, compiled_end = 0
            )
        )

        val html = HyperTranspilerService.buildInjections(compiled, segments)
            .filter { it.type == "html" }
        assertEquals(2, html.size)
        for (inj in html) {
            assertEquals("", inj.prefix)
            assertEquals("", inj.suffix)
        }
    }

    @Test
    fun htmlSegmentsKeepTheirHtmlPrefix() {
        // Component-attribute HTML fragments carry an html_prefix so JetBrains' HTML
        // parser sees a valid synthetic tag (e.g. "<x ...").
        val segments = listOf(
            Segment(
                type = "html",
                source_start = 9, source_end = 21,
                compiled_start = 0, compiled_end = 0,
                html_prefix = "<x"
            )
        )

        val html = HyperTranspilerService.buildInjections("ignored", segments)
            .filter { it.type == "html" }
        assertEquals(1, html.size)
        assertEquals("<x", html[0].prefix)
        assertEquals("", html[0].suffix)
    }

    @Test
    fun htmlSegmentsSortBySourceStart() {
        val segments = listOf(
            Segment(
                type = "html",
                source_start = 50, source_end = 55,
                compiled_start = 0, compiled_end = 0
            ),
            Segment(
                type = "html",
                source_start = 5, source_end = 10,
                compiled_start = 0, compiled_end = 0
            )
        )

        val html = HyperTranspilerService.buildInjections("anything", segments)
            .filter { it.type == "html" }
        assertEquals(5, html[0].start)
        assertEquals(50, html[1].start)
    }

    @Test
    fun outOfBoundsCompiledOffsetsAreClamped() {
        // A malformed wire response with compiled offsets past the end of the
        // compiled string must not crash; coerceIn clamps to the string length.
        val compiled = "abc"
        val segments = listOf(
            Segment(
                type = "python",
                source_start = 0, source_end = 1,
                compiled_start = 100, compiled_end = 200
            )
        )

        val python = HyperTranspilerService.buildInjections(compiled, segments)
            .filter { it.type == "python" }
        assertEquals(1, python.size)
        assertTrue(
            "Prefix should fit within the compiled string, got: ${python[0].prefix}",
            python[0].prefix.length <= compiled.length
        )
        assertEquals("", python[0].suffix)
    }
}
