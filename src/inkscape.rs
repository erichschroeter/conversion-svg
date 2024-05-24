use std::{env, path::{Path, PathBuf}};
use regex::Regex;

use super::AppUI;
use futures::{future::{Fuse, FusedFuture, FutureExt}, pin_mut, select};
use slint::ComponentHandle;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

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

#[derive(Debug)]
pub enum InkscapeMessage {
    Quit,
    Export,
}

pub struct InkscapeWorker {
    pub channel: UnboundedSender<InkscapeMessage>,
    worker_thread: std::thread::JoinHandle<()>,
}

impl InkscapeWorker {
    pub fn new(ui: &AppUI) -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let worker_thread = std::thread::spawn({
            let handle_weak = ui.as_weak();
            move || {
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(inkscape_worker_loop(rx, handle_weak))
            }
        });
        Self {
            channel: tx,
            worker_thread,
        }
    }

    pub fn join(self) -> std::thread::Result<()> {
        let _ = self.channel.send(InkscapeMessage::Quit);
        self.worker_thread.join()
    }
}

async fn inkscape_worker_loop(
    mut rx: UnboundedReceiver<InkscapeMessage>,
    handle: slint::Weak<AppUI>,
) {
    match tokio::task::spawn(find_inkscape_executable()).await {
        Err(e) => {
            log::error!("Failed while searching for Inkscape: {}", e);
            return;
        }
        Ok(None) => {
            log::error!("Failed to find Inkscape");
            return;
        }
        Ok(Some(exe_path)) => {
            log::info!("Inkscape executable found: {:?}", exe_path);
        }
    }
    loop {
        let m = rx.recv().await;
        match m {
            None => return,
            Some(InkscapeMessage::Export) => {
                println!("Exporting...");
                let _inkscape_handle = tokio::task::spawn(run_inkscape(handle.clone()));
                // return;
            },
            Some(InkscapeMessage::Quit) => {
                // inkscape_handle.abort();
                return;
            },
        }
    }
}

pub async fn find_inkscape_executable() -> Option<PathBuf> {
    // Search for inkscape on the PATH
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths).filter_map(|dir| {
            let abs_path = dir.join("inkscape");
            log::debug!("searching: {}", abs_path.display());
            if abs_path.exists() || abs_path.with_extension("exe").exists() {
                Some(abs_path)
            } else {
                None
            }
        }).next()
    })
}

async fn run_inkscape(handle: slint::Weak<AppUI>) {
    let args = InkscapeArgsBuilder::new();
    let cmd = InkscapeCmd::new(Path::new("inkscape").into(), args.build());
    cmd.exec();
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
