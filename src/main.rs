use std::sync::{Arc, Mutex};

mod inkscape;

mod generated_code {
    slint::include_modules!();
}
pub use generated_code::*;

#[derive(Clone, Debug)]
struct InkscapeArgs {
    file_path: Option<String>,
    export_png: bool,
    export_eps: bool,
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

impl std::fmt::Display for InkscapeArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "inkscape")?;
        match (self.export_png, self.export_pdf, self.export_eps) {
            (true, true, true) => write!(f, " --export-type=png,pdf,eps"),
            (true, true, false) => write!(f, " --export-type=png,pdf"),
            (true, false, true) => write!(f, " --export-type=png,eps"),
            (true, false, false) => write!(f, " --export-type=png"),
            (false, true, true) => write!(f, " --export-type=pdf,eps"),
            (false, true, false) => write!(f, " --export-type=pdf"),
            (false, false, true) => write!(f, " --export-type=eps"),
            (false, false, false) => write!(f, ""),
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

    pub fn pdf(&mut self, enabled: bool) -> &mut Self {
        self.cmd.export_pdf = enabled;
        self
    }

    #[allow(dead_code)]
    pub fn build(&self) -> InkscapeArgs {
        self.cmd.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_to_png() {
        let args = InkscapeArgsBuilder::new().png(true).build();
        assert_eq!("inkscape --export-type=png", format!("{}", args));
    }

    #[test]
    fn export_to_pdf() {
        let args = InkscapeArgsBuilder::new().pdf(true).build();
        assert_eq!("inkscape --export-type=pdf", format!("{}", args));
    }

    #[test]
    fn export_to_eps() {
        let args = InkscapeArgsBuilder::new().eps(true).build();
        assert_eq!("inkscape --export-type=eps", format!("{}", args));
    }

    #[test]
    fn export_to_all() {
        let args = InkscapeArgsBuilder::new()
            .png(true)
            .pdf(true)
            .eps(true)
            .build();
        assert_eq!("inkscape --export-type=png,pdf,eps", format!("{}", args));
    }
}

// fn main() -> Result<(), slint::PlatformError> {
fn main() {
    env_logger::init();
    let ui = AppUI::new().unwrap();
    let inkscape_worker = inkscape::InkscapeWorker::new(&ui);

    let mut inkscape_cmd = InkscapeArgsBuilder::new();
    inkscape_cmd.png(ui.get_export_png()).pdf(ui.get_export_pdf()).eps(ui.get_export_eps());
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
            log::info!("Executing: {}", args.build());
        }
    });

    ui.run().unwrap();
    inkscape_worker.join().unwrap();
}
