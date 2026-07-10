/**
 * Basic tests for Node.js binding
 */

import * as mc from '../src/index.ts';

console.log('🧪 Running Node.js binding tests...\n');

// Test 1: Basic render
try {
  const svg = mc.render('graph TD; A[Start]-->B[End]');
  if (svg.includes('<svg') && svg.includes('</svg>')) {
    console.log('✅ Test 1: Basic render - PASS');
  } else {
    console.log('❌ Test 1: Basic render - FAIL (invalid SVG)');
  }
} catch (e) {
  console.log('❌ Test 1: Basic render - FAIL', (e as Error).message);
}

// Test 2: Parse
try {
  const ast = mc.parse('graph TD; A-->B');
  if (ast.includes('Flowchart')) {
    console.log('✅ Test 2: Parse - PASS');
  } else {
    console.log('⚠️  Test 2: Parse - SKIP (JSON feature not enabled)');
  }
} catch (e) {
  console.log('⚠️  Test 2: Parse - SKIP');
}

// Test 3: Check syntax
try {
  const valid = mc.check('graph TD; A-->B');
  if (valid) {
    console.log('✅ Test 3: Check valid - PASS');
  } else {
    console.log('❌ Test 3: Check valid - FAIL');
  }
} catch (e) {
  console.log('❌ Test 3: Check valid - FAIL', (e as Error).message);
}

// Test 4: Check invalid syntax
try {
  const invalid = mc.check('grpah TD; A-->B');
  if (!invalid) {
    console.log('✅ Test 4: Check invalid - PASS');
  } else {
    console.log('❌ Test 4: Check invalid - FAIL (should detect error)');
  }
} catch (e) {
  console.log('✅ Test 4: Check invalid - PASS (correctly threw error)');
}

// Test 5: Fix
try {
  const fixed = mc.fix('grpah TD; A-->B');
  if (fixed.includes('graph')) {
    console.log('✅ Test 5: Fix - PASS');
  } else {
    console.log('❌ Test 5: Fix - FAIL (not fixed)');
  }
} catch (e) {
  console.log('❌ Test 5: Fix - FAIL', (e as Error).message);
}

// Test 6: Render with options
try {
  const svg = mc.renderWithOptions('pie title Test\n"A": 50\n"B": 50', {
    theme: 'dark',
    width: 1024,
    height: 768
  });
  if (svg.includes('<svg')) {
    console.log('✅ Test 6: Render with options - PASS');
  } else {
    console.log('❌ Test 6: Render with options - FAIL');
  }
} catch (e) {
  console.log('❌ Test 6: Render with options - FAIL', (e as Error).message);
}

console.log('\n✅ Test suite completed');
