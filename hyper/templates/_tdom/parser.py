import sys
import typing as t
from string.templatelib import Template, Interpolation
from html.parser import HTMLParser
from dataclasses import dataclass, field
from functools import lru_cache

from .nodes import (
    VOID_ELEMENTS,
    TNode,
    TComment,
    TDocumentType,
    TElement,
    TFragment,
    TText,
    TComponent,
    TAttribute,
    InterpolatedAttribute,
    TemplatedAttribute,
    StaticAttribute,
    SpreadAttribute,
    TConditional,
    TConditionalBranch,
    TMatch,
    TCase,
)
from .placeholders import (
    placeholder as construct_placeholder,
    find_placeholder as deconstruct_placeholder,
    placeholders_to_template,
    FRAGMENT_TAG,
)

type OpenTag = (
    OpenTElement | OpenTFragment | OpenTComponent | OpenTConditional | OpenTMatch
)


@dataclass
class OpenTElement:
    tag: str
    attrs: tuple[TAttribute, ...]
    children: list[TNode] = field(default_factory=list)


@dataclass
class OpenTFragment:
    children: list[TNode] = field(default_factory=list)


@dataclass
class OpenTComponent:
    starttag_interpolation_index: int
    starttag_string_index: int
    attrs: tuple[TAttribute, ...]
    children: list[TNode] = field(default_factory=list)


@dataclass
class OpenTConditional:
    """Tracks state while parsing if/elif/else directives."""

    branches: list[TConditionalBranch] = field(default_factory=list)
    current_branch_children: list[TNode] = field(default_factory=list)


@dataclass
class OpenTMatch:
    """Tracks state while parsing match/case directives."""

    subject_index: int
    cases: list[TCase] = field(default_factory=list)
    current_case_children: list[TNode] | None = None


@dataclass
class TemplateState:
    template: Template = field(default_factory=lambda: Template(""))
    root: OpenTFragment = field(default_factory=OpenTFragment)
    stack: list[OpenTag] = field(default_factory=list)
    active_placeholders: dict[str, int] = field(default_factory=dict)
    strings_index: int = -1
    interpolations_index: int = -1

    @property
    def interpolations(self):
        return self.template.interpolations


class TemplateParser(HTMLParser):
    tstate: TemplateState

    def __init__(self, *, convert_charrefs=True):
        self.tstate = TemplateState()
        super().__init__(convert_charrefs=convert_charrefs)

    def handle_attrs(
        self, attrs: t.Sequence[tuple[str, str | None]]
    ) -> tuple[TAttribute, ...]:
        """Create appropriate attribute types, finding and remove placeholdersas needed."""
        new_attrs: list[TAttribute] = []
        for k, v in attrs:
            k_parts = tuple(self.extract_template(k))
            v_parts = (
                (tuple(self.extract_template(v)) or ("",)) if v is not None else None
            )
            match k_parts, v_parts:
                case [Interpolation(value=interpolation_index)], None:
                    new_attrs.append(
                        SpreadAttribute(interpolation_index=interpolation_index)
                    )
                case [str() as name], None:
                    new_attrs.append(StaticAttribute(name=name, value=None))
                case [str() as name], [Interpolation(value=interpolation_index)]:
                    new_attrs.append(
                        InterpolatedAttribute(
                            name=name, interpolation_index=interpolation_index
                        )
                    )
                case [str() as name], [str() | Interpolation(), _, *_]:
                    new_attrs.append(
                        TemplatedAttribute(name=name, value_t=Template(*v_parts))
                    )
                case [str() as name], [str() as value]:
                    new_attrs.append(StaticAttribute(name=name, value=value))
                case _:
                    raise ValueError(
                        f"Unupported combination of attribute name/value: {k_parts}={v_parts}"
                    )
        return tuple(new_attrs)

    def handle_starttag(
        self, tag: str, attrs: t.Sequence[tuple[str, str | None]]
    ) -> None:
        """Dispatch *opening* a tag specialized handlers."""
        tag_t = list(self.extract_template(tag, ""))
        match tag_t:
            case [Interpolation(value=interpolation_index)]:
                open_component = self.handle_start_component(interpolation_index, attrs)
                self.tstate.stack.append(open_component)
            case [str()]:
                if tag == FRAGMENT_TAG:
                    open_fragment = self.handle_start_fragment(tag, attrs)
                    self.tstate.stack.append(open_fragment)
                else:
                    open_element = self.handle_start_element(tag, attrs)
                    if open_element.tag in VOID_ELEMENTS:
                        self.append_child(
                            TElement(tag=open_element.tag, attrs=open_element.attrs)
                        )
                    else:
                        self.tstate.stack.append(open_element)
            case _:
                raise ValueError(
                    "Component tags should be an exact match."
                )  # @TODO: Cleanup

    def handle_start_fragment(
        self, tag: str, attrs: t.Sequence[tuple[str, str | None]]
    ) -> OpenTFragment:
        if attrs:
            raise TypeError("Fragments cannot have attributes.")
        return OpenTFragment()

    def handle_start_element(
        self, tag: str, attrs: t.Sequence[tuple[str, str | None]]
    ) -> OpenTElement:
        return OpenTElement(tag, self.handle_attrs(attrs))

    def handle_start_component(
        self, interpolation_index: int, attrs: t.Sequence[tuple[str, str | None]]
    ) -> OpenTComponent:
        ip = self.tstate.interpolations[interpolation_index]
        # Skip check when value is an int (codegen mode uses indices as placeholders)
        if not isinstance(ip.value, int) and not callable(ip.value):
            expr = getattr(ip, "expression", repr(ip.value))
            raise TypeError(
                f"Component <{{{expr}}}> is not callable. "
                f"Got {type(ip.value).__name__}: {ip.value!r}"
            )
        return OpenTComponent(
            starttag_interpolation_index=interpolation_index,
            starttag_string_index=self.tstate.strings_index,
            attrs=self.handle_attrs(attrs),
        )

    def handle_startendtag(
        self, tag: str, attrs: t.Sequence[tuple[str, str | None]]
    ) -> None:
        """Dispatch a self-closing tag, `<tag />` to specialized handlers."""
        tag_t = list(self.extract_template(tag, ""))
        match tag_t:
            case [Interpolation(value=interpolation_index)]:
                component = self.handle_startend_component(interpolation_index, attrs)
                self.append_child(component)
            case [str() as starttag]:
                if starttag == FRAGMENT_TAG:
                    fragment = self.handle_startend_fragment(starttag, attrs)
                    self.append_child(fragment)
                else:
                    element = self.handle_startend_element(starttag, attrs)
                    self.append_child(element)
            case _:
                raise ValueError(
                    "Component tags should be an exact match."
                )  # @TODO: Cleanup

    def handle_startend_fragment(
        self, startendtag: str, attrs: t.Sequence[tuple[str, str | None]]
    ) -> TFragment:
        if attrs:  # @TODO: Here or inside the dispatcher itself?
            raise TypeError("Fragments cannot have attributes.")
        return TFragment()

    def handle_startend_element(
        self, startendtag: str, attrs: t.Sequence[tuple[str, str | None]]
    ) -> TElement:
        return TElement(startendtag, attrs=self.handle_attrs(attrs))

    def handle_startend_component(
        self, interpolation_index: int, attrs: t.Sequence[tuple[str, str | None]]
    ) -> TComponent:
        ip = self.tstate.interpolations[interpolation_index]
        # Skip check when value is an int (codegen mode uses indices as placeholders)
        if not isinstance(ip.value, int) and not callable(ip.value):
            expr = getattr(ip, "expression", repr(ip.value))
            raise TypeError(
                f"Component <{{{expr}}}> is not callable. "
                f"Got {type(ip.value).__name__}: {ip.value!r}"
            )
        return TComponent(
            starttag_interpolation_index=interpolation_index,
            endtag_interpolation_index=interpolation_index,
            starttag_string_index=self.tstate.strings_index,
            endtag_string_index=self.tstate.strings_index,
            attrs=self.handle_attrs(attrs),
        )

    def handle_endtag(self, tag: str) -> None:
        """Dispatch *closing* a tag, `</tag>`, to specialized handlers."""
        tag_t = list(self.extract_template(tag, ""))
        match tag_t:
            case [Interpolation(value=interpolation_index)]:
                component = self.handle_end_component(interpolation_index)
                self.append_child(component)
            case [str()]:
                if tag == FRAGMENT_TAG:
                    fragment = self.handle_end_fragment(tag)
                    self.append_child(fragment)
                else:
                    element = self.handle_end_element(tag)
                    self.append_child(element)
            case _:
                raise ValueError("Component end tag must be an exact match.")

    def handle_end_component(self, interpolation_index: int) -> TComponent:
        if not self.tstate.stack:
            raise ValueError(
                f"Unexpected closing tag </{self.get_comp_endtag(interpolation_index)}> with no open tag."
            )
        open_tag = self.tstate.stack.pop()
        match open_tag:
            case OpenTElement():
                raise TypeError(
                    f"Mismatched closing tag </{self.get_comp_endtag(interpolation_index)}> for </{open_tag.tag}>."
                )
            case OpenTFragment():
                raise TypeError(
                    f"Mismatched closing tag </{self.get_comp_endtag(interpolation_index)}> for </>."
                )
            case OpenTComponent():
                if (
                    self.tstate.interpolations[
                        open_tag.starttag_interpolation_index
                    ].value
                    != self.tstate.interpolations[interpolation_index].value
                ):
                    raise ValueError(
                        f"Mismatched component value for <{self.get_comp_starttag(open_tag.starttag_interpolation_index)}> and </{self.get_comp_endtag(interpolation_index)}>"
                    )
                return TComponent(
                    starttag_interpolation_index=open_tag.starttag_interpolation_index,
                    endtag_interpolation_index=interpolation_index,
                    starttag_string_index=open_tag.starttag_string_index,
                    endtag_string_index=self.tstate.strings_index,
                    attrs=open_tag.attrs,
                    children=tuple(open_tag.children),
                )

    def handle_end_element(self, tag: str) -> TElement:
        if not self.tstate.stack:
            raise ValueError(f"Unexpected closing tag </{tag}> with no open tag.")
        open_tag = self.tstate.stack.pop()
        match open_tag:
            case OpenTElement():
                if open_tag.tag != tag:
                    raise ValueError(
                        f"Mismatched closing tag </{tag}> for <{self.get_container_starttag(open_tag)}>."
                    )
                return TElement(
                    open_tag.tag,
                    attrs=open_tag.attrs,
                    children=tuple(open_tag.children),
                )
            case OpenTFragment():
                raise TypeError(
                    f"Mismatched closing tag </{tag}> for </{self.get_container_starttag(open_tag)}>."
                )
            case OpenTComponent():
                raise TypeError(
                    f"Mismatched closing tag </{tag}> for </{self.get_container_starttag(open_tag)}>."
                )

    def handle_end_fragment(self, tag: str) -> TFragment:
        if not self.tstate.stack:
            raise ValueError("Unexpected closing tag </> with no open tag.")
        open_tag = self.tstate.stack.pop()
        match open_tag:
            case OpenTFragment():
                return TFragment(children=tuple(open_tag.children))
            case OpenTElement():
                raise TypeError(
                    f"Mismatched closing tag </> for <{self.get_container_starttag(open_tag)}>."
                )
            case OpenTComponent():
                raise TypeError(
                    f"Mismatched closing tag </> for <{self.get_container_starttag(open_tag)}>."
                )

    def handle_data(self, data: str) -> None:
        text_t = self.extract_template(data)
        last_text_child = self.get_latest_text_child()
        if last_text_child:
            last_text_child.text_t += text_t
        else:
            text = TText(text_t)
            self.append_child(text)

    def handle_comment(self, data: str) -> None:
        # Check if this is a directive comment
        stripped = data.lstrip()
        if stripped.startswith("@"):
            self.handle_directive_comment(stripped[1:].strip())
        elif stripped.startswith("#"):
            # Server-side comment - don't include in output
            pass
        else:
            # Regular HTML comment
            text_t = self.extract_template(data)
            comment = TComment(text_t)
            self.append_child(comment)

    def handle_directive_comment(self, directive: str) -> None:
        """Parse and handle directive comments like <!--@ if {cond} -->"""
        if directive.startswith("if "):
            self.handle_directive_if(directive[3:].strip())
        elif directive.startswith("elif "):
            self.handle_directive_elif(directive[5:].strip())
        elif directive == "else":
            self.handle_directive_else()
        elif directive.startswith("match "):
            self.handle_directive_match(directive[6:].strip())
        elif directive.startswith("case "):
            self.handle_directive_case(directive[5:].strip())
        elif directive == "end":
            self.handle_directive_end()
        else:
            raise ValueError(f"Unknown directive: {directive}")

    def handle_directive_if(self, condition_expr: str) -> None:
        """Handle <!--@ if {condition} -->"""
        condition_t = self.extract_template(condition_expr)

        # Template should have exactly one interpolation and two strings (before and after)
        if len(condition_t.strings) != 2 or len(condition_t.interpolations) != 1:
            raise ValueError(
                f"if directive expects a single interpolation, got: {condition_expr}"
            )

        condition_index = condition_t.interpolations[0].value

        # Create a new conditional with the first branch
        open_conditional = OpenTConditional()
        open_conditional.branches.append(
            TConditionalBranch(condition_index=condition_index, children=tuple())
        )
        self.tstate.stack.append(open_conditional)

    def handle_directive_elif(self, condition_expr: str) -> None:
        """Handle <!--@ elif {condition} -->"""
        if not self.tstate.stack or not isinstance(
            self.tstate.stack[-1], OpenTConditional
        ):
            raise ValueError("elif directive without matching if")

        open_conditional = self.tstate.stack[-1]
        # Check if we already have an else branch (condition_index=None)
        if (
            open_conditional.branches
            and open_conditional.branches[-1].condition_index is None
        ):
            raise ValueError("elif directive cannot come after else")

        condition_t = self.extract_template(condition_expr)

        if len(condition_t.strings) != 2 or len(condition_t.interpolations) != 1:
            raise ValueError(
                f"elif directive expects a single interpolation, got: {condition_expr}"
            )

        condition_index = condition_t.interpolations[0].value

        # Finalize current branch
        open_conditional.branches[-1] = TConditionalBranch(
            condition_index=open_conditional.branches[-1].condition_index,
            children=tuple(open_conditional.current_branch_children),
        )
        # Start new elif branch
        open_conditional.current_branch_children = []
        open_conditional.branches.append(
            TConditionalBranch(condition_index=condition_index, children=tuple())
        )

    def handle_directive_else(self) -> None:
        """Handle <!--@ else -->"""
        if not self.tstate.stack or not isinstance(
            self.tstate.stack[-1], OpenTConditional
        ):
            raise ValueError("else directive without matching if")

        open_conditional = self.tstate.stack[-1]
        # Check if we already have an else branch
        if (
            open_conditional.branches
            and open_conditional.branches[-1].condition_index is None
        ):
            raise ValueError("multiple else clauses not allowed")

        # Finalize current branch
        open_conditional.branches[-1] = TConditionalBranch(
            condition_index=open_conditional.branches[-1].condition_index,
            children=tuple(open_conditional.current_branch_children),
        )
        # Start else branch (condition_index=None)
        open_conditional.current_branch_children = []
        open_conditional.branches.append(
            TConditionalBranch(condition_index=None, children=tuple())
        )

    def handle_directive_match(self, subject_expr: str) -> None:
        """Handle <!--@ match {subject} -->"""
        subject_t = self.extract_template(subject_expr)

        if len(subject_t.strings) != 2 or len(subject_t.interpolations) != 1:
            raise ValueError(
                f"match directive expects a single interpolation, got: {subject_expr}"
            )

        subject_index = subject_t.interpolations[0].value

        open_match = OpenTMatch(subject_index=subject_index)
        self.tstate.stack.append(open_match)

    def handle_directive_case(self, pattern_expr: str) -> None:
        """Handle <!--@ case {pattern} -->"""
        if not self.tstate.stack or not isinstance(self.tstate.stack[-1], OpenTMatch):
            raise ValueError("case directive without matching match")

        pattern_t = self.extract_template(pattern_expr)

        if len(pattern_t.strings) != 2 or len(pattern_t.interpolations) != 1:
            raise ValueError(
                f"case directive expects a single interpolation, got: {pattern_expr}"
            )

        pattern_index = pattern_t.interpolations[0].value

        open_match = self.tstate.stack[-1]
        # Finalize previous case if any
        if open_match.current_case_children is not None:
            open_match.cases[-1] = TCase(
                pattern_index=open_match.cases[-1].pattern_index,
                children=tuple(open_match.current_case_children),
            )
        # Start new case
        open_match.current_case_children = []
        open_match.cases.append(TCase(pattern_index=pattern_index, children=tuple()))

    def handle_directive_end(self) -> None:
        """Handle <!--@ end -->"""
        # Find the control flow directive on the stack
        control_flow_index = None
        for i in range(len(self.tstate.stack) - 1, -1, -1):
            if isinstance(self.tstate.stack[i], (OpenTConditional, OpenTMatch)):
                control_flow_index = i
                break

        if control_flow_index is None:
            raise ValueError("end directive without matching control flow directive")

        open_tag = self.tstate.stack.pop(control_flow_index)

        if isinstance(open_tag, OpenTConditional):
            # Finalize the last branch
            open_tag.branches[-1] = TConditionalBranch(
                condition_index=open_tag.branches[-1].condition_index,
                children=tuple(open_tag.current_branch_children),
            )
            # Create the TConditional node
            conditional = TConditional(branches=tuple(open_tag.branches))
            self.append_child(conditional)
        elif isinstance(open_tag, OpenTMatch):
            # Finalize the last case
            if open_tag.current_case_children is not None:
                open_tag.cases[-1] = TCase(
                    pattern_index=open_tag.cases[-1].pattern_index,
                    children=tuple(open_tag.current_case_children),
                )
            # Create the TMatch node
            match_node = TMatch(
                subject_index=open_tag.subject_index, cases=tuple(open_tag.cases)
            )
            self.append_child(match_node)

    def handle_decl(self, decl: str) -> None:
        if decl.upper().startswith("DOCTYPE"):
            doctype_content = decl[7:].strip()
            doctype = TDocumentType(doctype_content)
            self.append_child(doctype)
        # For simplicity, we ignore other declarations.
        pass

    def get_latest_text_child(self) -> TText | None:
        """Get the latest text child of the current parent or None if one does not exist."""
        parent = self.get_parent()
        # Get the appropriate children list based on parent type
        if isinstance(parent, OpenTConditional):
            children = parent.current_branch_children
        elif isinstance(parent, OpenTMatch):
            children = (
                parent.current_case_children
                if parent.current_case_children is not None
                else []
            )
        else:
            children = parent.children

        if children and isinstance(children[-1], TText):
            return children[-1]
        return None

    def get_parent(self) -> OpenTag:
        """Return the current parent node to which new children should be added."""
        return self.tstate.stack[-1] if self.tstate.stack else self.tstate.root

    def append_child(self, child: TNode) -> None:
        parent = self.get_parent()
        # Special handling for control flow directives
        if isinstance(parent, OpenTConditional):
            parent.current_branch_children.append(child)
        elif isinstance(parent, OpenTMatch):
            if parent.current_case_children is not None:
                parent.current_case_children.append(child)
            else:
                # Ignore whitespace-only text before first case
                if isinstance(child, TText):
                    # Check if it's only whitespace
                    is_whitespace_only = all(
                        isinstance(part, str) and part.strip() == ""
                        for part in child.text_t
                    )
                    if is_whitespace_only:
                        return  # Silently ignore whitespace before first case
                raise ValueError(
                    "match directive requires at least one case before content"
                )
        else:
            parent.children.append(child)

    def close(self) -> None:
        if self.tstate.stack:
            raise ValueError("Invalid HTML structure: unclosed tags remain.")
        if self.tstate and self.tstate.active_placeholders:
            raise ValueError(
                f"Some interpolations were never found: {list(self.tstate.active_placeholders.values())}"
            )
        super().close()

    def get_node(
        self,
    ) -> (
        TNode  # @TODO: Might be more consistent for this to always be a container.
    ):
        """Get the Node tree parsed from the input HTML."""
        # CONSIDER: Should we invert things and offer streaming parsing?
        assert not self.tstate.active_placeholders and not self.tstate.stack, (  # nosec B101
            "Did you forget to call close()?"
        )
        if len(self.tstate.root.children) > 1:
            # The parse structure results in multiple root elements, so we
            # return a Fragment to hold them all.
            return TFragment(children=tuple(self.tstate.root.children))
        elif len(self.tstate.root.children) == 1:
            return self.tstate.root.children[0]
        else:
            # Special case: the parse structure is empty; we treat
            # this as an empty document fragment.
            return TFragment(children=tuple())

    def feed(self, data: str) -> None:
        raise NotImplementedError("Did you mean to call feed_template()?")

    def feed_template(self, template: Template):
        assert self.tstate.template.strings == ("",), "Did you forget to call reset?"  # nosec B101
        tstate = self.tstate = TemplateState(template)
        last_strings_index = len(tstate.template.strings) - 1
        tstate.strings_index = 0
        while tstate.strings_index <= last_strings_index:
            data = template.strings[tstate.strings_index]
            # @TODO: Add tracking for this
            # Could key on (strings_index, pos_in_string).
            data = data.replace("<>", f"<{FRAGMENT_TAG}>").replace(
                "</>", f"</{FRAGMENT_TAG}>"
            )
            super().feed(data)
            if tstate.strings_index != last_strings_index:
                tstate.interpolations_index = tstate.strings_index
                placeholder = construct_placeholder(tstate.interpolations_index)
                tstate.active_placeholders[placeholder] = tstate.interpolations_index
                super().feed(placeholder)
            tstate.strings_index += 1

    def extract_template(self, text: str, format_spec: str = "") -> Template:
        text_t, found = placeholders_to_template(text, format_spec)
        for placeholder in found:
            if placeholder not in self.tstate.active_placeholders:
                raise ValueError(
                    f"Found unexpected placeholder {placeholder} for interpolation {deconstruct_placeholder(placeholder)}."
                )
            else:
                del self.tstate.active_placeholders[placeholder]
        return text_t

    def reset(self):
        super().reset()
        self.tstate = TemplateState()

    def get_container_starttag(self, node: OpenTag) -> str:
        match node:
            case OpenTElement() as element:
                return element.tag
            case OpenTFragment():
                return ""
            case OpenTComponent() as component:
                return self.get_comp_starttag(component.starttag_interpolation_index)

    def get_ip_expression(
        self, ip_index: int, fallback_prefix: str = "interpolation-at-"
    ) -> str:
        """
        When an error occurs processing a placeholder resolve an expression to use for debugging.
        """
        ip = self.tstate.interpolations[ip_index]
        return (
            ip.expression
            if ip.expression != ""
            else f"{{{fallback_prefix}-{ip_index}}}"
        )

    def get_comp_endtag(self, endtag_ip_index: int) -> str:
        return self.get_ip_expression(
            endtag_ip_index, fallback_prefix="component-endtag-at-"
        )

    def get_comp_starttag(self, starttag_ip_index: int) -> str:
        return self.get_ip_expression(
            starttag_ip_index, fallback_prefix="component-starttag-at-"
        )


@lru_cache(maxsize=0 if "pytest" in sys.modules else 512)
def _parse_html(
    cached_template: CachedTemplate,
) -> (
    TNode  # @TODO: Might be more consistent for this to always be a container.
):
    parser = TemplateParser()
    parser.feed_template(cached_template.template)
    parser.close()
    return parser.get_node()


@dataclass
class CachedTemplate:
    """Attempt to cache template just by its strings."""

    template: Template

    def __hash__(self):
        return hash(self.template.strings)

    def __eq__(self, other):
        return (
            isinstance(other, CachedTemplate)
            and self.template.strings == other.template.strings
        )


def parse_html(template: Template) -> TNode:
    """
    Parse a string, or sequence of HTML string chunks, into a Node tree.

    If a single string is provided, it is parsed as a whole. If an iterable
    of strings is provided, each string is fed to the parser in sequence.
    This is particularly useful if you want to keep specific text chunks
    separate in the resulting Node tree.
    """
    return _parse_html(CachedTemplate(template))
