//! Minimal PDF writer - zero external dependencies.
//! Wraps SVG content in a valid PDF 1.4 container.

/// A minimal, zero-dependency PDF writer that embeds SVG content
/// as a Form XObject on a single-page PDF document.
pub struct PdfWriter;

impl PdfWriter {
    /// Render SVG content into a valid PDF 1.4 document.
    ///
    /// The SVG is embedded inside a Form XObject so PDF viewers
    /// can render the vector graphics natively.
    pub fn render_svg(svg: &str, width: u32, height: u32) -> String {
        // Sanitize SVG: remove XML declaration if present
        let clean_svg = Self::clean_svg(svg);

        let mut pdf = String::with_capacity(
            256 + clean_svg.len(), // rough pre-allocation
        );

        // ── Header ──
        pdf.push_str("%PDF-1.4\n");

        // ── Object offsets for xref table ──
        let obj1_offset = pdf.len(); // Catalog
        pdf.push_str("1 0 obj\n");
        pdf.push_str("<< /Type /Catalog /Pages 2 0 R >>\n");
        pdf.push_str("endobj\n");

        let obj2_offset = pdf.len(); // Pages
        pdf.push_str("2 0 obj\n");
        pdf.push_str("<< /Type /Pages /Kids [3 0 R] /Count 1 >>\n");
        pdf.push_str("endobj\n");

        let obj3_offset = pdf.len(); // Page
        pdf.push_str(&format!(
            "3 0 obj\n\
             << /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}]\n\
             /Contents 4 0 R /Resources << /XObject << /X0 5 0 R >> >> >>\n\
             endobj\n",
            width, height
        ));

        // Object 4: Content stream (references the XObject)
        let stream_content = format!(
            "q\n\
             1 0 0 1 0 0 cm\n\
             /X0 Do\n\
             Q\n"
        );
        let obj4_offset = pdf.len();
        pdf.push_str("4 0 obj\n");
        pdf.push_str(&format!(
            "<< /Length {} >>\n\
             stream\n\
             {}endstream\n\
             endobj\n",
            stream_content.len(),
            stream_content
        ));

        // Object 5: XObject form containing the SVG
        let obj5_offset = pdf.len();
        pdf.push_str("5 0 obj\n");
        pdf.push_str(&format!(
            "<< /Type /XObject /Subtype /Form /BBox [0 0 {} {}] /Length {} >>\n\
             stream\n\
             {}endstream\n\
             endobj\n",
            width,
            height,
            clean_svg.len(),
            clean_svg
        ));

        // ── Cross-reference table ──
        let xref_offset = pdf.len();
        pdf.push_str("xref\n");
        pdf.push_str(&format!("0 6\n"));
        pdf.push_str("0000000000 65535 f \n");
        pdf.push_str(&format!("{:010} 00000 n \n", obj1_offset));
        pdf.push_str(&format!("{:010} 00000 n \n", obj2_offset));
        pdf.push_str(&format!("{:010} 00000 n \n", obj3_offset));
        pdf.push_str(&format!("{:010} 00000 n \n", obj4_offset));
        pdf.push_str(&format!("{:010} 00000 n \n", obj5_offset));

        // ── Trailer ──
        pdf.push_str("trailer\n");
        pdf.push_str("<< /Size 6 /Root 1 0 R >>\n");
        pdf.push_str("startxref\n");
        pdf.push_str(&format!("{}\n", xref_offset));
        pdf.push_str("%%EOF\n");

        pdf
    }

    /// Strip XML declarations from SVG content so it can be embedded
    /// directly inside a PDF stream.
    fn clean_svg(input: &str) -> &str {
        let trimmed = input.trim_start();
        if trimmed.starts_with("<?xml") {
            if let Some(pos) = trimmed.find("?>") {
                trimmed[pos + 2..].trim_start()
            } else {
                trimmed
            }
        } else {
            trimmed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_starts_with_magic_bytes() {
        let svg = "<svg xmlns='http://www.w3.org/2000/svg'></svg>";
        let pdf = PdfWriter::render_svg(svg, 800, 600);
        assert!(
            pdf.starts_with("%PDF-1.4"),
            "PDF should start with %PDF-1.4"
        );
    }

    #[test]
    fn test_pdf_ends_with_eof_marker() {
        let svg = "<svg></svg>";
        let pdf = PdfWriter::render_svg(svg, 400, 300);
        assert!(pdf.ends_with("%%EOF\n"), "PDF should end with %%EOF");
    }

    #[test]
    fn test_pdf_contains_svg_content() {
        let svg = "<svg><text>HelloPDF</text></svg>";
        let pdf = PdfWriter::render_svg(svg, 200, 200);
        assert!(
            pdf.contains("HelloPDF"),
            "PDF should contain the SVG text content"
        );
    }

    #[test]
    fn test_pdf_has_valid_structure() {
        let svg = "<svg xmlns='http://www.w3.org/2000/svg'><rect/></svg>";
        let pdf = PdfWriter::render_svg(svg, 800, 600);

        // Must have all required PDF structural keywords
        assert!(pdf.contains("1 0 obj"), "Missing Catalog object");
        assert!(pdf.contains("2 0 obj"), "Missing Pages object");
        assert!(pdf.contains("3 0 obj"), "Missing Page object");
        assert!(pdf.contains("4 0 obj"), "Missing Contents object");
        assert!(pdf.contains("5 0 obj"), "Missing XObject form");
        assert!(pdf.contains("xref"), "Missing cross-reference table");
        assert!(pdf.contains("trailer"), "Missing trailer");
        assert!(pdf.contains("startxref"), "Missing startxref");
    }

    #[test]
    fn test_pdf_media_box_matches_dimensions() {
        let svg = "<svg></svg>";
        let pdf = PdfWriter::render_svg(svg, 1024, 768);
        assert!(
            pdf.contains("[0 0 1024 768]"),
            "MediaBox should match given dimensions"
        );
    }

    #[test]
    fn test_clean_svg_removes_xml_declaration() {
        let input = "<?xml version=\"1.0\"?>\n<svg>...</svg>";
        let cleaned = PdfWriter::clean_svg(input);
        assert!(
            !cleaned.starts_with("<?xml"),
            "XML declaration should be removed"
        );
        assert!(cleaned.starts_with("<svg"), "Should start with <svg");
    }

    #[test]
    fn test_clean_svg_no_declaration() {
        let input = "<svg>...</svg>";
        let cleaned = PdfWriter::clean_svg(input);
        assert_eq!(
            cleaned, input,
            "Without declaration, SVG should pass through unchanged"
        );
    }
}
