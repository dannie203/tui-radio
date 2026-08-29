import { test, describe } from 'node:test';
import assert from 'node:assert/strict';
import { renderCarouselTabs } from '../src/ui/layout.js';
import { GENRE_FILTERS } from '../src/state/store.js';

describe('Carousel Tab Bar Engine', () => {
  test('renders first tabs when first item ALL is selected', () => {
    const result = renderCarouselTabs(GENRE_FILTERS, 'ALL', 40);
    assert.ok(result.includes('▶ [ ALL ]'));
    assert.ok(result.includes('▶')); // Right arrow indicating more items
    assert.ok(!result.startsWith('{bold}{#00e5ff-fg}◀')); // No left arrow at the beginning
  });

  test('slides window to include and center CLASSICAL with both left and right arrows', () => {
    const result = renderCarouselTabs(GENRE_FILTERS, 'CLASSICAL', 40);
    assert.ok(result.includes('▶ [ CLASSICAL ]'));
    assert.ok(result.includes('◀')); // Left arrow
    assert.ok(result.includes('▶')); // Right arrow
  });

  test('slides window to the end when GLOBAL TOP is selected', () => {
    const result = renderCarouselTabs(GENRE_FILTERS, 'GLOBAL TOP', 40);
    assert.ok(result.includes('▶ [ GLOBAL TOP ]'));
    assert.ok(result.includes('◀')); // Left arrow
  });

  test('handles empty or single item arrays gracefully', () => {
    assert.equal(renderCarouselTabs([], 'ALL', 40), '');
    const single = renderCarouselTabs(['LO-FI'], 'LO-FI', 40);
    assert.ok(single.includes('▶ [ LO-FI ]'));
    assert.ok(!single.startsWith('{bold}{#00e5ff-fg}◀'));
    assert.ok(!single.endsWith('{bold}{#00e5ff-fg}▶{/#00e5ff-fg}{/bold}'));
  });
});
