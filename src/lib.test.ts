import { describe, test, expect } from 'vitest'
import { processEvents, Event, LayoutGroup } from './lib'

describe('事件处理测试', () => {
  test('测试用例 1: 正常事件', () => {
    const events1: Event[] = [
      { id: '1', start: 0, end: 1 },
      { id: '2', start: 1, end: 2 },
      { id: '3', start: 2, end: 3 },
    ]
    const layoutGroups1 = processEvents(events1)
    expect(layoutGroups1.length).toBe(3)
  })

  test('测试用例 2: 重叠事件', () => {
    const events2: Event[] = [
      { id: '1', start: 0, end: 2 },
      { id: '2', start: 1, end: 3 },
      { id: '3', start: 2, end: 4 },
    ]
    const layoutGroups2 = processEvents(events2)
    expect(layoutGroups2.length).toBe(1)
    expect(layoutGroups2[0].start).toBe(0)
    expect(layoutGroups2[0].end).toBe(4)
    expect(layoutGroups2[0].columnCount).toBe(2)
    expect(layoutGroups2[0].items.length).toBe(3)
  })

  test('测试用例 3: 空事件', () => {
    const events3: Event[] = []
    const layoutGroups3 = processEvents(events3)
    expect(layoutGroups3.length).toBe(0)
  })

  test('测试用例 4: 单个事件', () => {
    const events4: Event[] = [{ id: '1', start: 1, end: 2 }]
    const layoutGroups4 = processEvents(events4)
    expect(layoutGroups4.length).toBe(1)
    expect(layoutGroups4[0].start).toBe(1)
    expect(layoutGroups4[0].end).toBe(2)
    expect(layoutGroups4[0].columnCount).toBe(1)
    expect(layoutGroups4[0].items.length).toBe(1)
  })

  test('测试用例 5: 复杂重叠事件', () => {
    const events5: Event[] = [
      { id: '1', start: 0, end: 3 },  // 第一组
      { id: '2', start: 1, end: 4 },
      { id: '3', start: 2, end: 5 },
      { id: '4', start: 6, end: 8 },  // 第二组
      { id: '5', start: 7, end: 9 },
      { id: '6', start: 8, end: 10 },
    ]
    const layoutGroups5 = processEvents(events5)
    expect(layoutGroups5.length).toBe(2)
    expect(layoutGroups5[0].start).toBe(0)
    expect(layoutGroups5[0].end).toBe(5)
    expect(layoutGroups5[0].columnCount).toBe(3)
    expect(layoutGroups5[0].items.length).toBe(3)
    expect(layoutGroups5[1].start).toBe(6)
    expect(layoutGroups5[1].end).toBe(10)
    expect(layoutGroups5[1].columnCount).toBe(2)
    expect(layoutGroups5[1].items.length).toBe(3)
  })

  test('测试用例 6: 嵌套事件', () => {
    const events6: Event[] = [
      { id: '1', start: 0, end: 10 },
      { id: '2', start: 2, end: 8 },
      { id: '3', start: 3, end: 7 },
    ]
    const layoutGroups6 = processEvents(events6)
    expect(layoutGroups6.length).toBe(1)
    expect(layoutGroups6[0].columnCount).toBe(3)
    expect(layoutGroups6[0].items.length).toBe(3)
  })

  test('测试用例 7: 边界情况（零时长事件）', () => {
    const events7: Event[] = [
      { id: '1', start: 0, end: 0 },
      { id: '2', start: 1, end: 1 },
      { id: '3', start: 2, end: 2 },
      { id: '4', start: 3, end: 3 },
    ]
    const layoutGroups7 = processEvents(events7)
    expect(layoutGroups7.length).toBe(4)
    layoutGroups7.forEach(group => {
      expect(group.columnCount).toBe(1)
      expect(group.items.length).toBe(1)
    })
  })

  test('测试用例 8: 大量事件', () => {
    const events8: Event[] = []
    for (let i = 0; i < 100; i++) {
      events8.push({
        id: i.toString(),
        start: i * 0.5,
        end: i * 0.5 + 1,
      })
    }
    const layoutGroups8 = processEvents(events8)
    expect(layoutGroups8.length).toBeGreaterThan(0)
    const totalEvents = layoutGroups8.reduce((sum, group) => sum + group.items.length, 0)
    expect(totalEvents).toBe(100)
  })

  test('测试用例 9: 随机重叠事件', () => {
    const events9: Event[] = [
      { id: '1', start: 0, end: 5 },
      { id: '2', start: 1, end: 3 },
      { id: '3', start: 2, end: 4 },
      { id: '4', start: 3, end: 6 },
      { id: '5', start: 4, end: 7 },
      { id: '6', start: 5, end: 8 },
    ]
    const layoutGroups9 = processEvents(events9)
    expect(layoutGroups9.length).toBe(1)
    expect(layoutGroups9[0].columnCount).toBe(3)
    expect(layoutGroups9[0].items.length).toBe(6)
    layoutGroups9[0].items.forEach(item => {
      expect(item.top).toBeGreaterThanOrEqual(0)
      expect(item.bottom).toBeGreaterThanOrEqual(0)
      expect(item.height).toBeGreaterThan(0)
      expect(item.column).toBeLessThan(layoutGroups9[0].columnCount)
    })
  })
}) 