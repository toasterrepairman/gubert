mod generators;

extern crate gtk;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, HeaderBar, Box, Entry, ScrolledWindow, TextView, TextBuffer, TextIter, ComboBoxText, Orientation, Button, ReliefStyle, Adjustment, Label, SpinButton, Switch, ListBox, Popover, gdk, EntryBuffer, TextTagTable};
use gdk::{keys::constants as key, EventKey};
use crate::generators::gptneo_generate;
use tokio::runtime::Runtime;
use tokio::runtime::Handle;
use tokio::sync::Mutex;
use std::sync::Arc;
use tokio::task::spawn_local;

use std::rc::Rc;
use std::cell::RefCell;

fn build_ui(application: &Application) {
    let window = ApplicationWindow::new(application);
    window.set_title("Chat with AI");
    window.set_default_size(500, 400);

    let header = HeaderBar::new();
    header.set_title(Some("AI Models"));

    let model_combo = ComboBoxText::new();
    model_combo.append_text("Model 1");
    model_combo.append_text("Model 2");
    // Add more AI models as needed

    header.pack_start(&model_combo);

    let main_box = Box::new(Orientation::Vertical, 5);
    main_box.set_margin_top(10);
    main_box.set_margin_bottom(10);
    main_box.set_margin_start(10);
    main_box.set_margin_end(10);

    let scrolled_window = ScrolledWindow::new(None::<&Adjustment>, None::<&Adjustment>);
    scrolled_window.set_hexpand(true);
    scrolled_window.set_vexpand(true);

    let text_buffer = TextBuffer::new(Some(&TextTagTable::new()));
    let text_view = TextView::new();
    text_view.set_wrap_mode(gtk::WrapMode::Word);
    text_view.set_editable(false);

    let buffer = TextBuffer::new(None::<&gtk::TextTagTable>);
    text_view.set_buffer(Some(&buffer));

    scrolled_window.add(&text_view);
    main_box.pack_start(&scrolled_window, true, true, 0);

    let entry_buffer = EntryBuffer::new(None);
    let entry = Entry::with_buffer(&entry_buffer);
    let runtime = Rc::new(Runtime::new().unwrap());

    main_box.pack_start(&entry, false, false, 0);

    let runtime = Arc::new(Mutex::new(tokio::runtime::Runtime::new().unwrap()));

    entry.connect_activate(move |_| {
        let entry_text = entry_buffer.text();
        let runtime = runtime.clone();

        spawn_local(async move {
            let result = gptneo_generate().await;
            // Process the result as needed
            if let Err(e) = result {
                eprintln!("Error generating text: {}", e);
            }
        });
    });

    let send_button = Button::with_label("Send");
    send_button.set_relief(ReliefStyle::None);

    send_button.connect_clicked(move |_| {
        let temp_entry = entry.clone();
        temp_entry.activate(); // Simulate Enter key press
        temp_entry.grab_focus();
    });

    header.pack_end(&send_button);

    let settings_button = Button::with_label("Settings");
    settings_button.set_relief(ReliefStyle::None);
    header.pack_end(&settings_button);

    let popover = Popover::new(Some(&settings_button));
    popover.set_position(gtk::PositionType::Bottom);

    let settings_list = ListBox::new();
    settings_list.set_margin_top(10);
    settings_list.set_margin_bottom(10);
    settings_list.set_margin_start(10);
    settings_list.set_margin_end(10);

    // Initial Prompt Entry
    let prompt_label = Label::new(Some("Initial prompt entry:"));
    let prompt_entry = Entry::new();
    prompt_entry.set_text("A dialog, where User interacts with AI. AI is helpful, kind, honest, and knows its own limits.");
    let prompt_row = gtk::Box::new(Orientation::Horizontal, 5);
    prompt_row.pack_start(&prompt_label, false, false, 0);
    prompt_row.pack_start(&prompt_entry, true, true, 0);
    settings_list.add(&prompt_row);

    // Maximum Generation Length
    let length_label = Label::new(Some("Maximum generation length:"));
    let length_adjustment = Adjustment::new(256.0, 0.0, 2048.0, 1.0, 1.0, 0.0);
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

    let stopping_switch = Switch::new();
    let stopping_label = Label::new(Some("Early Stopping"));
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

    popover.add(&settings_list);

    settings_button.connect_clicked(move |_| {
        popover.show_all();
    });

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.pack_start(&header, false, false, 0);
    vbox.pack_start(&main_box, true, true, 0);

    window.add(&vbox);
    window.connect_key_press_event(|_, event| {
        if let Some(key) = event.keyval().into() {
            if event.state().contains(gdk::ModifierType::CONTROL_MASK) && key == key::q {
                gtk::main_quit();
                Inhibit(true);
            }
        }
        Inhibit(false)
    });

    window.show_all();
}

fn main() {
    let application = Application::builder()
        .application_id("com.example.chat_ai")
        .build();

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run();
}
