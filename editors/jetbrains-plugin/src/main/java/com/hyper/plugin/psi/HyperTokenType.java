package com.hyper.plugin.psi;

import com.hyper.plugin.HyperLanguage;
import com.intellij.psi.tree.IElementType;
import org.jetbrains.annotations.NotNull;

public class HyperTokenType extends IElementType {
    public HyperTokenType(@NotNull String debugName) {
        super(debugName, HyperLanguage.INSTANCE);
    }

    @Override
    public String toString() {
        return "HyperTokenType." + super.toString();
    }
}
