use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use netlib::{Reaction, Reactor, Timer};
use rand::prelude::*;
use tinybit::events::{events, Event, EventModel, Events, KeyCode, KeyEvent};
use tinybit::widgets::Text;
use tinybit::{term_size, Pixel, Renderer, ScreenPos, ScreenSize, StdoutTarget, Viewport};

const MAX_ACTIVE: usize = 10;

// -----------------------------------------------------------------------------
//     - Scene -
// -----------------------------------------------------------------------------
enum Scene {
    Running {
        target: u8,
        inputs: VecDeque<(u8, String)>,
        active: Vec<(Pixel, u8, String)>,
    },
    GameOver(String, usize),
}

// -----------------------------------------------------------------------------
//     - Game -
// -----------------------------------------------------------------------------
pub struct Game {
    timer: Timer,
    input: u8,
    leaderboard: HashMap<String, usize>,
    scene: Scene,
    viewport: Viewport,
    renderer: Renderer<StdoutTarget>,
    centre: ScreenPos,
    events: Events,
}

impl Game {
    pub fn new() -> Self {
        let freq = Duration::from_millis(100);

        let (w, h) = term_size().unwrap();
        let viewport = Viewport::new(ScreenPos::zero(), ScreenSize::new(w, h));

        let stdout = StdoutTarget::new().unwrap();
        let renderer = Renderer::new(stdout);

        let mut inst = Self {
            timer: Timer::new(Duration::new(0, 1), Some(freq)).unwrap(),
            input: 0,
            leaderboard: HashMap::new(),
            scene: Scene::Running {
                target: random(),
                inputs: VecDeque::new(),
                active: Vec::new(),
            },
            viewport,
            renderer,
            centre: ScreenPos::new(w / 2 - 4, h / 2),
            events: events(EventModel::NonBlocking),
        };

        inst.restart();

        inst
    }

    fn restart(&mut self) {
        match self.scene {
            Scene::GameOver(..) => {
                let target = {
                    let mut t = self.input;
                    while t == self.input {
                        t = random();
                    }
                    t
                };

                self.scene = Scene::Running {
                    target,
                    inputs: VecDeque::new(),
                    active: Vec::new(),
                }
            }
            Scene::Running { .. } => {}
        }
    }

    fn tick(&mut self) {
        // Sneaky way of being able to quit
        match self.events.next() {
            Some(Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            })) => std::process::exit(0),
            Some(Event::Resize(w, h)) => {
                self.centre = ScreenPos::new(w / 2, h / 2);
                self.viewport.resize(w, h);
                self.renderer.clear();
            }
            _ => {}
        }

        match self.scene {
            Scene::GameOver(ref name, ref mut countdown) => {
                *countdown -= 1;
                if *countdown == 0 {
                    self.restart();
                    return;
                }
                let text = Text::new(format!("The winner is: {}", name), None, None);
                self.viewport
                    .draw_widget(&text, ScreenPos::new(self.centre.x, self.centre.y));

                let mut y = self.centre.y + 2;
                for (name, wins) in &self.leaderboard {
                    let text = Text::new(format!("{} : {}", name, wins), None, None);
                    self.viewport
                        .draw_widget(&text, ScreenPos::new(self.centre.x, y));
                    y += 1;
                }

                self.renderer.render(&mut self.viewport);
            }
            Scene::Running {
                ref mut active,
                ref mut inputs,
                target,
            } => {
                if active.len() < MAX_ACTIVE {
                    if let Some((val, name)) = inputs.pop_front() {
                        let offset = 7 - val as u16;
                        let x = offset + self.centre.x;
                        let pixel = Pixel::new('1', ScreenPos::new(x, 1), None, None);
                        active.push((pixel, val, name));
                    }
                }

                let centre_y = self.centre.y;
                let mut delete = Vec::new();
                let input = &mut self.input;

                let mut winner = None;

                let viewport = &mut self.viewport;
                active
                    .iter_mut()
                    .enumerate()
                    .for_each(|(i, (ref mut pixel, val, name))| {
                        pixel.pos.y += 1;
                        viewport.draw_pixel(*pixel);

                        if pixel.pos.y == centre_y {
                            let mask = 1 << *val;
                            *input = *input ^ mask;
                            delete.push(i);

                            if *input == target {
                                winner = Some(name.clone());
                            }
                        }
                    });

                // Hackery hackery
                delete.iter().rev().for_each(|i| drop(active.remove(*i)));

                // -----------------------------------------------------------------------------
                //     - Render -
                // -----------------------------------------------------------------------------
                let target_text = Text::new(format!("{:08b}", target), None, None);
                let input_text = Text::new(format!("{:08b}", input), None, None);

                self.viewport
                    .draw_widget(&input_text, ScreenPos::new(self.centre.x, self.centre.y));

                self.viewport.draw_widget(
                    &target_text,
                    ScreenPos::new(self.centre.x, self.centre.y + 2),
                );

                self.renderer.render(&mut self.viewport);

                if let Some(name) = winner {
                    let entry = self.leaderboard.entry(name.clone()).or_insert(0);
                    *entry += 1;
                    self.scene = Scene::GameOver(name, 30);
                }
            }
        }
    }
}

impl Reactor for Game {
    type Input = (u8, String);
    type Output = ();

    fn react(&mut self, reaction: Reaction<Self::Input>) -> Reaction<Self::Output> {
        match reaction {
            Reaction::Event(ev) if ev.owner != self.timer.reactor_id => Reaction::Event(ev),
            Reaction::Event(_) => {
                self.tick();
                let _ = self.timer.consume_event();
                Reaction::Continue
            }
            Reaction::Value(val) => {
                if let Scene::Running { inputs, .. } = &mut self.scene {
                    inputs.push_back(val);
                }
                Reaction::Continue
            }
            Reaction::Continue => Reaction::Continue,
        }
    }
}
