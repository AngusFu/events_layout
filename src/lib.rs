use std::rc::Rc;
use wasm_bindgen::prelude::*;

// Constants
const EVENT_FIELDS: usize = 3;
const LAYOUT_GROUP_FIELDS: usize = 4;
const EVENT_LAYOUT_FIELDS: usize = 7;

#[derive(Clone, Debug)]
pub struct Event {
    id: f32,
    start: f32,
    end: f32,
}

impl Event {
    fn new(id: f32, start: f32, end: f32) -> Result<Self, &'static str> {
        if start >= end {
            return Err("Event start time must be less than end time");
        }
        Ok(Self { id, start, end })
    }
}

#[derive(Clone, Debug)]
pub struct LayoutGroup {
    pub start: f32,
    pub end: f32,
    pub column_count: usize,
    pub items: Vec<EventLayout>,
}

#[derive(Clone, Debug)]
pub struct EventLayout {
    pub top: f32,
    pub bottom: f32,
    pub height: f32,
    pub column: usize,
    pub event: Rc<Event>,
}

#[wasm_bindgen]
pub fn process_events(events_array: &[f32]) -> Box<[f32]> {
    match parse_events(events_array) {
        Ok(events) => {
            let layout_groups = process_events_impl(&events);
            generate_result_array(&layout_groups)
        }
        Err(_) => Box::new([]), // Return empty array on error
    }
}

fn parse_events(events_array: &[f32]) -> Result<Vec<Event>, &'static str> {
    if events_array.len() % EVENT_FIELDS != 0 {
        return Err("Invalid events array length");
    }

    events_array
        .chunks(EVENT_FIELDS)
        .enumerate()
        .map(|(_, chunk)| {
            Event::new(chunk[0], chunk[1], chunk[2]).map_err(|_| "Invalid event data")
        })
        .collect()
}

fn process_events_impl(events: &[Event]) -> Vec<LayoutGroup> {
    // Convert events to Rc
    let events: Vec<Rc<Event>> = events.iter().map(|e| Rc::new(e.clone())).collect();

    // Sort using indices
    let mut indices: Vec<usize> = (0..events.len()).collect();
    indices.sort_by(|&i, &j| {
        events[i]
            .start
            .partial_cmp(&events[j].start)
            .unwrap()
            .then_with(|| events[j].end.partial_cmp(&events[i].end).unwrap())
    });

    let mut groups: Vec<Group> = Vec::with_capacity(events.len());

    for &index in &indices {
        let event = events[index].clone();
        let mut placed = false;

        if let Some(group) = groups.last_mut() {
            if group.is_overlap(&*event) {
                group.add(event.clone());
                placed = true;
            }
        }

        if !placed {
            let mut new_group = Group::new();
            new_group.add(event);
            groups.push(new_group);
        }
    }

    groups
        .into_iter()
        .map(|group| group.calc_layout())
        .collect()
}

#[derive(Debug)]
struct Group {
    start: f32,
    end: f32,
    columns_of_events: Vec<Vec<Rc<Event>>>,
}

impl Group {
    fn new() -> Self {
        Self {
            start: f32::INFINITY,
            end: f32::NEG_INFINITY,
            columns_of_events: Vec::new(),
        }
    }

    fn add(&mut self, event: Rc<Event>) {
        let mut placed = false;

        for column in &mut self.columns_of_events {
            if let Some(last_event) = column.last() {
                if last_event.end <= event.start {
                    column.push(event.clone());
                    placed = true;
                    break;
                }
            }
        }

        if !placed {
            self.columns_of_events.push(vec![event.clone()]);
        }

        self.start = self.start.min(event.start);
        self.end = self.end.max(event.end);
    }

    fn is_overlap(&self, event: &Event) -> bool {
        self.start < event.end && event.start < self.end
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

fn generate_result_array(layout_groups: &[LayoutGroup]) -> Box<[f32]> {
    let total_length = layout_groups
        .iter()
        .map(|lg| LAYOUT_GROUP_FIELDS + lg.items.len() * EVENT_LAYOUT_FIELDS)
        .sum::<usize>();

    let mut result_array = Vec::with_capacity(total_length);
    result_array.resize(total_length, 0.0f32);
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
        // Test case 1: Normal events
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

        // Test case 2: Overlapping events
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

        // Test case 3: Empty events
        let empty_events: Vec<Event> = vec![];
        let layout_groups = process_events_impl(&empty_events);
        assert_eq!(layout_groups.len(), 0);

        // Test case 4: Single event
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

    #[test]
    fn test_complex_overlapping_events() {
        // Test multiple overlapping event groups
        let events = vec![
            Event {
                id: 1.0,
                start: 0.0,
                end: 3.0,
            }, // First group
            Event {
                id: 2.0,
                start: 1.0,
                end: 4.0,
            },
            Event {
                id: 3.0,
                start: 2.0,
                end: 5.0,
            },
            Event {
                id: 4.0,
                start: 6.0,
                end: 8.0,
            }, // Second group
            Event {
                id: 5.0,
                start: 7.0,
                end: 9.0,
            },
            Event {
                id: 6.0,
                start: 8.0,
                end: 10.0,
            },
        ];

        let layout_groups = process_events_impl(&events);
        assert_eq!(layout_groups.len(), 2);

        // Check first group
        assert_eq!(layout_groups[0].start, 0.0);
        assert_eq!(layout_groups[0].end, 5.0);
        assert_eq!(layout_groups[0].column_count, 3);
        assert_eq!(layout_groups[0].items.len(), 3);

        // Check second group
        assert_eq!(layout_groups[1].start, 6.0);
        assert_eq!(layout_groups[1].end, 10.0);
        assert_eq!(layout_groups[1].column_count, 2);
        assert_eq!(layout_groups[1].items.len(), 3);
    }

    #[test]
    fn test_nested_events() {
        // Test nested events (one event completely contained within another)
        let events = vec![
            Event {
                id: 1.0,
                start: 0.0,
                end: 10.0,
            },
            Event {
                id: 2.0,
                start: 2.0,
                end: 8.0,
            },
            Event {
                id: 3.0,
                start: 3.0,
                end: 7.0,
            },
        ];

        let layout_groups = process_events_impl(&events);
        assert_eq!(layout_groups.len(), 1);
        assert_eq!(layout_groups[0].column_count, 3);
        assert_eq!(layout_groups[0].items.len(), 3);
    }

    #[test]
    fn test_edge_cases() {
        // Test edge cases
        let events = vec![
            Event {
                id: 1.0,
                start: 0.0,
                end: 0.0,
            }, // Zero duration event
            Event {
                id: 2.0,
                start: 1.0,
                end: 1.0,
            }, // Zero duration event
            Event {
                id: 3.0,
                start: 2.0,
                end: 2.0,
            }, // Zero duration event
            Event {
                id: 4.0,
                start: 3.0,
                end: 3.0,
            }, // Zero duration event
        ];

        let layout_groups = process_events_impl(&events);
        assert_eq!(layout_groups.len(), 4);
        for group in layout_groups {
            assert_eq!(group.column_count, 1);
            assert_eq!(group.items.len(), 1);
        }
    }

    #[test]
    fn test_large_number_of_events() {
        // Test large number of events
        let mut events = Vec::new();
        for i in 0..100 {
            events.push(Event {
                id: i as f32,
                start: (i as f32) * 0.5,
                end: (i as f32) * 0.5 + 1.0,
            });
        }

        let layout_groups = process_events_impl(&events);
        assert!(layout_groups.len() > 0);

        // Verify all events are processed correctly
        let total_events = layout_groups.iter().map(|g| g.items.len()).sum::<usize>();
        assert_eq!(total_events, 100);
    }

    #[test]
    fn test_random_overlapping_events() {
        // Test random overlapping events
        let events = vec![
            Event {
                id: 1.0,
                start: 0.0,
                end: 5.0,
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
            Event {
                id: 4.0,
                start: 3.0,
                end: 6.0,
            },
            Event {
                id: 5.0,
                start: 4.0,
                end: 7.0,
            },
            Event {
                id: 6.0,
                start: 5.0,
                end: 8.0,
            },
        ];

        let layout_groups = process_events_impl(&events);
        assert_eq!(layout_groups.len(), 1);
        assert_eq!(layout_groups[0].column_count, 3);
        assert_eq!(layout_groups[0].items.len(), 6);

        // Verify layout correctness
        for item in &layout_groups[0].items {
            assert!(item.top >= 0.0);
            assert!(item.bottom >= 0.0);
            assert!(item.height > 0.0);
            assert!(item.column < layout_groups[0].column_count);
        }
    }

    #[test]
    fn test_demo_example() {
        let events = vec![
            Event::new(1.0, 0.0, 2.0).unwrap(),
            Event::new(2.0, 1.0, 3.0).unwrap(),
            Event::new(3.0, 2.0, 4.0).unwrap(),
        ];

        let layout_groups = process_events_impl(&events);
        assert_eq!(layout_groups.len(), 1);
        assert_eq!(layout_groups[0].start, 0.0);
        assert_eq!(layout_groups[0].end, 4.0);
        assert_eq!(layout_groups[0].column_count, 2);
        assert_eq!(layout_groups[0].items.len(), 3);

        // 验证布局位置
        let mut columns = vec![Vec::new(); layout_groups[0].column_count];
        for item in &layout_groups[0].items {
            columns[item.column].push((item.event.start, item.event.end));
        }

        // 验证每列中的事件不重叠
        for column in &columns {
            let mut sorted = column.clone();
            sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            for i in 1..sorted.len() {
                assert!(sorted[i - 1].1 <= sorted[i].0);
            }
        }

        // 验证所有事件都被分配到了列中
        assert_eq!(columns.iter().map(|c| c.len()).sum::<usize>(), 3);
    }
}
