// Quick test for the WASM package
// Run: node --experimental-vm-modules test/test.mjs

import init, { render, parse, check } from "../pkg/mermaid_cli.js";

async function main() {
  await init();

  // Test render
  console.log("Testing render...");
  const svg = render("graph TD; A-->B");
  console.assert(svg.includes("<svg"), "SVG should contain <svg>");
  console.assert(svg.includes("</svg>"), "SVG should contain </svg>");
  console.log("  ✓ render works");

  // Test parse
  console.log("Testing parse...");
  const ast = parse("graph TD; A-->B");
  const obj = JSON.parse(ast);
  console.assert(obj.diagram_type === "Flowchart", "Should be flowchart");
  console.log("  ✓ parse works");

  // Test check
  console.log("Testing check...");
  const valid = JSON.parse(check("graph TD; A-->B"));
  console.assert(valid.valid === true, "Valid code should pass");
  console.log("  ✓ check works");

  // Test check with error
  const invalid = JSON.parse(check("invalid syntax"));
  console.assert(invalid.valid === false, "Invalid code should fail");
  console.log("  ✓ error detection works");

  // Test with sequence diagram
  console.log("Testing sequence diagram...");
  const seqSvg = render("sequenceDiagram\nA->B: Hello");
  console.assert(seqSvg.includes("A"), "Should contain participant");
  console.log("  ✓ sequence diagram works");

  console.log("\n✓ All tests passed!");
}

main().catch(console.error);
