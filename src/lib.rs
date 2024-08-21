use wasm_bindgen::prelude::*;

#[derive(Clone)]
pub struct Event {
    id: f32,
    start: f32,
    end: f32,
}

#[derive(Clone)]
pub struct LayoutGroup {
    pub start: f32,
    pub end: f32,
    pub column_count: usize,
    pub items: Vec<EventLayout>,
}

#[derive(Clone)]
pub struct EventLayout {
    pub top: f32,
    pub bottom: f32,
    pub height: f32,
    pub column: usize,
    pub event: Event,
}

#[wasm_bindgen]
pub fn process_events(events_array: &[f32]) -> Box<[f32]> {
    let events = parse_events(&events_array);
    let layout_groups = process_events_impl(&events);
    generate_result_array(&layout_groups)
}

fn parse_events(events_array: &[f32]) -> Vec<Event> {
    let mut events = Vec::with_capacity(events_array.len() as usize / 3);
    for i in 0..(events_array.len() / 3) {
        events.push(Event {
            id: *events_array.get(i * 3).unwrap(),
            start: *events_array.get(i * 3 + 1).unwrap(),
            end: *events_array.get(i * 3 + 2).unwrap(),
        });
    }
    events
}
fn process_events_impl(events: &Vec<Event>) -> Vec<LayoutGroup> {
    // 对事件按开始时间进行排序，若开始时间相同则按结束时间降序排序
    let mut sorted_events = events.clone();
    sorted_events.sort_by(|a, b| {
        a.start
            .partial_cmp(&b.start)
            .unwrap()
            .then_with(|| b.end.partial_cmp(&a.end).unwrap())
    });

    let mut groups: Vec<Group> = Vec::new();

    for event in sorted_events {
        let mut placed = false;

        // 尝试将事件放入现有的组中
        for group in &mut groups {
            if is_overlap(&group, &event) {
                group.add(event.clone());
                placed = true;
                break;
            }
        }

        // 如果没有找到合适的组，则创建一个新的组
        if !placed {
            let mut new_group = Group::new();
            new_group.add(event.clone());
            groups.push(new_group);
        }
    }

    // 计算每个组的布局
    groups
        .into_iter()
        .map(|group| group.calc_layout())
        .collect()
}

struct Group {
    start: f32,
    end: f32,
    columns_of_events: Vec<Vec<Event>>,
    unused_rects: Vec<UnusedRect>,
}

#[derive(Clone, Copy)]
struct UnusedRect {
    start: f32,
    end: f32,
    column_index: usize,
}

impl Group {
    fn new() -> Self {
        Self {
            start: f32::INFINITY,
            end: f32::NEG_INFINITY,
            columns_of_events: Vec::new(),
            unused_rects: Vec::new(),
        }
    }

    fn add(&mut self, event: Event) {
        let mut unused_rects = self.unused_rects.clone();
        let placed = self.insert(&event, &mut unused_rects);

        if !placed {
            let start = self.start.min(event.start);
            let end = self.end.max(event.end);
            let mut empty_rects = self.get_empty_rects(start, end);

            if !empty_rects.is_empty() && self.insert(&event, &mut empty_rects) {
                self.unused_rects = empty_rects;
            } else {
                self.columns_of_events.push(vec![event.clone()]);
            }
        }
        self.start = self.start.min(event.start);
        self.end = self.end.max(event.end);
    }

    fn insert(&mut self, event: &Event, empty_rects: &mut Vec<UnusedRect>) -> bool {
        let mut placed = false;

        for i in 0..empty_rects.len() {
            let rect = &mut empty_rects[i];
            if event.start >= rect.start && event.end <= rect.end {
                if rect.column_index >= self.columns_of_events.len() {
                    self.columns_of_events
                        .resize(rect.column_index + 1, Vec::new());
                }
                self.columns_of_events[rect.column_index].push(event.clone());
                placed = true;

                if event.start == rect.start && event.end == rect.end {
                    empty_rects.remove(i);
                } else if event.start == rect.start {
                    rect.start = event.end;
                } else if event.end == rect.end {
                    rect.end = event.start;
                } else {
                    let new_rects = vec![
                        UnusedRect {
                            start: rect.start,
                            end: event.start,
                            column_index: rect.column_index,
                        },
                        UnusedRect {
                            start: event.end,
                            end: rect.end,
                            column_index: rect.column_index,
                        },
                    ];
                    empty_rects.remove(i);
                    empty_rects.push(new_rects[0].clone());
                    empty_rects.push(new_rects[1].clone());
                }
                break;
            }
        }
        placed
    }

    fn get_empty_rects(&self, start: f32, end: f32) -> Vec<UnusedRect> {
        let mut empty_rects = Vec::new();

        for (column_index, column) in self.columns_of_events.iter().enumerate() {
            let mut last_end = start;

            for event in column {
                if event.start > last_end {
                    empty_rects.push(UnusedRect {
                        start: last_end,
                        end: event.start,
                        column_index,
                    });
                }
                last_end = last_end.max(event.end);
            }
            if last_end < end {
                empty_rects.push(UnusedRect {
                    start: last_end,
                    end,
                    column_index,
                });
            }
        }
        empty_rects
    }

    fn calc_layout(&self) -> LayoutGroup {
        let column_count = self.columns_of_events.len();
        let total_height = self.end - self.start;

        let items: Vec<EventLayout> = self
            .columns_of_events
            .iter()
            .enumerate()
            .flat_map(|(i, events)| {
                events.iter().map(move |event| EventLayout {
                    event: event.clone(),
                    column: i,
                    top: (event.start - self.start) / total_height,
                    height: (event.end - event.start) / total_height,
                    bottom: (self.end - event.end) / total_height,
                })
            })
            .collect();

        LayoutGroup {
            items,
            column_count,
            start: self.start,
            end: self.end,
        }
    }
}

fn is_overlap(group: &Group, event: &Event) -> bool {
    group.start < event.end && event.start < group.end
}

fn generate_result_array(layout_groups: &Vec<LayoutGroup>) -> Box<[f32]> {
    let total_length = layout_groups
        .iter()
        .map(|lg| {
            4 + lg.items.len() * 7 // 4 for LayoutGroup fields + 5 for each EventLayout
        })
        .sum::<usize>();

    let mut result_array = vec![0.0f32; total_length];
    let mut index = 0;

    for layout_group in layout_groups {
        result_array[index] = layout_group.start;
        index += 1;
        result_array[index] = layout_group.end;
        index += 1;
        result_array[index] = layout_group.column_count as f32;
        index += 1;
        result_array[index] = layout_group.items.len() as f32;
        index += 1;

        for event_layout in &layout_group.items {
            result_array[index] = event_layout.top;
            index += 1;
            result_array[index] = event_layout.bottom;
            index += 1;
            result_array[index] = event_layout.height;
            index += 1;
            result_array[index] = event_layout.column as f32;
            index += 1;
            result_array[index] = event_layout.event.id;
            index += 1;
            result_array[index] = event_layout.event.start;
            index += 1;
            result_array[index] = event_layout.event.end;
            index += 1;
        }
    }
    result_array.into_boxed_slice()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_events_impl() {
        // 测试用例 1: 正常事件
        let events = vec![
            Event {
                id: 1.0,
                start: 0.0,
                end: 1.0,
            },
            Event {
                id: 2.0,
                start: 1.0,
                end: 2.0,
            },
            Event {
                id: 3.0,
                start: 2.0,
                end: 3.0,
            },
        ];

        let layout_groups = process_events_impl(&events);
        assert_eq!(layout_groups.len(), 3);

        // 测试用例 2: 重叠事件
        let overlapping_events = vec![
            Event {
                id: 1.0,
                start: 0.0,
                end: 2.0,
            },
            Event {
                id: 2.0,
                start: 1.0,
                end: 3.0,
            },
            Event {
                id: 3.0,
                start: 2.0,
                end: 4.0,
            },
        ];

        let layout_groups = process_events_impl(&overlapping_events);
        assert_eq!(layout_groups.len(), 1);
        assert_eq!(layout_groups[0].start, 0.0);
        assert_eq!(layout_groups[0].end, 4.0);
        assert_eq!(layout_groups[0].column_count, 2);
        assert_eq!(layout_groups[0].items.len(), 3);
        assert_eq!(layout_groups[0].items[0].event.start, 0.0);
        assert_eq!(layout_groups[0].items[0].event.end, 2.0);
        assert_eq!(layout_groups[0].items[1].event.start, 2.0);
        assert_eq!(layout_groups[0].items[1].event.end, 4.0);
        assert_eq!(layout_groups[0].items[2].event.start, 1.0);
        assert_eq!(layout_groups[0].items[2].event.end, 3.0);

        // 测试用例 3: 空事件
        let empty_events: Vec<Event> = vec![];
        let layout_groups = process_events_impl(&empty_events);
        assert_eq!(layout_groups.len(), 0);

        // 测试用例 4: 单个事件
        let single_event = vec![Event {
            id: 1.0,
            start: 1.0,
            end: 2.0,
        }];

        let layout_groups = process_events_impl(&single_event);
        assert_eq!(layout_groups.len(), 1);
        assert_eq!(layout_groups[0].start, 1.0);
        assert_eq!(layout_groups[0].end, 2.0);
        assert_eq!(layout_groups[0].column_count, 1);
        assert_eq!(layout_groups[0].items.len(), 1);
    }
}
