use lazy_static::*;

use directories::ProjectDirs;
use gtk::{
    BoxExt, ButtonExt, Cast, ContainerExt, DialogExt, FileChooserExt, GtkWindowExt, Inhibit,
    LabelExt, Orientation, SpinButtonExt, WidgetExt, Window, WindowType,
};
use std::fs;

use daemonize::Daemonize;

lazy_static! {
    static ref CONFIG_DIR: std::path::PathBuf = ProjectDirs::from("", "", "Alarm")
        .expect("Cant locate home folder")
        .config_dir()
        .to_path_buf();
}

struct Alarm {
    main_win: Window,
}
impl Alarm {
    fn new() -> Self {
        let main_win = Self::main_win();
        Self { main_win }
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
        vbox.pack_start(&Self::action_select(), false, false, 0);
        vbox.pack_start(&hbox, true, true, 10);
        vbox.pack_start(&done_button, true, true, 10);

        vbox.set_margin_top(10);
        vbox
    }

    fn on_done_clicked(_but: &gtk::Button, selected_file: String) {
        let daemonize = Daemonize::new().privileged_action(move || {
            std::process::Command::new("xdg-open")
                .arg(&selected_file)
                .spawn()
                .expect("Error opening desired file");
        });

        match daemonize.start() {
            Ok(_) => std::process::exit(0),
            Err(e) => eprintln!("Error, {}", e),
        }
    }

    fn action_select() -> gtk::Box {
        let label = gtk::Label::new(None);
        label.set_markup("<b>Set the timer and choose a program to execute</b>");
        label.set_line_wrap(true);
        label.set_max_width_chars(20);

        let btn = gtk::Button::new_from_icon_name("list-add-symbolic", 5);
        btn.connect_clicked(|_btn| {
            let dialog: gtk::FileChooserDialog =
                gtk::FileChooserDialog::with_buttons::<gtk::FileChooserDialog>(
                    Some("Choose music"),
                    None,
                    gtk::FileChooserAction::Open,
                    &[
                        ("_Cancel", gtk::ResponseType::Cancel),
                        ("_Select", gtk::ResponseType::Accept),
                    ],
                );
            dialog.connect_response(|dlg, id| {
                if id == -3 {
                    Self::save_audio(
                        &dlg.get_filename()
                            .expect("Can't find config file anymore")
                            .to_path_buf(),
                    );
                };

                dlg.close();
            });
            dialog.show();
        });

        let hbox = gtk::Box::new(Orientation::Horizontal, 0);
        hbox.pack_start(&label, true, true, 10);
        hbox.pack_start(&btn, false, false, 10);
        hbox
    }

    fn save_audio(selected_file: &std::path::PathBuf) {
        let config_file = get_config_file();
        let selected_file = selected_file
            .to_str()
            .expect("Error while reading audio file path");
        let contents = format!("Audio: {}", selected_file);
        fs::write(config_file, contents).expect("Error while writing to config file");
    }

    fn get_audio() -> Option<String> {
        let config_file = get_config_file();
        let contents = fs::read_to_string(&config_file).expect("Error while reading config file");
        contents.find("Audio")?;
        Some(String::from(
            contents
                .split(": ")
                .nth(1)
                .expect("Config file format error"),
        ))
    }
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
        gtk::timeout_add_seconds(1, move || {
            if Alarm::get_audio().is_some() {
                let image =
                    gtk::Image::new_from_icon_name("view-refresh", 5).upcast::<gtk::Widget>();
                add_btn.set_image::<gtk::Widget, &Option<gtk::Widget>>(&Some(image));
                return gtk::Continue(false);
            };
            gtk::Continue(true)
        });

        btn.connect_clicked(move |_btn| {
            let selected_file = match Self::get_audio() {
                Some(file) => file,
                None => return,
            };

            self.main_win.hide();

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
                std::thread::sleep(std::time::Duration::from_secs(time));
                Alarm::on_done_clicked(&gtk::Button::new(), selected_file.clone());
                gtk::Continue(false)
            });
        });
    }
}

fn get_config_file() -> std::path::PathBuf {
    let mut config_file = CONFIG_DIR.clone();
    config_file.push("config_file");
    config_file
}

fn setup_env() {
    fs::create_dir_all(CONFIG_DIR.as_path()).expect("Error while creating config dir");
    let config_file = get_config_file();
    match fs::File::open(&config_file) {
        Ok(_) => write_config(&config_file),
        Err(_) => write_config(&config_file),
    };
}

fn write_config(path: &std::path::PathBuf) {
    fs::File::create(path).expect("Error while creating config file");
}

fn main() {
    setup_env();

    gtk::init().unwrap();

    let alarm = Alarm::new();
    alarm.connect_all();
    
    gtk::main();
}

//Commands: linux:xdg-open win:start mac:open
