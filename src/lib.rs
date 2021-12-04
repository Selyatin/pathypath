#![warn(clippy::all)]

use eframe::{egui::*, epi};
use std::{
    collections::VecDeque,
    ops::{Add, Sub},
};
use rustc_hash::FxHashMap;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Cell {
    Empty,
    Wall,
    Visited,
    Path,
    Start,
    End,
}

pub struct App {
    grid: Vec<Cell>,
    selected_cell_type: Cell,
    start_pos: Option<(usize, usize)>,
    // Used to traceback the path from end -> start by tracing each cell's parent cell
    traceback_pos: Option<(usize, usize)>,
    end_exists: bool,
    searching: bool,
    drawing_shortest_path: bool,
    cell_size: f32,
    speed: usize,
    queue: VecDeque<(usize, usize)>,
    parent_cells: FxHashMap<(usize, usize), (usize, usize)>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            grid: vec![],
            start_pos: None,
            traceback_pos: None,
            end_exists: false,
            selected_cell_type: Cell::Wall,
            searching: false,
            drawing_shortest_path: false,
            cell_size: 25.0,
            queue: VecDeque::new(),
            parent_cells: FxHashMap::default(),
            speed: 2,
        }
    }
}

impl App {
    fn reset(&mut self) {
        self.grid.iter_mut().for_each(|cell| *cell = Cell::Empty);
        self.queue.clear();
        self.parent_cells.clear();
        self.start_pos = None;
        self.traceback_pos = None;
        self.end_exists = false;
        self.searching = false;
        self.drawing_shortest_path = false;
    }

    fn breadth_first_search(&mut self, width: usize) {
        for _ in 0..self.speed {
            let (parent_row, parent_column) = match self.queue.pop_back() {
                Some(tuple) => tuple,
                None => {
                    self.searching = false;
                    return;
                }
            };

            let neighbors = [
                (parent_row.add(1), parent_column),
                (parent_row.sub(1), parent_column),
                (parent_row, parent_column.add(1)),
                (parent_row, parent_column.sub(1)),
            ];

            for (row, column) in neighbors {
                if column >= width {
                    continue;
                }
                let index = row * width + column;
                let cell = match self.grid.get_mut(index) {
                    Some(cell) => cell,
                    None => continue,
                };
                match cell {
                    Cell::Wall => continue,
                    Cell::Visited => continue,
                    Cell::Empty => {
                        self.queue.push_front((row, column));
                        self.parent_cells.insert((row, column), (parent_row, parent_column));
                        *cell = Cell::Visited
                    }
                    Cell::End => {
                        self.traceback_pos = Some((row, column));
                        self.parent_cells.insert((row, column), (parent_row, parent_column));
                        self.searching = false;
                        self.drawing_shortest_path = true;
                        return;
                    }
                    _ => (),
                };
            }
        }
    }

    fn shortest_path(&mut self, width: usize){
        for _ in 0..self.speed {
            let pos = self.traceback_pos.unwrap();
            // Row and Column of Parent cell.
            let (row, column) = match self.parent_cells.remove(&pos){
                Some(tuple) => tuple,
                None => return
            };
            // In the next iteration get current parent cell's parent cell.
            self.traceback_pos = Some((row, column));
            let index = row * width + column;
            match self.grid[index] {
                Cell::Start => {
                    self.drawing_shortest_path = false;
                    self.traceback_pos = None;
                    self.parent_cells.clear();
                    return;
                },
                _ => self.grid[index] = Cell::Path
            };
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "Pathypath"
    }

    fn update(&mut self, ctx: &CtxRef, _frame: &mut epi::Frame<'_>) {
        CentralPanel::default().show(ctx, |ui| {
            let available_size = ui.available_size();

            let (mut painter_response, painter) =
                ui.allocate_painter(available_size, Sense::drag());

            // width/height transformed to columns/rows
            let available_size = Vec2::new(
                (available_size.x / self.cell_size).round(),
                (available_size.y / self.cell_size).round(),
            );

            let cell_count = (available_size.x * available_size.y) as usize;

            // Add Cells if available_space got bigger
            if self.grid.len() < cell_count {
                self.reset();
                while self.grid.len() < cell_count {
                    self.grid.push(Cell::Empty);
                }
            } else if self.grid.len() > cell_count {
                self.reset();
                // Remove cells if available_space got smaller
                while self.grid.len() > cell_count {
                    self.grid.pop();
                }
            }

            if let Some(pointer_pos) = painter_response.interact_pointer_pos() {
                let row = (pointer_pos.y / self.cell_size).round();
                let column = (pointer_pos.x / self.cell_size).round();
                // row * width + column
                let index = (row * available_size.x + column) as usize;
                if let Some(cell) = self.grid.get_mut(index) {
                    let cell_type = match self.selected_cell_type {
                        Cell::Start => {
                            if self.start_pos.is_some() {
                                None
                            } else {
                                self.start_pos = Some((row as usize, column as usize));
                                Some(Cell::Start)
                            }
                        }

                        Cell::End => {
                            if self.end_exists {
                                None
                            } else {
                                self.end_exists = true;
                                Some(Cell::End)
                            }
                        }
                        _ => Some(self.selected_cell_type),
                    };

                    if let Some(cell_type) = cell_type {
                        match cell {
                            Cell::Start => {
                                if cell_type != Cell::Start {
                                    self.start_pos = None;
                                }
                            }

                            Cell::End => {
                                if cell_type != Cell::End {
                                    self.end_exists = false;
                                }
                            }

                            _ => (),
                        };

                        *cell = cell_type;
                        painter_response.mark_changed();
                    }
                }
            }

            if self.searching {
                self.breadth_first_search(available_size.x as usize);
                painter_response.mark_changed();
                ctx.request_repaint();
            }

            if self.drawing_shortest_path {
                self.shortest_path(available_size.x as usize);
                painter_response.mark_changed();
                ctx.request_repaint();
            }

            let (mut x, mut y, mut i) = (0.0, 0.0, 0);

            while y < available_size.y {
                while x < available_size.x {
                    let (pos_x, pos_y) = (x * self.cell_size, y * self.cell_size);
                    let rect = Rect::from_two_pos(
                        Pos2::new(pos_x, pos_y),
                        Pos2::new(pos_x + self.cell_size, pos_y + self.cell_size),
                    );
                    let shape = match self.grid[i] {
                        Cell::Wall => Shape::rect_filled(rect, 1.0, Color32::LIGHT_BLUE),
                        Cell::Start => Shape::rect_filled(rect, 1.0, Color32::BLUE),
                        Cell::End => Shape::rect_filled(rect, 1.0, Color32::GOLD),
                        Cell::Visited => Shape::rect_filled(rect, 1.0, Color32::GRAY),
                        Cell::Path => Shape::rect_filled(rect, 1.0, Color32::GREEN),
                        Cell::Empty => {
                            Shape::rect_stroke(rect, 1.0, Stroke::new(0.1, Color32::LIGHT_BLUE))
                        }
                    };
                    painter.add(shape);
                    x += 1.0;
                    i += 1;
                }
                y += 1.0;
                x = 0.0;
            }
        });

        containers::panel::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            Grid::new("grid").min_row_height(30.0).show(ui, |ui| {
                ui.radio_value(&mut self.selected_cell_type, Cell::Wall, "Wall");
                ui.radio_value(&mut self.selected_cell_type, Cell::Start, "Start Point");
                ui.radio_value(&mut self.selected_cell_type, Cell::End, "End Point");
                ui.radio_value(&mut self.selected_cell_type, Cell::Empty, "Empty");
                if ui.button("Search").clicked() && self.start_pos.is_some() && self.end_exists {
                    for cell in &mut self.grid {
                        if *cell == Cell::Visited || *cell == Cell::Path{
                            *cell = Cell::Empty;
                        }
                    }
                    self.queue.clear();
                    self.queue.push_front(self.start_pos.unwrap());
                    self.searching = true;
                }
                ui.radio_value(&mut self.speed, 2, "Slow");
                ui.radio_value(&mut self.speed, 15, "Fast");
                ui.radio_value(&mut self.speed, 50, "Faster");
                ui.radio_value(&mut self.speed, usize::MAX, "Instant");
                ui.label("Cell Size");
                ui.add(DragValue::new(&mut self.cell_size).speed(1.0));
                if ui.button("Reset").clicked() {
                    self.reset();
                }
            });
        });
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), JsValue> {
    let app = App::default();
    eframe::start_web(canvas_id, Box::new(app))
}
