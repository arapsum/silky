use std::process::ExitCode;

use service::App;

#[tokio::main]
async fn main() -> Result<ExitCode, ExitCode> {
    if let Err(e) = App::run().await {
        eprintln!("{:?}", e);
        Err(ExitCode::FAILURE)
    } else {
        Ok(ExitCode::SUCCESS)
    }
}
