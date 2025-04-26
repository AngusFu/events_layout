import { describe, test, expect } from "vitest";
import { processEvents, Event, LayoutGroup } from "./lib";

describe("Event Processing Tests", () => {
  test("Test Case 1: Normal Events", () => {
    const events1: Event[] = [
      { id: "1", start: 0, end: 1 },
      { id: "2", start: 1, end: 2 },
      { id: "3", start: 2, end: 3 },
    ];
    const layoutGroups1 = processEvents(events1);
    expect(layoutGroups1.length).toBe(3);
  });

  test("Test Case 2: Overlapping Events", () => {
    const events2: Event[] = [
      { id: "1", start: 0, end: 2 },
      { id: "2", start: 1, end: 3 },
      { id: "3", start: 2, end: 4 },
    ];
    const layoutGroups2 = processEvents(events2);
    expect(layoutGroups2.length).toBe(1);
    expect(layoutGroups2[0].start).toBe(0);
    expect(layoutGroups2[0].end).toBe(4);
    expect(layoutGroups2[0].columnCount).toBe(2);
    expect(layoutGroups2[0].items.length).toBe(3);
  });

  test("Test Case 3: Empty Events", () => {
    const events3: Event[] = [];
    const layoutGroups3 = processEvents(events3);
    expect(layoutGroups3.length).toBe(0);
  });

  test("Test Case 4: Single Event", () => {
    const events4: Event[] = [{ id: "1", start: 1, end: 2 }];
    const layoutGroups4 = processEvents(events4);
    expect(layoutGroups4.length).toBe(1);
    expect(layoutGroups4[0].start).toBe(1);
    expect(layoutGroups4[0].end).toBe(2);
    expect(layoutGroups4[0].columnCount).toBe(1);
    expect(layoutGroups4[0].items.length).toBe(1);
  });

  test("Test Case 5: Complex Overlapping Events", () => {
    const events5: Event[] = [
      { id: "1", start: 0, end: 3 }, // First group
      { id: "2", start: 1, end: 4 },
      { id: "3", start: 2, end: 5 },
      { id: "4", start: 6, end: 8 }, // Second group
      { id: "5", start: 7, end: 9 },
      { id: "6", start: 8, end: 10 },
    ];
    const layoutGroups5 = processEvents(events5);
    expect(layoutGroups5.length).toBe(2);
    expect(layoutGroups5[0].start).toBe(0);
    expect(layoutGroups5[0].end).toBe(5);
    expect(layoutGroups5[0].columnCount).toBe(3);
    expect(layoutGroups5[0].items.length).toBe(3);
    expect(layoutGroups5[1].start).toBe(6);
    expect(layoutGroups5[1].end).toBe(10);
    expect(layoutGroups5[1].columnCount).toBe(2);
    expect(layoutGroups5[1].items.length).toBe(3);
  });

  test("Test Case 6: Nested Events", () => {
    const events6: Event[] = [
      { id: "1", start: 0, end: 10 },
      { id: "2", start: 2, end: 8 },
      { id: "3", start: 3, end: 7 },
    ];
    const layoutGroups6 = processEvents(events6);
    expect(layoutGroups6.length).toBe(1);
    expect(layoutGroups6[0].columnCount).toBe(3);
    expect(layoutGroups6[0].items.length).toBe(3);
  });

  test("Test Case 7: Edge Cases (Zero Duration Events)", () => {
    const events7: Event[] = [
      { id: "1", start: 0, end: 0 },
      { id: "2", start: 1, end: 1 },
      { id: "3", start: 2, end: 2 },
      { id: "4", start: 3, end: 3 },
    ];
    const layoutGroups7 = processEvents(events7);
    expect(layoutGroups7.length).toBe(4);
    layoutGroups7.forEach((group) => {
      expect(group.columnCount).toBe(1);
      expect(group.items.length).toBe(1);
    });
  });

  test("Test Case 8: Large Number of Events", () => {
    const events8: Event[] = [];
    for (let i = 0; i < 100; i++) {
      events8.push({
        id: i.toString(),
        start: i * 0.5,
        end: i * 0.5 + 1,
      });
    }
    const layoutGroups8 = processEvents(events8);
    expect(layoutGroups8.length).toBeGreaterThan(0);
    const totalEvents = layoutGroups8.reduce(
      (sum, group) => sum + group.items.length,
      0
    );
    expect(totalEvents).toBe(100);
  });

  test("Test Case 9: Random Overlapping Events", () => {
    const events9: Event[] = [
      { id: "1", start: 0, end: 5 },
      { id: "2", start: 1, end: 3 },
      { id: "3", start: 2, end: 4 },
      { id: "4", start: 3, end: 6 },
      { id: "5", start: 4, end: 7 },
      { id: "6", start: 5, end: 8 },
    ];
    const layoutGroups9 = processEvents(events9);
    expect(layoutGroups9.length).toBe(1);
    expect(layoutGroups9[0].columnCount).toBe(3);
    expect(layoutGroups9[0].items.length).toBe(6);
    layoutGroups9[0].items.forEach((item) => {
      expect(item.top).toBeGreaterThanOrEqual(0);
      expect(item.bottom).toBeGreaterThanOrEqual(0);
      expect(item.height).toBeGreaterThan(0);
      expect(item.column).toBeLessThan(layoutGroups9[0].columnCount);
    });
  });
});
