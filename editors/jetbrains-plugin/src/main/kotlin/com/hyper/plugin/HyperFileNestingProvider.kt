package com.hyper.plugin

import com.intellij.ide.projectView.TreeStructureProvider
import com.intellij.ide.projectView.ViewSettings
import com.intellij.ide.projectView.impl.nodes.NestingTreeNode
import com.intellij.ide.projectView.impl.nodes.PsiFileNode
import com.intellij.ide.util.treeView.AbstractTreeNode

/**
 * Provides file nesting for .hyper files.
 * Makes .py files appear nested under their corresponding .hyper files.
 */
class HyperFileNestingProvider : TreeStructureProvider {

    private fun getGeneratedPath(hyperPath: String): String {
        return hyperPath.removeSuffix(".hyper") + ".py"
    }

    override fun modify(
        parent: AbstractTreeNode<*>,
        children: MutableCollection<AbstractTreeNode<*>>,
        settings: ViewSettings?
    ): Collection<AbstractTreeNode<*>> {
        val result = mutableListOf<AbstractTreeNode<*>>()

        // Map hyper file paths to their nodes
        val hyperFiles = mutableMapOf<String, PsiFileNode>()
        // Map generated file paths to their nodes
        val generatedFiles = mutableMapOf<String, MutableList<PsiFileNode>>()

        // First pass: collect hyper files and their expected generated paths
        for (child in children) {
            if (child is PsiFileNode) {
                val file = child.virtualFile
                if (file != null && file.name.endsWith(".hyper")) {
                    hyperFiles[file.path] = child
                }
            }
        }

        // Second pass: collect generated files and other files
        for (child in children) {
            if (child is PsiFileNode) {
                val file = child.virtualFile
                if (file != null) {
                    when {
                        file.name.endsWith(".hyper") -> {
                            // Skip for now, will add with nested children
                        }
                        file.name.endsWith(".py") -> {
                            // Check if this is a generated file for a hyper file
                            val hyperPath = file.path.removeSuffix(".py") + ".hyper"
                            if (hyperFiles.containsKey(hyperPath)) {
                                generatedFiles.getOrPut(hyperPath) { mutableListOf() }.add(child)
                            } else {
                                result.add(child)
                            }
                        }
                        else -> {
                            result.add(child)
                        }
                    }
                } else {
                    result.add(child)
                }
            } else {
                result.add(child)
            }
        }

        // Third pass: create nesting nodes for hyper files
        for ((hyperPath, hyperNode) in hyperFiles) {
            val nested = generatedFiles[hyperPath]
            if (nested != null && nested.isNotEmpty()) {
                result.add(NestingTreeNode(hyperNode, nested))
            } else {
                result.add(hyperNode)
            }
        }

        return result
    }
}
