use std::any::type_name;
use serde::Serialize;

use eframe::egui;
use eframe::egui::{Color32, Galley, RichText, TextFormat, Ui, WidgetText};
use eframe::egui::text::LayoutJob;
use eframe::epaint::TextShape;
use serde_value::{NamedStruct, NamedVariant, TupleStruct, TupleVariant, UnitStruct, UnitVariant, Value};

pub(super) fn inspect<T: Serialize>(value: &T) {
    let value = serde_value::to_value(value).unwrap();
    let name = type_name::<T>();
    inner(name, value).unwrap()
}

fn inner(name: &'static str, value: Value) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };
    eframe::run_native(
        format!("inspect: {}", name).as_str(),
        options,
        Box::new(move |_cc| Box::new(Inspector { name, value })),
    )
}

struct Inspector {
    name: &'static str,
    value: Value,
}

impl eframe::App for Inspector {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("inspect: {}", self.name));
            egui::ScrollArea::vertical().show(ui, |ui| {
                build_component(ui, &self.value);
            });
        });
    }
}

fn build_component(ui: &mut Ui, value: &Value) {
    match value {
        Value::Unit => {
            ui.label("()");
        }
        Value::Bool(value) => {
            ui.label(value.to_string());
        }
        Value::Char(value) => {
            ui.label(format!("{:?}", value));
        }
        Value::Number(num) => {
            ui.label(format!("{}", num));
        }
        Value::String(num) => {
            ui.label(format!("{:?}", num));
        }
        Value::Seq(values) => {
            build_component_list(ui, values);
        }
        Value::Map(values) => {
            for (i, (key, value)) in values.iter().enumerate() {
                ui.collapsing(RichText::new(i.to_string()).strong(), |ui| {
                    ui.collapsing("key", |ui| build_component(ui, key));
                    ui.collapsing("value", |ui| build_component(ui, value));
                });
            }
        }
        Value::Tuple(values) => {
            build_component_list(ui, values);
        }
        Value::UnitStruct(UnitStruct { name, .. }) => {
            ui.label(format!("struct {name}"));
        }
        Value::TupleStruct(TupleStruct { name, values, .. }) => {
            ui.collapsing(*name, |ui| build_component_list(ui, values));
        }
        Value::NamedStruct(NamedStruct { name, fields, .. }) => {
            ui.collapsing(*name, |ui| {
                for (name, value) in fields {
                    ui.collapsing(*name, |ui| {
                        build_component(ui, value);
                    });
                }
            });
        }
        Value::UnitVariant(UnitVariant { name, variant, .. }) => {
            ui.label(format!("{}::{}", *name, *variant));
        }
        Value::TupleVariant(TupleVariant { name, variant, values, .. }) => {
            ui.collapsing(format!("{}::{}", *name, *variant), |ui| {
                build_component_list(ui, values);
            });
        }
        Value::NamedVariant(NamedVariant { name, variant, fields, .. }) => {
            ui.collapsing(format!("{}::{}", *name, *variant), |ui| {
                for (name, value) in fields {
                    ui.collapsing(*name, |ui| {
                        build_component(ui, value);
                    });
                }
            });
        }
    };
}

fn build_component_list<'a>(ui: &mut Ui, values: impl IntoIterator<Item=&'a Value>) {
    let values = values.into_iter().enumerate();
    for (i, value) in values {
        ui.collapsing(RichText::new(i.to_string()).strong(), |ui| {
            build_component(ui, value);
        });
    }
}