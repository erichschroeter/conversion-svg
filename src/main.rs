use std::{path::PathBuf, sync::{Arc, Mutex}};

mod inkscape;

mod generated_code {
    slint::include_modules!();
}
pub use generated_code::*;
use regex::Regex;

#[derive(Debug)]
pub struct InkscapeCmd {
    exe_path: PathBuf,
    args: InkscapeArgs,
}

impl InkscapeCmd {
    pub fn new(exe_path: PathBuf, args: InkscapeArgs) -> Self {
        Self { exe_path, args }
    }

    pub fn as_command(&self) -> std::process::Command {
        let mut cmd = std::process::Command::new(&self.exe_path);
        match (self.args.export_png, self.args.export_pdf, self.args.export_eps) {
            (true, true, true)   => cmd.arg("--export-type=png,pdf,eps"),
            (true, true, false)  => cmd.arg("--export-type=png,pdf"),
            (true, false, true)  => cmd.arg("--export-type=png,eps"),
            (true, false, false) => cmd.arg("--export-type=png"),
            (false, true, true)  => cmd.arg("--export-type=pdf,eps"),
            (false, true, false) => cmd.arg("--export-type=pdf"),
            (false, false, true) => cmd.arg("--export-type=eps"),
            (false, false, false) => cmd.arg(""),
        };
        if let Some(input_path) = &self.args.file_path_input {
            cmd.arg(input_path);
        }
        cmd
    }

    pub fn exec(&self) {
        let mut cmd = self.as_command();
        log::debug!("RUNNING!");
        cmd.spawn().expect("Failed to execute inkscape");
    }
}

impl From<std::process::Command> for InkscapeCmd {
    fn from(cmd: std::process::Command) -> Self {
        let exe_path = cmd.get_program().to_owned();
        let args = cmd.get_args().into();
        Self { exe_path: exe_path.into(), args }
    }
}

#[derive(Clone, Debug)]
pub struct InkscapeArgs {
    file_path_input: Option<String>,
    // file_path_output: Option<String>,
    export_png: bool,
    export_eps: bool,
    export_pdf: bool,
}

impl Default for InkscapeArgs {
    fn default() -> Self {
        InkscapeArgs {
            file_path_input: None,
            // file_path_output: None,
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

impl<'a> From<std::process::CommandArgs<'a>> for InkscapeArgs {
    fn from(args: std::process::CommandArgs) -> Self {
        let re_export_type = Regex::new(r"--export-type=(?<types>.*)").unwrap();
        let mut cmd = InkscapeArgs::default();
        for arg in args {
            if let Some(caps) = re_export_type.captures(arg.to_str().unwrap()) {
                let types = caps.name("types").unwrap().as_str();
                for t in types.split(",") {
                    match t {
                        "png" => cmd.export_png = true,
                        "eps" => cmd.export_eps = true,
                        "pdf" => cmd.export_pdf = true,
                        _ => {}
                    }
                }
            }
        }
        cmd
    }
}

#[derive(Debug)]
pub struct InkscapeArgsBuilder {
    #[allow(dead_code)]
    file_path_input: Option<String>,
    cmd: InkscapeArgs,
}

impl Default for InkscapeArgsBuilder {
    fn default() -> Self {
        InkscapeArgsBuilder {
            file_path_input: None,
            cmd: InkscapeArgs::default(),
        }
    }
}

impl InkscapeArgsBuilder {
    pub fn new() -> Self {
        InkscapeArgsBuilder::default()
    }

    pub fn input_file(mut self, file_path_input: &str) -> Self {
        self.cmd.file_path_input = Some(file_path_input.to_owned());
        self
    }

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

    #[test]
    fn from_command_args_with_export_type_png() {
        let mut cmd = std::process::Command::new("inkscape");
        cmd.arg("--export-type=png");
        let args = InkscapeArgs::from(cmd.get_args());
        assert!(args.export_png);
    }

    #[test]
    fn from_command_args_with_export_type_png_and_pdf() {
        let mut cmd = std::process::Command::new("inkscape");
        cmd.arg("--export-type=eps,png,pdf");
        let args = InkscapeArgs::from(cmd.get_args());
        assert!(args.export_png);
        assert!(args.export_pdf);
        assert!(args.export_eps);
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
        let inkscape_tx = inkscape_worker.channel.clone();
        // let inkscape_args = cmd_arc.clone();
        move || {
            // let inkscape_args = inkscape_args.lock().unwrap();
            // let args = &inkscape_args;
            log::info!("Sending InkscapeMessage::Export");
            inkscape_tx.send(inkscape::InkscapeMessage::Export).unwrap()
        }
    });

    ui.run().unwrap();
    inkscape_worker.join().unwrap();
}
