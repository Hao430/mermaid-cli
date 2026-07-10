"""
Test suite for mermaid-cli Python binding
"""

import unittest
import mermaid_cli


class TestMermaidCli(unittest.TestCase):
    """Test mermaid_cli functions."""

    def test_render(self):
        """Test basic SVG rendering."""
        svg = mermaid_cli.render("graph TD; A[Start]-->B[End]")
        self.assertIn("<svg", svg)
        self.assertIn("</svg>", svg)

    def test_check_valid(self):
        """Test syntax checking for valid code."""
        result = mermaid_cli.check("graph TD; A-->B")
        self.assertTrue(result)

    def test_check_invalid(self):
        """Test syntax checking for invalid code."""
        result = mermaid_cli.check("grpah TD; A-->B")
        self.assertFalse(result)

    def test_fix(self):
        """Test auto-fix functionality."""
        fixed = mermaid_cli.fix("grpah TD; A-->B")
        self.assertIn("graph", fixed)

    def test_render_with_options(self):
        """Test rendering with custom options."""
        svg = mermaid_cli.render_with_options(
            "pie title Test\n\"A\": 50\n\"B\": 50",
            theme="dark",
            width=1024,
            height=768,
        )
        self.assertIn("<svg", svg)

    def test_parse(self):
        """Test AST parsing."""
        try:
            ast = mermaid_cli.parse("graph TD; A-->B")
            self.assertIsInstance(ast, dict)
        except mermaid_cli.MermaidError:
            # JSON feature might not be enabled
            pass


if __name__ == "__main__":
    unittest.main()
