
//! # PMDR
//!
//! A GTK+ Pomodoro application with "snazzy" reporting

extern crate pmdr;
extern crate gio;
extern crate gtk;
extern crate notify_rust;

use notify_rust::{Notification, NotificationHandle};
use gio::prelude::*;
use gtk::prelude::*;


use std::rc::Rc;
use std::cell::RefCell;

const NOTIFICATION_SOUND: &'static str = "complete";

fn build_ui(app: &gtk::Application,
            pomodoro: Rc<RefCell<pmdr::PMDRApp>>,
            notification: Rc<RefCell<NotificationHandle>>)
{
    let win = gtk::ApplicationWindow::new(app);

    win.set_title("PMDR");
    win.set_border_width(10);
    win.set_position(gtk::WindowPosition::Mouse);
    win.set_default_size(200,180);

    win.connect_delete_event(move |w,_| {
        w.destroy();
        Inhibit(false)
    });

    let vbox = gtk::Box::new(gtk::Orientation::Vertical,  8);

    let state_label = gtk::Label::new(None);
    state_label.set_text(&pomodoro.borrow().state_label());
    vbox.add(&state_label);

    let countdown_label = gtk::Label::new(None);

    countdown_label.set_text(&pomodoro.borrow().countdown_string());
    vbox.add(&countdown_label);

    let tally_label = gtk::Label::new(None);

    tally_label.set_text(&format!("Tally: {}", 0));
    vbox.add(&tally_label);

    let pause_button = gtk::Button::new_with_label("Pause");
    let button_pomodoro = pomodoro.clone();

    pause_button.connect_clicked(move |_btn| {
        button_pomodoro.borrow_mut().toggle_timer();
    });

    vbox.add(&pause_button);

    let stop_button = gtk::Button::new_with_label("Stop");
    let stop_button_pomodoro = pomodoro.clone();

    stop_button.connect_clicked(move |btn| {
        stop_button_pomodoro.borrow_mut().stop();
        btn.set_label("Reset Tally");
    });

    vbox.add(&stop_button);

    win.add(&vbox);
    win.show_all();

    let tick = move || {
        let state_changed = pomodoro.borrow_mut().tick();

        // set button labels
        if pomodoro.borrow().ticking() {
            pause_button.set_label("Pause");
            stop_button.set_label("Stop");
        } else {
            pause_button.set_label("Play");
        }

        // update labels
        state_label.set_text(&pomodoro.borrow().state_label());
        countdown_label.set_text(&pomodoro.borrow().countdown_string());
        tally_label.set_text(&format!("Tally: {}", pomodoro.borrow().tally()));

        if state_changed {
            let on_break = pomodoro.borrow().on_break();
            if on_break {
                notification.borrow_mut().summary("TIME FOR A BREAK").timeout(10);
            } else {
                // pause until the user clicks play.
                pomodoro.borrow_mut().toggle_timer();
                notification.borrow_mut().summary("HEY, GET BACK TO WORK!").timeout(0);
            }

            notification.borrow_mut().update();
        }
        gtk::Continue(true)
    };

    gtk::timeout_add_seconds(1, tick);
}


fn main() {

    let app = gtk::Application::new("games.coolmedium.pmdr", gio::ApplicationFlags::empty())
        .expect("Failed to initialize GTK Application");

    app.connect_startup( move |app0| {
        let pomodoro = Rc::new(RefCell::new(pmdr::PMDRApp::new()));
        let notification = Notification::new()
            .summary("Get to it!")
            .timeout(10)
            .sound_name(NOTIFICATION_SOUND)
            .show()
            .unwrap();
        let notification = Rc::new(RefCell::new(notification));

        build_ui(app0, pomodoro, notification);
    });

    app.connect_activate(|_| {});
    app.run(&vec![]);
}
