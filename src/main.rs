// mod generators;
// mod llmgenerator;

extern crate gtk;

use anyhow::{Result, Error, Ok};
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, HeaderBar, Box, Entry, ScrolledWindow, TextView, TextBuffer, ComboBoxText, Orientation, Button, ReliefStyle, Adjustment, Label, SpinButton, Switch, ListBox, Popover, gdk, EntryBuffer, TextTagTable};
use gdk::{keys::constants as key};
extern crate glib;
use glib::{num_processors};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::fs;
use std::string::String;
use std::io::{self, Write};
use std::thread;
use glib::idle_add;
use futures::future::Future;
use serde_json::{Value};
use reqwest::blocking::Client;
use std::rc::Rc;
use std::cell::RefCell;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct APIResponse {
    models: Vec<Model>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Model {
    name: String,
}

fn build_ui(application: &Application) {
    // Define window attributes
    let window = ApplicationWindow::new(application);
    window.set_title("Chat with AI");
    window.set_default_size(720, 400);
    // prep headerbar
    let header = HeaderBar::new();
    header.set_title(Some("AI Models"));
    header.set_show_close_button(true);
    // create combobox for headerbar
    let model_combo = ComboBoxText::new();
    model_combo.set_hexpand(false);
    let bin_files = enumerate_models();
    model_combo.set_size_request(50, -1);
    for model in enumerate_models() {
        model_combo.append_text(&model);
    }

    model_combo.set_active(Some(0));
    header.pack_start(&model_combo);
    header.set_hexpand(false);
    // initialize main box
    let main_box = Box::new(Orientation::Vertical, 5);
    main_box.set_margin_top(0);
    main_box.set_margin_bottom(0);
    main_box.set_margin_start(0);
    main_box.set_margin_end(0);

    // initialize scrolling container
    let scrolled_window = ScrolledWindow::new(None::<&Adjustment>, None::<&Adjustment>);
    scrolled_window.set_hexpand(true);
    scrolled_window.set_vexpand(true);
    // create textview and buffer for scrolling window
    let text_buffer = TextBuffer::new(Some(&TextTagTable::new()));
    let text_view = TextView::new();
    text_view.set_wrap_mode(gtk::WrapMode::Word);
    text_view.set_editable(true);
    text_view.set_border_width(10);
    text_view.set_pixels_below_lines(5);
    // initialize text buffer with neccesary attributes
    let mut buffer = TextBuffer::new(None::<&gtk::TextTagTable>);
    text_view.set_buffer(Some(&buffer));
    buffer.set_text("How can I help you today?");
    // glue it all together
    scrolled_window.add(&text_view);
    main_box.pack_start(&scrolled_window, true, true, 0);
    // create entry box
    let entry_buffer = EntryBuffer::new(None);
    let entry = Entry::with_buffer(&entry_buffer);
    entry.set_margin_bottom(5);
    entry.set_margin_top(0);
    entry.set_margin_start(5);
    entry.set_margin_end(5);
    // put entry box at bottom of app window
    main_box.pack_start(&entry, false, false, 0);
    // initialize send button for headerbar
    let send_button = Button::with_label("Send");
    send_button.set_relief(ReliefStyle::None);
    send_button.connect_clicked(glib::clone!(@weak entry => move |_| {
        let temp_entry = entry.clone();
        temp_entry.activate(); // Simulate Enter key press
        temp_entry.grab_focus();
    }));
    header.pack_end(&send_button);
    // initialize settings button for headerbar
    let settings_button = Button::with_label("Settings");
    settings_button.set_relief(ReliefStyle::None);
    header.pack_end(&settings_button);
    // display popover window when clickking settings
    let popover = Popover::new(Some(&settings_button));
    popover.set_position(gtk::PositionType::Bottom);
    // oh boy get ready
    let settings_list = ListBox::new();
    settings_list.set_margin_top(10);
    settings_list.set_margin_bottom(10);
    settings_list.set_margin_start(10);
    settings_list.set_margin_end(10);
    settings_list.set_border_width(5);
    // WARNING !!! SHIT IS FUCKED PAST THIS POINT !!! YOU HAVE BEEN WARNED
    // Initial Prompt Entry
    let prompt_label = Label::new(Some("Initial prompt entry:"));
    let prompt_buffer = EntryBuffer::new(None);
    let prompt_entry = Entry::with_buffer(&prompt_buffer);
    prompt_entry.set_text("I am an AI chatbot, trained on large amounts of human knowledge to answer your every question. You asked me:");
    let prompt_row = gtk::Box::new(Orientation::Horizontal, 5);
    prompt_row.pack_start(&prompt_label, false, false, 0);
    prompt_row.pack_start(&prompt_entry, true, true, 0);
    settings_list.add(&prompt_row);
    // Maximum Generation Length
    let length_label = Label::new(Some("Maximum generation length:"));
    let length_adjustment = Adjustment::new(64.0, 0.0, 2048.0, 1.0, 1.0, 0.0);
    let length_spin_button = SpinButton::new(Some(&length_adjustment), 1.0, 0);
    length_spin_button.set_numeric(true);
    let length_row = gtk::Box::new(Orientation::Horizontal, 5);
    length_row.pack_start(&length_label, false, false, 0);
    length_row.pack_start(&length_spin_button, false, false, 0);
    settings_list.add(&length_row);
    // Toggles for Sampling and Early Stopping
    let sampling_switch = Switch::new();
    let sampling_label = Label::new(Some("Sampling"));
    let sampling_row = gtk::Box::new(Orientation::Horizontal, 5);
    sampling_row.pack_start(&sampling_switch, false, false, 0);
    sampling_row.pack_start(&sampling_label, false, false, 0);
    settings_list.add(&sampling_row);
    // Newline penalization Switch
    let stopping_switch = Switch::new();
    let stopping_label = Label::new(Some("Penalize Newlines"));
    let stopping_row = gtk::Box::new(Orientation::Horizontal, 5);
    stopping_row.pack_start(&stopping_switch, false, false, 0);
    stopping_row.pack_start(&stopping_label, false, false, 0);
    settings_list.add(&stopping_row);
    // Temperature Float
    let temperature_label = Label::new(Some("Temperature float:"));
    let temperature_adjustment = Adjustment::new(0.7, 0.0, 100.0, 0.1, 1.0, 0.0);
    let temperature_spin_button = SpinButton::new(Some(&temperature_adjustment), 0.1, 1);
    temperature_spin_button.set_numeric(true);
    let temperature_row = gtk::Box::new(Orientation::Horizontal, 5);
    temperature_row.pack_start(&temperature_label, false, false, 0);
    temperature_row.pack_start(&temperature_spin_button, false, false, 0);
    settings_list.add(&temperature_row);
    // Beam Number (1-16)
    let beam_label = Label::new(Some("Beam count (1-16):"));
    let beam_adjustment = Adjustment::new(4.0, 1.0, 16.0, 1.0, 1.0, 0.0);
    let beam_spin_button = SpinButton::new(Some(&beam_adjustment), 1.0, 0);
    beam_spin_button.set_numeric(true);
    let beam_row = gtk::Box::new(Orientation::Horizontal, 5);
    beam_row.pack_start(&beam_label, false, false, 0);
    beam_row.pack_start(&beam_spin_button, false, false, 0);
    settings_list.add(&beam_row);
    // Thread count
    let thread_limit = num_processors();
    let thread_label = Label::new(Some(&format!("Thread limit (1-{})", thread_limit)));
    let thread_adjustment = Adjustment::new((&thread_limit / 2) as f64, 1.0, thread_limit as f64, 1.0, 1.0, 0.0);
    let thread_spin_button = SpinButton::new(Some(&thread_adjustment), 1.0, 0);
    thread_spin_button.set_numeric(true);
    let thread_row = gtk::Box::new(Orientation::Horizontal, 5);
    thread_row.pack_start(&thread_label, false, false, 0);
    thread_row.pack_start(&thread_spin_button, false, false, 0);
    settings_list.add(&thread_row);
    // add that shit to the popover
    popover.add(&settings_list);
    // tie it together
    settings_button.connect_clicked(move |_| {
        popover.show_all();
    });

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    // vbox.pack_start(&header, false, false, 0);
    vbox.pack_start(&main_box, true, true, 0);

    // on enter-press of the entrybox
    entry.connect_activate(glib::clone!(@weak entry => move |_| {
        let text = format!("{:?}\n{}", buffer.text(&buffer.start_iter(), &buffer.end_iter(), true).unwrap(), entry_buffer.text());
        let init = prompt_buffer.text();
        let max = length_adjustment.value() as u16;
        let sampling = sampling_switch.state();
        let stopping = stopping_switch.state();
        let temp = temperature_adjustment.value() as f32;
        let beams = beam_adjustment.value() as u8;
        let model_name = std::string::String::from(model_combo.active_text().unwrap());
        let num_cpus = thread_adjustment.value() as i32;
        println!("habbening =D {}", &text);
        // Get an iterator pointing to the end of the buffer
        let mut end_iter = buffer.end_iter();
        // Convert the iterator to a mutable iterator
        let mut end_iter_mut = end_iter.clone();
        buffer.insert(&mut end_iter_mut, &format!("\n{}", &entry_buffer.text()));
        // insert ebic threading code here ( you know ;) )
        println!("Working (infer stage): {:?}", &model_name);

        let home_dir = dirs::home_dir().unwrap();
        let ai_dir = home_dir.join(".ai");
        let model_path = ai_dir.join(&model_name);



        // clear entry buffer
        entry_buffer.set_text("");
        // Add to text buffer
        let msg = "placeholder";
        buffer.insert(&mut end_iter_mut, &msg);
        println!("{:?}", msg);
    }));

    window.add(&vbox);
    // ctrl + c close
    window.connect_key_press_event(|_, event| {
        if let Some(key) = event.keyval().into() {
            if event.state().contains(gdk::ModifierType::CONTROL_MASK) && key == key::q {
                gtk::main_quit();
                Inhibit(true);
            }
        }
        Inhibit(false)
    });
    // ctrl + enter for infer
    window.connect_key_press_event(move |_, event| {
        if let Some(key) = event.keyval().into() {
            if event.state().contains(gdk::ModifierType::CONTROL_MASK) && key == key::Return {
                entry.activate();
                Inhibit(true);
            }
        }
        Inhibit(false)
    });

    window.set_titlebar(Some(&header));
    window.show_all();
}

fn enumerate_models() -> Vec<String> {
    let response_text = reqwest::blocking::get("http://localhost:11434/api/tags").unwrap().text().unwrap();

    let api_response: APIResponse = serde_json::from_str(&response_text).unwrap();

    api_response.models.iter().map(|model| model.name.clone()).collect()
}

async fn llm_generate(model: &str, prompt: &str, init: &str, max: u16, sampling: bool, stopping: bool, temp: f32, beams: u8) -> Result<String, anyhow::Error> {
    let response = "uhm..... cheesed to meet you?";

    Ok(response.to_string())
}

fn main() {
    let application = Application::builder()
        .application_id("com.toaster.gubert")
        .build();

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run();
}
