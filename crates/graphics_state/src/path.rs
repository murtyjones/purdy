use anyhow::{Ok, Result};
use lyon::geom::{vector, LineSegment, Point as SPoint};
use lyon::math::{point, Point, Vector};
use lyon::path::PathEvent;
use std::convert::TryInto;

use shared::{Height, Width};

use crate::draw_state::{Command, DrawState, State};
use crate::geom::as_rect;

#[derive(Debug)]
pub(crate) struct Path {
    events: Vec<PathEvent>,
    draw_state: DrawState,
    bottom_left_of_page: Point,
}

impl Path {
    pub fn new(page_width: Width, page_height: Height) -> Self {
        Path {
            events: vec![],
            draw_state: DrawState::default(),
            bottom_left_of_page: point(-page_width / 2.0, page_height / 2.0),
        }
    }

    pub fn move_to(&mut self, to: Vector) -> Result<()> {
        self.end_if_needed()?;

        let to = vector(to.x, -to.y);

        self.begin(to)?;

        self.draw_state.assert_is_active()?;

        Ok(())
    }

    pub fn line_to(&mut self, to: Vector) -> Result<()> {
        self.draw_state.make_commands(Command::LineTo)?;
        let from = self.draw_state.assert_is_commands()?.current;
        let to = vector(to.x, -to.y);
        let to = self.bottom_left_of_page + to;
        self.events.push(PathEvent::Line { from, to });
        Ok(())
    }

    pub fn rect(&mut self, low_left: Vector, width: Width, height: Height) -> Result<()> {
        let width = f32::max(*width, 1.0);
        let height = f32::max(*height, 1.0);
        self.move_to(low_left)?;
        let to = vector(low_left.x + width, low_left.y);
        self.line_to(to)?;
        let to = vector(low_left.x + width, low_left.y + height);
        self.line_to(to)?;
        let to = vector(low_left.x, low_left.y + height);
        self.line_to(to)?;
        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        self.end(true)
    }

    pub fn end(&mut self, close: bool) -> Result<()> {
        self.draw_state.assert_is_not_inactive()?;
        match self.draw_state.current() {
            State::Active(a) => {
                self.events.push(PathEvent::End {
                    first: a.first,
                    last: a.first,
                    close: false,
                });
            }
            State::Commands(c) => {
                self.events.push(PathEvent::End {
                    first: c.first,
                    last: c.current,
                    close,
                });
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    pub fn build(&mut self) -> Result<Vec<PathEvent>> {
        let events = std::mem::take(&mut self.events);
        Ok(events)
    }

    fn begin(&mut self, to: Vector) -> Result<()> {
        // TODO: I think this assertion should always be true... but not 100% sure
        self.draw_state.assert_is_inactive()?;
        let at = self.bottom_left_of_page + to;

        self.events.push(PathEvent::Begin { at });
        self.draw_state.make_active(at)?;
        Ok(())
    }

    fn end_if_needed(&mut self) -> Result<()> {
        let maybe_last = self.events.last();
        match maybe_last {
            Some(PathEvent::Begin { at }) => {
                let at = *at;
                self.events.push(PathEvent::End {
                    first: at,
                    last: at,
                    close: false,
                })
            }
            // Maybe we just do nothing if the last command was LineTo etc?
            Some(_) => unimplemented!(),
            None => {}
        }
        Ok(())
    }

    /// For any single linetos ending with a fill, makes them rectangles that can be filled
    pub fn make_fillable_if_needed(&mut self) {
        let mut insertions = vec![];
        let mut replacements = vec![];
        let mut after_replacements = vec![];
        for (first_item_index, window) in self.events.windows(3).enumerate() {
            let [a, b, c]: [PathEvent; 3] = window.try_into().unwrap();
            let begin = match a {
                PathEvent::Begin { at } => Some(at),
                _ => None,
            };
            let line = match b {
                PathEvent::Line { from, to } => Some((from, to)),
                _ => None,
            };
            let end = match c {
                PathEvent::End { last, first, .. } => Some((first, last)),
                _ => None,
            };
            if let (Some(_begin), Some(line), Some(_end)) = (begin, line, end) {
                let (from, to) = line;
                let to = if from == to {
                    point(to.x + 1.0, to.y - 1.0)
                } else {
                    to
                };
                let line = LineSegment { from, to };
                let rect_points = as_rect(line);

                let (
                    SPoint { x: x1, y: y1, .. },
                    SPoint { x: x2, y: y2, .. },
                    SPoint { x: x3, y: y3, .. },
                    SPoint { x: x4, y: y4, .. },
                ) = (
                    rect_points[0],
                    rect_points[1],
                    rect_points[2],
                    rect_points[3],
                );
                let from = point(x1, y1);
                let first = from;
                let to = point(x2, y2);
                // Replace Begin
                replacements.push((first_item_index, PathEvent::Begin { at: from }));
                // Replace first Line
                replacements.push((first_item_index + 1, PathEvent::Line { from, to }));
                let from = point(x3, y3);
                let to = point(x4, y4);
                // Insert third Line
                insertions.push((first_item_index + 3, PathEvent::Line { from, to }));
                // Replace the end
                after_replacements.push((
                    first_item_index + 4,
                    PathEvent::End {
                        first,
                        last: to,
                        close: true,
                    },
                ));
                let from = point(x2, y2);
                let to = point(x3, y3);
                // Insert second Line
                insertions.push((first_item_index + 2, PathEvent::Line { from, to }));
            }
        }

        for (i, event) in replacements.into_iter() {
            self.events[i] = event;
        }
        for (i, event) in insertions.into_iter().rev() {
            self.events.insert(i, event);
        }
        for (i, event) in after_replacements.into_iter() {
            self.events[i] = event;
        }
    }
}
