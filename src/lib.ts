interface BaseRange {
  start: number
  end: number
}

export interface Event extends BaseRange {
  id: string
}

export interface EventLayout {
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

  constructor() {
    this.start = Infinity
    this.end = -Infinity
    this.columnsOfEvents = []
  }

  add(event: Event) {
    let inserted = false
    for (let i = 0; i < this.columnsOfEvents.length; i++) {
      const column = this.columnsOfEvents[i]
      if (column[column.length - 1].end <= event.start) {
        column.push(event)
        inserted = true
        break
      }
    }
    if (!inserted) {
      this.columnsOfEvents.push([event])
    }
    this.start = Math.min(this.start, event.start)
    this.end = Math.max(this.end, event.end)
  }

  calcLayout(): LayoutGroup {
    const columnCount = this.columnsOfEvents.length
    const totalHeight = this.end - this.start

    const items = this.columnsOfEvents.flatMap((events, i) => events.map(el => ({
      event: el,
      column: i,
      top: (el.start - this.start) / totalHeight,
      height: (el.end - el.start) / totalHeight,
      bottom: (this.end - el.end) / totalHeight,
    })))

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

  const groups: Group[] = []

  for (const event of events) {
    let placed = false
    if (groups.length > 0) {
      const lastGroup = groups[groups.length - 1]
      if (isOverlap(lastGroup, event)) {
        lastGroup.add(event)
        placed = true
      }
    }
    if (!placed) {
      const newGroup = new Group()
      newGroup.add(event)
      groups.push(newGroup)
    }
  }

  return groups
}

export function processEvents(events: Event[]): LayoutGroup[] {
  return mergeEvents([...events].sort((a, b) => a.start - b.start || b.end - a.end)).map(group => group.calcLayout())
}
