// use std::collections::BTreeMap;
// use std::fs::File;
// use std::io::{Read, Seek, SeekFrom};
// use std::ops::Deref;
// use std::path::{PathBuf};
// use druid::{AppLauncher, Color, Data, Lens, lens, LensExt, UnitPoint, Widget, WidgetExt, WindowDesc};
// use druid::widget::{Align, Button, Flex, Label, List, ListIter, Maybe, Scroll, Split, Tabs};
// use pcode::binary::Binary;
// use pcode::emulator::Machine;
// use crate::shared::{ReadCloned, Shared};
// use crate::task::task;
//
// mod task;
// mod shared;
// mod data;
//
// #[derive(Clone, Data, Lens)]
// struct GlobalState {
//     program: CurrentProgram,
//     #[data(same_fn = "data::shared_error_eq")]
//     errors: Shared<Vec<anyhow::Error>>,
// }
//
// #[derive(Clone, Data, Default)]
// pub struct CurrentProgram(Shared<Option<Shared<ProgramData>>>);
//
// #[derive(Clone, Data, Lens)]
// struct ProgramData {
//     #[data(eq)]
//     path: PathBuf,
//     #[data(ignore)]
//     binary: Shared<Binary>,
//     #[data(ignore)]
//     machine: Shared<Machine>,
// }
//
// fn main() {
//     let window = WindowDesc::new(ui_builder())
//         .window_size((800.0, 600.0));
//
//     let state = GlobalState {
//         program: CurrentProgram::default(),
//         errors: Shared::default(),
//     };
//
//     AppLauncher::with_window(window)
//         .launch(state)
//         .expect("launch failed");
// }
//
// fn ui_builder() -> impl Widget<GlobalState> {
//     Tabs::new()
//         .with_tab("Main", main_ui())
//         .with_tab("Emulator", emulator_ui())
//         .with_tab("Disassembly", disassembly_ui())
// }
//
// fn main_ui() -> impl Widget<GlobalState> {
//     let mut column = Flex::column();
//
//     column.add_child(Flex::row()
//         .with_child(Button::new("Current Program: ")
//             .on_click(|ctx, _data, _env| {
//                 task(ctx.get_external_handle(), pick_program, |state: &mut GlobalState, program| {
//                     let Some(result) = program else {
//                         return;
//                     };
//                     match result {
//                         Ok((path, binary, machine)) => state.set_program(ProgramData {
//                             path,
//                             binary: Shared::new(binary),
//                             machine: Shared::new(machine),
//                         }),
//                         Err(error) => state.set_error(error),
//                     }
//                 });
//             })
//         )
//         .with_child(Label::dynamic(|data: &GlobalState, _env| {
//             if let Some(program) = data.program.0.read().deref() {
//                 let program = program.read();
//                 format!("Current program: {}", program.path.display())
//             } else {
//                 "No program loaded".to_string()
//             }
//         }))
//     );
//
//     column
// }
//
// fn emulator_ui() -> impl Widget<GlobalState> {
//     fn registers() -> impl Widget<GlobalState> {
//         Maybe::new(
//             || Label::new("Registers"),
//             || Label::new("No program loaded"),
//         )
//             .lens(lens::Identity.map(
//                 |state: &GlobalState| state.program.0.read().clone(),
//                 |state: &mut GlobalState, value| {
//                     state.program.0.replace(value);
//                 },
//             ))
//     }
//
//     fn memory() -> impl Widget<GlobalState> {
//         Label::new("Memory")
//     }
//
//     fn debugger() -> impl Widget<GlobalState> {
//         Label::new("Debugger")
//     }
//
//     Split::columns(
//         Split::rows(registers(), memory()),
//         debugger(),
//     )
//         .split_point(0.3)
// }
//
// fn disassembly_ui() -> impl Widget<GlobalState> {
//     fn disassembly() -> impl Widget<Shared<ProgramData>> {
//         Scroll::new(
//             List::new(|| {
//                 Flex::row()
//                     .with_child(
//                         Label::new(|(_, item): &(_, u32), _env: &_| {
//                             format!("List item #{item}")
//                         })
//                             .align_vertical(UnitPoint::LEFT),
//                     )
//                     .with_flex_spacer(1.0)
//                     // .with_child(
//                     //     Button::new("Delete")
//                     //         .on_click(|_ctx, (shared, item): &mut (Vector<u32>, u32), _env| {
//                     //             // We have access to both child's data and shared data.
//                     //             // Remove element from right list.
//                     //             shared.retain(|v| v != item);
//                     //         })
//                     //         .fix_size(80.0, 20.0)
//                     //         .align_vertical(UnitPoint::CENTER),
//                     // )
//                     .padding(10.0)
//                     .background(Color::grey(0.5))
//                     .fix_height(50.0)
//             })
//                 .with_spacing(10.),
//         )
//             .vertical()
//             .lens(lens::Identity.map(
//                 // Expose shared data with children data
//                 |d: &Shared<ProgramData>| d.read().machine.read().instructions.clone(),
//                 |d: &mut Shared<ProgramData>, x| {},
//             ))
//     }
//
//     Maybe::new(
//         disassembly,
//         || Label::new("No program loaded"),
//     )
//         .lens(lens::Identity.map(
//             |state: &GlobalState| state.program.0.read().clone(),
//             |state: &mut GlobalState, value| {
//                 state.program.0.replace(value);
//             },
//         ))
// }
//
// impl GlobalState {
//     pub fn set_program(&mut self, program: ProgramData) {
//         self.program = CurrentProgram(Shared::new(Some(Shared::new(program))));
//     }
//
//     pub fn set_error(&mut self, error: anyhow::Error) {
//         eprintln!("Error: {}", error);
//         self.errors.write_in_place(|errors| errors.push(error));
//     }
// }
//
// fn pick_program() -> Option<anyhow::Result<(PathBuf, Binary, Machine)>> {
//     let mut dialog = rfd::FileDialog::new();
//
//     if let Ok(dir) = std::env::current_dir() {
//         dialog = dialog.set_directory(dir);
//     }
//
//     let path = dialog.pick_file()?;
//     let binary = match Binary::x86_32(&path) {
//         Ok(binary) => binary,
//         Err(result) => {
//             return Some(Err(result));
//         }
//     };
//
//     let machine = match Machine::new(&binary) {
//         Ok(binary) => binary,
//         Err(e) => {
//             return Some(Err(e));
//         }
//     };
//
//     Some(Ok((path, binary, machine)))
// }
//
// #[derive(Data, Clone)]
// struct TreeListIter<K, V>(BTreeMap<K, V>);
//
// impl<K, V> ListIter<(K, V)> for TreeListIter<K, V> {
//     fn for_each(&self, cb: impl FnMut(&(K, V), usize)) {
//         todo!()
//     }
//
//     fn for_each_mut(&mut self, cb: impl FnMut(&mut (K, V), usize)) {
//         todo!()
//     }
//
//     fn data_len(&self) -> usize {
//         todo!()
//     }
// }

use std::backtrace::BacktraceStatus;
use std::path::PathBuf;
use eframe::{App, Frame};
use eframe::egui::{CentralPanel, Context, Label, ScrollArea, SidePanel, Window};
use pcode::binary::Binary;
use pcode::emulator::{Cursor, Emulator, Machine};
use crate::task::{Task, TaskHandle};

mod task;

fn main() {
    eframe::run_native(
        "Emulator",
        eframe::NativeOptions::default(),
        Box::new(|_| Box::<EmulatorApp>::default()),
    ).unwrap();
}

#[derive(Default)]
pub struct EmulatorApp {
    program: Option<CurrentProgram>,
    error: Option<anyhow::Error>,
    load_program: Option<Task<Option<anyhow::Result<CurrentProgram>>>>,

    symbol_name: String,
}

pub struct CurrentProgram {
    path: PathBuf,
    binary: Binary,
    machine: Machine,
    emulator: Option<(Emulator, Cursor)>,
}

impl App for EmulatorApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        if let Some(Some(value)) = self.load_program.get_result() {
            match value {
                Ok(prog) => {
                    self.program = Some(prog);
                }
                Err(err) => {
                    self.error = Some(err);
                }
            };
        }

        if let Some(err) = self.error.as_ref() {
            let mut open = true;
            Window::new("Error")
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.heading(format!("Error: {}", err.root_cause()));
                    ui.label(err.to_string());
                });

            if !open {
                self.error = None;
            }
        }

        if let Some(program) = &mut self.program {
            let Some((emulator, cursor)) = program.emulator.as_mut() else {
                CentralPanel::default()
                    .show(ctx, |ui| {
                        ui.heading(format!("Current Program: {}", program.path.display()));
                        ui.text_edit_singleline(&mut self.symbol_name);

                        if ui.button("emulate").clicked() {
                            if !program.binary.symbols.contains_key(&self.symbol_name) {
                                self.error = Some(anyhow::anyhow!("symbol not found"));
                                return;
                            }

                            let emulator = match program.machine.emulate(&program.binary, &self.symbol_name) {
                                Ok(emulator) => emulator,
                                Err(e) => {
                                    self.error = Some(e);
                                    return;
                                }
                            };
                            program.emulator = Some(emulator);
                        }
                    });

                return;
            };

            SidePanel::left("side_panel")
                .resizable(true)
                .show(ctx, |ui| {
                    ui.heading("Registers");

                    ScrollArea::vertical()
                        .show(ui, |ui| {
                            for (name, value) in &program.machine.named_registers {
                                let value: u64 = emulator.read(value);
                                ui.label(format!("{}: {:0>16X} ({})", name, value, value));
                            }
                        });
                });
            SidePanel::right("memory")
                .show(ctx, |ui| {
                    ui.heading("Memory");
                });
            CentralPanel::default()
                .show(ctx, |ui| {
                    ui.heading(format!("Current Program: {}", program.path.display()));
                });
        } else {
            CentralPanel::default()
                .show(ctx, |ui| {
                    if ui.button("Select Program").clicked() && self.load_program.is_none() {
                        self.load_program = Some(Task::new(pick_program));
                    }
                });
        }
    }
}

fn pick_program() -> Option<anyhow::Result<CurrentProgram>> {
    let mut dialog = rfd::FileDialog::new();

    if let Ok(dir) = std::env::current_dir() {
        dialog = dialog.set_directory(dir);
    }

    let path = dialog.pick_file()?;
    let binary = match Binary::x86_32(&path) {
        Ok(binary) => binary,
        Err(result) => {
            return Some(Err(result));
        }
    };

    let mut machine = match Machine::new(&binary) {
        Ok(binary) => binary,
        Err(e) => {
            return Some(Err(e));
        }
    };

    Some(Ok(CurrentProgram {
        path,
        binary,
        machine,
        emulator: None,
    }))
}
