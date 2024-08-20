interface BaseRange {
  start: number
  end: number
}

interface Event extends BaseRange {
  id: string
}

interface EventLayout {
  /** by percent */
  top: number
  /** by percent */
  bottom: number
  /** by percent */
  height: number
  /** integer */
  column: number

  event: Event
}
export interface LayoutGroup extends BaseRange {
  items: EventLayout[]
  columnCount: number
}

class Group {
  start: number

  end: number

  columnsOfEvents: Event[][]

  unusedRects: { start: number; end: number; columnIndex: number }[]

  constructor() {
    this.start = Infinity
    this.end = -Infinity
    this.columnsOfEvents = []
    this.unusedRects = []
  }

  add(event: Event) {
    const placed = this.#insert(event, this.unusedRects)

    if (!placed) {
      // 尝试重新解决空白区域 让列尽可能少
      const start = Math.min(this.start, event.start)
      const end = Math.max(this.end, event.end)
      const emptyRects = this.#getEmptyRects(start, end)

      if (emptyRects.length > 0 && this.#insert(event, emptyRects)) {
        this.unusedRects = emptyRects
      } else {
        this.columnsOfEvents.push([event])
      }
    }
    this.start = Math.min(this.start, event.start)
    this.end = Math.max(this.end, event.end)
  }

  #insert(event: Event, emptyRects: { start: number; end: number; columnIndex: number }[]) {
    let placed = false
    for (let i = 0; i < emptyRects.length; i++) {
      const rect = emptyRects[i]
      if (event.start >= rect.start && event.end <= rect.end) {
        this.columnsOfEvents[rect.columnIndex].push(event)
        placed = true
        if (event.start === rect.start && event.end === rect.end) {
          emptyRects.splice(i, 1)
        } else if (event.start === rect.start) {
          emptyRects[i].start = event.end
        } else if (event.end === rect.end) {
          emptyRects[i].end = event.start
        } else {
          emptyRects.splice(
            i,
            1,
            { start: rect.start, end: event.start, columnIndex: rect.columnIndex },
            { start: event.end, end: rect.end, columnIndex: rect.columnIndex },
          )
        }
        break
      }
    }
    return placed
  }

  #getEmptyRects(start: number, end: number) {
    const emptyRects = [] as Array<{ start: number; end: number; columnIndex: number }>
    this.columnsOfEvents.forEach((column, columnIndex) => {
      let lastEnd = start
      column.forEach(event => {
        if (event.start > lastEnd) {
          emptyRects.push({ start: lastEnd, end: event.start, columnIndex })
        }
        lastEnd = Math.max(lastEnd, event.end)
      })
      if (lastEnd < end) {
        emptyRects.push({ start: lastEnd, end, columnIndex })
      }
    })
    return emptyRects
  }

  calcLayout(): LayoutGroup {
    const columnCount = this.columnsOfEvents.length
    const totalHeight = this.end - this.start

    const items = this.columnsOfEvents.reduce((acc, events, i) => {
      acc.push(
        ...events.map(el => ({
          event: el,

          column: i,
          // by percent
          top: (el.start - this.start) / totalHeight,
          // by percent
          height: (el.end - el.start) / totalHeight,
          // by percent
          bottom: (this.end - el.end) / totalHeight,
        })),
      )
      return acc
    }, [] as EventLayout[])

    return {
      items,
      columnCount,
      start: this.start,
      end: this.end,
    }
  }
}

function isOverlap(r1: BaseRange, r2: BaseRange) {
  return r1.start < r2.end && r2.start < r1.end
}

function mergeEvents(events: Event[]) {
  if (events.length === 0) return []

  const groups = [] as Group[]
  let currentGroup = new Group()
  currentGroup.add(events[0])

  for (let i = 1; i < events.length; i++) {
    const currentEvent = events[i]
    if (isOverlap(currentGroup, currentEvent)) {
      currentGroup.add(currentEvent)
    } else {
      groups.push(currentGroup)
      currentGroup = new Group()
      currentGroup.add(currentEvent)
    }
  }

  groups.push(currentGroup)

  return groups
}

export function processEvents(events: Event[]): LayoutGroup[] {
  return mergeEvents([...events].sort((a, b) => a.start - b.start || b.end - a.end)).map(group => group.calcLayout())
}
