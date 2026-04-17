package com.hyper.plugin.psi;

import com.hyper.plugin.HyperLanguage;
import com.intellij.psi.tree.IElementType;
import org.jetbrains.annotations.NotNull;

public class HyperElementType extends IElementType {
    public HyperElementType(@NotNull String debugName) {
        super(debugName, HyperLanguage.INSTANCE);
    }
}
