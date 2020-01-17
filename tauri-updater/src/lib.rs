#[macro_use]
pub mod macros;

pub mod http;
pub mod process;
pub mod updater;

use error_chain::error_chain;

error_chain! {
    foreign_links{
        Io(::std::io::Error);
        Json(::serde_json::Error);
        Zip(::zip::result::ZipError);
        API(::tauri_api::Error);
        HTTP(::attohttpc::Error);
    }
    errors{
        Download(t: String) {
            description("Download Error")
            display("Download Error: '{}'", t)
        }
        Updater(t: String) {
            description("Updater Error")
            display("Updater Error: '{}'", t)
        }
        Release(t: String) {
            description("Release Error")
            display("Release Error: '{}'", t)
        }
        Network(t: String) {
            description("Network Error")
            display("Network Error: '{}'", t)
        }
        Config(t: String) {
            description("Config Error")
            display("Config Error: '{}'", t)
        }
    }
}
