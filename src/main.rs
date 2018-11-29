use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;

use gtk::{
    BoxExt, ButtonExt, Cast, ContainerExt, DialogExt, FileChooserExt, GtkWindowExt, Inhibit,
    LabelExt, Orientation, SpinButtonExt, WidgetExt, Window, WindowType,
};

struct Alarm {
    main_win: Window,
    selected_file: std::path::PathBuf,
}
impl Alarm {
    fn new() -> Self {
        let main_win = Self::main_win();
        Self {
            main_win,
            selected_file: Default::default(),
        }
    }
    // GUI
    fn main_win() -> Window {
        let main_win = Window::new(WindowType::Toplevel);
        main_win.set_title("Alarm");
        main_win.set_resizable(false);

        let vbox: gtk::Box = Self::insides();
        main_win.add(&vbox);

        main_win.show_all();
        main_win.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });
        main_win
    }
    fn create_vbox<L: gtk::IsA<gtk::Widget>, B: gtk::IsA<gtk::Widget> + gtk::OrientableExt>(
        widgets: (&L, &B),
    ) -> gtk::Box {
        widgets.1.set_orientation(Orientation::Vertical);

        let vbox = gtk::Box::new(Orientation::Vertical, 0);
        vbox.pack_start(widgets.0, true, true, 0);
        vbox.pack_start(widgets.1, true, true, 0);

        vbox
    }

    fn label_with_markup(label_text: &str) -> gtk::Label {
        let label = gtk::Label::new(None);
        label.set_markup(&format!("<b>{}</b>", label_text));
        label.set_max_width_chars(80);
        label
    }
    fn create_timer() -> Vec<gtk::SpinButton> {
        let mut vec = Vec::new();
        for i in &[24.0, 60.0, 60.0] {
            let btn = gtk::SpinButton::new_with_range(0.0, *i, 1.0);
            vec.push(btn);
        }
        vec
    }
    fn insides() -> gtk::Box {
        let spin_btns_vec = Self::create_timer();
        let hours = Self::create_vbox((&Self::label_with_markup("H"), &spin_btns_vec[0]));
        let minutes = Self::create_vbox((&Self::label_with_markup("M"), &spin_btns_vec[1]));
        let seconds = Self::create_vbox((&Self::label_with_markup("S"), &spin_btns_vec[2]));
        let hbox = gtk::Box::new(Orientation::Horizontal, 10);
        hbox.pack_start(&hours, true, true, 10);
        hbox.pack_start(&minutes, true, true, 10);
        hbox.pack_start(&seconds, true, true, 10);

        let done_button = gtk::Button::new_with_label("Done");

        let vbox = gtk::Box::new(Orientation::Vertical, 10);
        vbox.pack_start(&Self::select_file(), false, false, 0);
        vbox.pack_start(&hbox, true, true, 10);
        vbox.pack_start(&done_button, true, true, 10);

        vbox.set_margin_top(10);
        vbox
    }

    fn execute_file(selected_file: &std::path::PathBuf) {
        if cfg!(target_os = "linux") {
            std::process::Command::new("xdg-open")
                .arg(&selected_file)
                .spawn()
                .expect("Error opening desired file");
        } else if cfg!(target_os = "windows") {
            std::process::Command::new("cmd")
                .args(&["/C", "start", selected_file.to_str().unwrap()])
                .spawn()
                .expect("Error opening desired file");
        } else {
            //Hmmm
        }
    }

    fn select_file() -> gtk::Box {
        let label = gtk::Label::new(None);
        label.set_markup("<b>Set the timer and choose a program to execute</b>");
        label.set_line_wrap(true);
        label.set_max_width_chars(20);

        let btn = gtk::Button::new_from_icon_name("list-add-symbolic", 5);

        let hbox = gtk::Box::new(Orientation::Horizontal, 0);
        hbox.pack_start(&label, true, true, 10);
        hbox.pack_start(&btn, false, false, 10);
        hbox
    }

    // Connect Callbacks
    fn connect_all(self) {
        let mut vbox = self.main_win.get_children();
        let vbox = vbox
            .pop()
            .unwrap()
            .downcast::<gtk::Box>()
            .expect("Dunno what happened");

        let btn = vbox
            .get_children()
            .pop()
            .unwrap()
            .downcast::<gtk::Button>()
            .expect("Dunno what happened");

        let hbox = vbox
            .get_children()
            .into_iter()
            .nth(1)
            .unwrap()
            .downcast::<gtk::Box>()
            .unwrap();

        let add_btn_box = vbox
            .get_children()
            .into_iter()
            .nth(0)
            .unwrap()
            .downcast::<gtk::Box>()
            .unwrap();
        let add_btn = add_btn_box
            .get_children()
            .into_iter()
            .nth(1)
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap();

        let alarm_ref1 = Rc::new(RefCell::new(self));
        let alarm_ref2 = alarm_ref1.clone();
        let alarm_ref4 = alarm_ref1.clone();
        add_btn.connect_clicked(move |_btn| {
            let alarm_ref3 = alarm_ref2.clone();
            let dialog: gtk::FileChooserDialog =
                gtk::FileChooserDialog::with_buttons::<gtk::FileChooserDialog>(
                    Some("Choose file"),
                    None,
                    gtk::FileChooserAction::Open,
                    &[
                        ("_Cancel", gtk::ResponseType::Cancel),
                        ("_Select", gtk::ResponseType::Accept),
                    ],
                );
            dialog.connect_response(move |dlg, id| {
                if id == -3 {
                    let alarm_ref3: &RefCell<Alarm> = alarm_ref3.borrow();
                    alarm_ref3.borrow_mut().selected_file =
                        dlg.get_filename().expect("Can't find selected file");
                };

                dlg.close();
            });
            dialog.show();
        });

        gtk::timeout_add_seconds(1, move || {
            let alarm_ref4: &RefCell<Alarm> = alarm_ref4.borrow();
            let alarm_ref4 = alarm_ref4.borrow();
            if alarm_ref4.selected_file.exists() {
                let image =
                    gtk::Image::new_from_icon_name("view-refresh", 5).upcast::<gtk::Widget>();
                add_btn.set_image::<gtk::Widget, &Option<gtk::Widget>>(&Some(image));
                return gtk::Continue(false);
            };
            gtk::Continue(true)
        });

        btn.connect_clicked(move |_btn| {
            let alarm_ref2 = alarm_ref1.clone();
            let alarm_ref1: &RefCell<Alarm> = alarm_ref1.borrow();
            let alarm_ref1 = alarm_ref1.borrow();
            if alarm_ref1.selected_file.exists() {
                alarm_ref1.main_win.hide();

                let mut time = 0.0;
                let fact = [3600.0, 60.0, 1.0];
                for (idx, child) in hbox.get_children().into_iter().enumerate() {
                    let child = child.downcast::<gtk::Box>().unwrap();
                    let v = child
                        .get_children()
                        .pop()
                        .unwrap()
                        .downcast::<gtk::SpinButton>()
                        .unwrap()
                        .get_value();
                    time += v * fact[idx];
                }

                let time = time as u64;
                gtk::idle_add(move || {
                    let alarm_ref2: &RefCell<Alarm> = alarm_ref2.borrow();
                    let alarm_ref2 = alarm_ref2.borrow();
                    std::thread::sleep(std::time::Duration::from_secs(time));
                    Alarm::execute_file(&alarm_ref2.selected_file);
                    &alarm_ref2.main_win.show();
                    gtk::Continue(false)
                });
            }
        });
    }
}

fn main() {
    gtk::init().unwrap();

    let alarm = Alarm::new();
    alarm.connect_all();

    gtk::main();
}
