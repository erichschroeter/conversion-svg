use std::sync::{Arc, Mutex};

mod inkscape;

mod generated_code {
    slint::include_modules!();
}
pub use generated_code::*;
use inkscape::{find_inkscape_executable, InkscapeArgsBuilder};

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
