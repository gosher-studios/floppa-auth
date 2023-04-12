use std::path::Path;
use std::fs::{File, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::io::Write;
use std::process::Command;
use std::error::Error;

const TAILWIND_URL: &str =
  "https://github.com/tailwindlabs/tailwindcss/releases/download/v3.1.8/tailwindcss-linux-x64";
const TAILWIND_PATH: &str = "target/tailwind";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  if !Path::new(TAILWIND_PATH).exists() {
    let mut tw = File::create(TAILWIND_PATH)?;
    tw.set_permissions(Permissions::from_mode(0o755))?;
    tw.write_all(&reqwest::get(TAILWIND_URL).await?.bytes().await?)?;
    drop(tw);
  }
  Command::new(TAILWIND_PATH)
    .args([
      "-i",
      "templates/main.css",
      "-o",
      "static/main.css",
      "--minify",
    ])
    .output()?;
  Ok(())
}
