package com.hyper.plugin.lexer;

import com.intellij.lexer.FlexLexer;
import com.intellij.psi.tree.IElementType;

import static com.intellij.psi.TokenType.BAD_CHARACTER;
import static com.intellij.psi.TokenType.WHITE_SPACE;
import static com.hyper.plugin.psi.HyperTypes.*;

%%

%{ 
  public _HyperLexer() {
    this((java.io.Reader)null);
  }
%}

%public
%class _HyperLexer
%implements FlexLexer
%function advance
%type IElementType
%unicode

EOL=\R
// We use lookahead or specific patterns to distinguish types
// Note: Flex matches longest match first, then order.

// Matches lines starting with optional whitespace then <
HTML_LINE=[ \t]*"<"[^\r\n]*

// Matches lines starting with control keywords followed by space/colon/paren
// JFlex doesn't support \b, so we explicitly require a non-identifier char after keyword
// Also handles async def/for/with and try/except/finally/with
// Includes 'fragment' for Hyper template fragments
CONTROL_LINE=[ \t]*(async[ \t]+)?(if|for|while|match|def|class|elif|else|case|try|except|finally|with|fragment)[ \t(:][^\r\n]*

// Matches 'end' on a line by itself (with optional whitespace)
END_LINE=[ \t]*"end"[ \t]*

// Everything else is python
PYTHON_LINE=[^\r\n]+

%%

<YYINITIAL> {
  {HTML_LINE} {EOL}?     { return HTML_LINE_TOKEN; }
  {CONTROL_LINE} {EOL}?  { return CONTROL_LINE_TOKEN; }
  {END_LINE} {EOL}?      { return END_LINE_TOKEN; }
  {PYTHON_LINE} {EOL}?   { return PYTHON_LINE_TOKEN; }
  {EOL}                  { return PYTHON_LINE_TOKEN; } 
  [^]                    { return BAD_CHARACTER; }
}

