plugins {
    id("java")
    id("org.jetbrains.kotlin.jvm") version "2.1.0"
    id("org.jetbrains.kotlin.plugin.serialization") version "2.1.0"
    id("org.jetbrains.intellij.platform") version "2.2.1"
    id("org.jetbrains.grammarkit") version "2022.3.2.2"
}

group = "com.hyper"
version = "0.1.0"

repositories {
    mavenCentral()
    intellijPlatform {
        defaultRepositories()
    }
}

sourceSets {
    main {
        java {
            srcDirs("src/main/gen")
        }
    }
}

dependencies {
    // Match PyCharm 2025.1's bundled kotlinx-serialization; an older pin shadows it and hangs test startup.
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.7.3")

    intellijPlatform {
        pycharmCommunity("2025.1")
        bundledPlugin("PythonCore")
        testFramework(org.jetbrains.intellij.platform.gradle.TestFrameworkType.Platform)
    }

    testImplementation("junit:junit:4.13.2")
}

grammarKit {
    jflexRelease.set("1.7.0-1")
    grammarKitRelease.set("2022.3.2")
}

tasks {
    generateLexer {
        sourceFile.set(file("src/main/grammar/Hyper.flex"))
        targetOutputDir.set(file("src/main/gen/com/hyper/plugin/lexer"))
        purgeOldFiles.set(true)
    }

    generateParser {
        sourceFile.set(file("src/main/grammar/Hyper.bnf"))
        targetRootOutputDir.set(file("src/main/gen"))
        pathToParser.set("com/hyper/plugin/parser/HyperParser.java")
        pathToPsiRoot.set("com/hyper/plugin/psi")
        purgeOldFiles.set(true)
    }

    compileKotlin {
        dependsOn(generateLexer, generateParser)
    }

    compileJava {
        dependsOn(generateLexer, generateParser)
    }

    buildSearchableOptions {
        enabled = false
    }

    test {
        systemProperty("hyper.binary.path",
            rootProject.file("../../rust/target/debug/hyper").absolutePath)
        systemProperty("java.awt.headless", "true")
        systemProperty("idea.classpath.index.enabled", "false")
        jvmArgs = listOf("-Xmx4g", "-XX:+UseG1GC")
        workingDir = project.projectDir
    }
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}

kotlin {
    jvmToolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}

intellijPlatform {
    sandboxContainer = layout.projectDirectory.dir(".sandbox")

    pluginConfiguration {
        name = "Hyper"
        version = project.version.toString()
        ideaVersion {
            sinceBuild = "243"
            untilBuild = provider { null }
        }
    }
}
