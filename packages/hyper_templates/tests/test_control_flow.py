"""Tests for control flow directives: if/elif/else and match/case."""
import pytest
from hyper_templates._tdom.processor import html


class TestConditionals:
    """Test if/elif/else directives."""

    def test_simple_if_true(self):
        """Test basic if directive with true condition."""
        show = True
        result = html(t"""
            <div>
                <!--@ if {show} -->
                    <p>Visible</p>
                <!--@ end -->
            </div>
        """)
        assert "<p>Visible</p>" in str(result)

    def test_simple_if_false(self):
        """Test basic if directive with false condition."""
        show = False
        result = html(t"""
            <div>
                <!--@ if {show} -->
                    <p>Hidden</p>
                <!--@ end -->
            </div>
        """)
        assert "<p>Hidden</p>" not in str(result)
        # The empty conditional leaves some whitespace
        assert "<div>" in str(result) and "</div>" in str(result)

    def test_if_else_true(self):
        """Test if/else with true condition."""
        is_admin = True
        result = html(t"""
            <nav>
                <!--@ if {is_admin} -->
                    <a href="/admin">Admin</a>
                <!--@ else -->
                    <a href="/account">Account</a>
                <!--@ end -->
            </nav>
        """)
        assert '<a href="/admin">Admin</a>' in str(result)
        assert '<a href="/account">Account</a>' not in str(result)

    def test_if_else_false(self):
        """Test if/else with false condition."""
        is_admin = False
        result = html(t"""
            <nav>
                <!--@ if {is_admin} -->
                    <a href="/admin">Admin</a>
                <!--@ else -->
                    <a href="/account">Account</a>
                <!--@ end -->
            </nav>
        """)
        assert '<a href="/admin">Admin</a>' not in str(result)
        assert '<a href="/account">Account</a>' in str(result)

    def test_if_elif_else_first_true(self):
        """Test if/elif/else with first condition true."""
        role = "admin"
        result = html(t"""
            <nav>
                <!--@ if {role == "admin"} -->
                    <a href="/admin">Admin Panel</a>
                <!--@ elif {role == "moderator"} -->
                    <a href="/mod">Mod Panel</a>
                <!--@ else -->
                    <a href="/account">Account</a>
                <!--@ end -->
            </nav>
        """)
        assert '<a href="/admin">Admin Panel</a>' in str(result)
        assert '<a href="/mod">Mod Panel</a>' not in str(result)
        assert '<a href="/account">Account</a>' not in str(result)

    def test_if_elif_else_elif_true(self):
        """Test if/elif/else with elif condition true."""
        role = "moderator"
        result = html(t"""
            <nav>
                <!--@ if {role == "admin"} -->
                    <a href="/admin">Admin Panel</a>
                <!--@ elif {role == "moderator"} -->
                    <a href="/mod">Mod Panel</a>
                <!--@ else -->
                    <a href="/account">Account</a>
                <!--@ end -->
            </nav>
        """)
        assert '<a href="/admin">Admin Panel</a>' not in str(result)
        assert '<a href="/mod">Mod Panel</a>' in str(result)
        assert '<a href="/account">Account</a>' not in str(result)

    def test_if_elif_else_all_false(self):
        """Test if/elif/else with all conditions false."""
        role = "user"
        result = html(t"""
            <nav>
                <!--@ if {role == "admin"} -->
                    <a href="/admin">Admin Panel</a>
                <!--@ elif {role == "moderator"} -->
                    <a href="/mod">Mod Panel</a>
                <!--@ else -->
                    <a href="/account">Account</a>
                <!--@ end -->
            </nav>
        """)
        assert '<a href="/admin">Admin Panel</a>' not in str(result)
        assert '<a href="/mod">Mod Panel</a>' not in str(result)
        assert '<a href="/account">Account</a>' in str(result)

    def test_multiple_elif(self):
        """Test multiple elif branches."""
        score = 75
        result = html(t"""
            <div>
                <!--@ if {score >= 90} -->
                    <span>A</span>
                <!--@ elif {score >= 80} -->
                    <span>B</span>
                <!--@ elif {score >= 70} -->
                    <span>C</span>
                <!--@ else -->
                    <span>F</span>
                <!--@ end -->
            </div>
        """)
        assert "<span>C</span>" in str(result)

    def test_nested_conditionals(self):
        """Test nested if statements."""
        is_logged_in = True
        is_premium = True
        result = html(t"""
            <div>
                <!--@ if {is_logged_in} -->
                    <p>Welcome!</p>
                    <!--@ if {is_premium} -->
                        <p>Premium Member</p>
                    <!--@ end -->
                <!--@ end -->
            </div>
        """)
        assert "<p>Welcome!</p>" in str(result)
        assert "<p>Premium Member</p>" in str(result)


class TestMatch:
    """Test match/case directives."""

    def test_simple_match_first_case(self):
        """Test basic match with first case matching."""
        status = "loading"
        result = html(t"""
            <div>
                <!--@ match {status} -->
                    <!--@ case {"loading"} -->
                        <p>Loading...</p>
                    <!--@ case {"error"} -->
                        <p>Error!</p>
                    <!--@ case {"success"} -->
                        <p>Done!</p>
                <!--@ end -->
            </div>
        """)
        assert "<p>Loading...</p>" in str(result)
        assert "<p>Error!</p>" not in str(result)
        assert "<p>Done!</p>" not in str(result)

    def test_simple_match_middle_case(self):
        """Test basic match with middle case matching."""
        status = "error"
        result = html(t"""
            <div>
                <!--@ match {status} -->
                    <!--@ case {"loading"} -->
                        <p>Loading...</p>
                    <!--@ case {"error"} -->
                        <p>Error!</p>
                    <!--@ case {"success"} -->
                        <p>Done!</p>
                <!--@ end -->
            </div>
        """)
        assert "<p>Loading...</p>" not in str(result)
        assert "<p>Error!</p>" in str(result)
        assert "<p>Done!</p>" not in str(result)

    def test_match_with_wildcard(self):
        """Test match with wildcard pattern."""
        # Create a wildcard sentinel
        _ = object()  # Wildcard pattern
        status = "unknown"
        result = html(t"""
            <div>
                <!--@ match {status} -->
                    <!--@ case {"loading"} -->
                        <p>Loading...</p>
                    <!--@ case {"error"} -->
                        <p>Error!</p>
                    <!--@ case {_} -->
                        <p>Unknown status</p>
                <!--@ end -->
            </div>
        """)
        assert "<p>Loading...</p>" not in str(result)
        assert "<p>Error!</p>" not in str(result)
        assert "<p>Unknown status</p>" in str(result)

    def test_match_numeric_patterns(self):
        """Test match with numeric values."""
        code = 404
        result = html(t"""
            <div>
                <!--@ match {code} -->
                    <!--@ case {200} -->
                        <p>OK</p>
                    <!--@ case {404} -->
                        <p>Not Found</p>
                    <!--@ case {500} -->
                        <p>Server Error</p>
                <!--@ end -->
            </div>
        """)
        assert "<p>Not Found</p>" in str(result)

    def test_match_no_match(self):
        """Test match where nothing matches (no wildcard)."""
        status = "pending"
        result = html(t"""
            <div>
                <!--@ match {status} -->
                    <!--@ case {"loading"} -->
                        <p>Loading...</p>
                    <!--@ case {"error"} -->
                        <p>Error!</p>
                <!--@ end -->
            </div>
        """)
        # Should render nothing for unmatched case
        assert "<p>Loading...</p>" not in str(result)
        assert "<p>Error!</p>" not in str(result)


class TestServerSideComments:
    """Test server-side comment stripping."""

    def test_server_side_comment_removed(self):
        """Server-side comments should not appear in output."""
        result = html(t"""
            <div>
                <!--# This is a server-side comment -->
                <p>Content</p>
            </div>
        """)
        assert "This is a server-side comment" not in str(result)
        assert "<p>Content</p>" in str(result)

    def test_regular_comment_preserved(self):
        """Regular HTML comments should be preserved."""
        result = html(t"""
            <div>
                <!-- This is a regular comment -->
                <p>Content</p>
            </div>
        """)
        assert "<!-- This is a regular comment -->" in str(result)
        assert "<p>Content</p>" in str(result)


class TestFalsyValues:
    """Test behavior with various falsy values."""

    def test_zero_is_falsy(self):
        """0 should be treated as falsy."""
        count = 0
        result = html(t"""
            <!--@ if {count} -->
                <p>Has items</p>
            <!--@ else -->
                <p>Empty</p>
            <!--@ end -->
        """)
        assert "Empty" in str(result)
        assert "Has items" not in str(result)

    def test_empty_string_is_falsy(self):
        """Empty string should be falsy."""
        name = ""
        result = html(t"""
            <!--@ if {name} -->
                <p>Hello {name}</p>
            <!--@ else -->
                <p>Anonymous</p>
            <!--@ end -->
        """)
        assert "Anonymous" in str(result)

    def test_none_is_falsy(self):
        """None should be falsy."""
        value = None
        result = html(t"""
            <!--@ if {value} -->
                <p>Present</p>
            <!--@ else -->
                <p>Absent</p>
            <!--@ end -->
        """)
        assert "Absent" in str(result)

    def test_match_with_zero(self):
        """Match should distinguish 0 from other falsy values."""
        code = 0
        result = html(t"""
            <!--@ match {code} -->
                <!--@ case {0} -->
                    <p>Zero</p>
                <!--@ case {False} -->
                    <p>False</p>
            <!--@ end -->
        """)
        assert "Zero" in str(result)
        assert "False" not in str(result)

    def test_match_with_none(self):
        """Match should match None exactly."""
        value = None
        result = html(t"""
            <!--@ match {value} -->
                <!--@ case {None} -->
                    <p>Is None</p>
                <!--@ case {False} -->
                    <p>Is False</p>
            <!--@ end -->
        """)
        assert "Is None" in str(result)
        assert "Is False" not in str(result)


class TestErrorCases:
    """Test error handling for malformed directives."""

    def test_elif_without_if(self):
        """elif without if should raise error."""
        with pytest.raises(ValueError, match="elif directive without matching if"):
            html(t"""
                <div>
                    <!--@ elif {True} -->
                        <p>Oops</p>
                    <!--@ end -->
                </div>
            """)

    def test_else_without_if(self):
        """else without if should raise error."""
        with pytest.raises(ValueError, match="else directive without matching if"):
            html(t"""
                <div>
                    <!--@ else -->
                        <p>Oops</p>
                    <!--@ end -->
                </div>
            """)

    def test_case_without_match(self):
        """case without match should raise error."""
        with pytest.raises(ValueError, match="case directive without matching match"):
            html(t"""
                <div>
                    <!--@ case {"value"} -->
                        <p>Oops</p>
                    <!--@ end -->
                </div>
            """)

    def test_end_without_start(self):
        """end without control flow should raise error."""
        with pytest.raises(ValueError, match="end directive without matching control flow"):
            html(t"""
                <div>
                    <!--@ end -->
                </div>
            """)

    def test_unclosed_if(self):
        """Unclosed if should raise error."""
        with pytest.raises(ValueError, match="unclosed tags remain"):
            html(t"""
                <div>
                    <!--@ if {True} -->
                        <p>Never closed</p>
                </div>
            """)

    def test_elif_after_else(self):
        """elif after else should raise error."""
        with pytest.raises(ValueError, match="elif directive cannot come after else"):
            html(t"""
                <!--@ if {True} -->
                    <p>True</p>
                <!--@ else -->
                    <p>False</p>
                <!--@ elif {False} -->
                    <p>Should error</p>
                <!--@ end -->
            """)

    def test_multiple_else_clauses(self):
        """Multiple else clauses should raise error."""
        with pytest.raises(ValueError, match="multiple else clauses not allowed"):
            html(t"""
                <!--@ if {True} -->
                    <p>True</p>
                <!--@ else -->
                    <p>First else</p>
                <!--@ else -->
                    <p>Second else</p>
                <!--@ end -->
            """)
