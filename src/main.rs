use std::sync::{Arc, Mutex};

slint::include_modules!();

#[derive(Debug)]
struct InkscapeCmd {
    file_path: Option<String>,
    export_png: bool,
    export_eps: bool,
    export_pdf: bool,
}

impl Default for InkscapeCmd {
    fn default() -> Self {
        InkscapeCmd {
            file_path: None,
            export_png: false,
            export_eps: false,
            export_pdf: false,
        }
    }
}

#[derive(Debug)]
struct InkscapeCmdBuilder {
    file_path: Option<String>,
    cmd: InkscapeCmd,
}

impl Default for InkscapeCmdBuilder {
    fn default() -> Self {
        InkscapeCmdBuilder {
            file_path: None,
            cmd: InkscapeCmd::default(),
        }
    }
}

impl InkscapeCmdBuilder {
    pub fn new() -> Self {
        InkscapeCmdBuilder::default()
    }

    // pub fn file(mut self, file_path: &str) -> Self {
    //     self.cmd.file_path = Some(file_path.to_owned());
    //     self
    // }

    pub fn png(&mut self, enabled: bool) -> &mut Self {
        self.cmd.export_png = enabled;
        self
    }

    pub fn eps(&mut self, enabled: bool) -> &mut Self {
        self.cmd.export_eps = enabled;
        self
    }

    // pub fn pdf(mut self, enabled: bool) -> InkscapeCmdBuilder {
    //     self.cmd.export_pdf = enabled;
    //     self
    // }

    pub fn build(self) -> InkscapeCmd {
        self.cmd
    }
}

fn main() -> Result<(), slint::PlatformError> {
    env_logger::init();
    let ui = AppWindow::new()?;
    let inkscape_cmd = InkscapeCmdBuilder::default();
    let cmd_arc = Arc::new(Mutex::new(inkscape_cmd));

    ui.on_toggle_export_png({
        // let ui_handle = ui.as_weak();
        let x = cmd_arc.clone();
        move |enabled| {
            // let ui = ui_handle.unwrap();
            // let x = arc_clone.clone();
            let mut y = x.lock().unwrap();
            let z = &y.png(enabled);
            log::debug!("{:?}", z);
            // println!("PNG: {:?}", z);
        }
    });
    ui.on_toggle_export_eps({
        // let ui_handle = ui.as_weak();
        let x = cmd_arc.clone();
        move |enabled| {
            // let x = arc_clone.clone();
            let mut y = x.lock().unwrap();
            let z = &y.eps(enabled);
            log::debug!("{:?}", z);
            // println!("PNG: {:?}", z);
        }
    });
    ui.on_show_folder_dialog({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            let mut dialog = rfd::FileDialog::new();
            dialog = dialog.set_title("Select output folder");
            let folder = match dialog.pick_folder() {
                Some(folder) => folder.display().to_string().into(),
                None => "".into(),
            };
            ui.set_output_dir(folder);
        }
    });

    ui.run()
}
