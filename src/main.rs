use std::sync::{Arc, Mutex};

slint::include_modules!();

#[derive(Debug)]
struct InkscapeArgs {
    #[allow(dead_code)]
    file_path: Option<String>,
    export_png: bool,
    export_eps: bool,
    #[allow(dead_code)]
    export_pdf: bool,
}

impl Default for InkscapeArgs {
    fn default() -> Self {
        InkscapeArgs {
            file_path: None,
            export_png: false,
            export_eps: false,
            export_pdf: false,
        }
    }
}

#[derive(Debug)]
struct InkscapeArgsBuilder {
    #[allow(dead_code)]
    file_path: Option<String>,
    cmd: InkscapeArgs,
}

impl Default for InkscapeArgsBuilder {
    fn default() -> Self {
        InkscapeArgsBuilder {
            file_path: None,
            cmd: InkscapeArgs::default(),
        }
    }
}

impl InkscapeArgsBuilder {
    pub fn new() -> Self {
        InkscapeArgsBuilder::default()
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

    // pub fn pdf(mut self, enabled: bool) -> InkscapeArgsBuilder {
    //     self.cmd.export_pdf = enabled;
    //     self
    // }

    // pub fn build(self) -> InkscapeArgs {
    //     self.cmd
    // }
}

fn main() -> Result<(), slint::PlatformError> {
    env_logger::init();
    let ui = AppWindow::new()?;
    let inkscape_cmd = InkscapeArgsBuilder::new();
    let cmd_arc = Arc::new(Mutex::new(inkscape_cmd));

    ui.on_toggle_export_png({
        let inkscape_args = cmd_arc.clone();
        move |enabled| {
            let mut inkscape_args = inkscape_args.lock().unwrap();
            let z = &inkscape_args.png(enabled);
            log::debug!("{:?}", z);
        }
    });
    ui.on_toggle_export_eps({
        let inkscape_args = cmd_arc.clone();
        move |enabled| {
            let mut inkscape_args = inkscape_args.lock().unwrap();
            let z = &inkscape_args.eps(enabled);
            log::debug!("{:?}", z);
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
            // ui.set_root_directory(folder);
        }
    });
    ui.on_execute_inkscape({
        let inkscape_args = cmd_arc.clone();
        move || {
            let inkscape_args = inkscape_args.lock().unwrap();
            let args = &inkscape_args;
            log::info!("Executing: {:?}", args);
        }
    });

    ui.run()
}
