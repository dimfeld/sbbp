use clap::{Args, Subcommand};
use error_stack::{Report, ResultExt};
use ts_rs::TS;

use crate::Error;

#[derive(Args, Debug)]
pub struct UtilCommand {
    #[clap(subcommand)]
    pub command: UtilSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum UtilSubcommand {
    HashPassword(HashPasswordCommand),
    SyncTypes,
}

#[derive(Args, Debug)]
pub struct HashPasswordCommand {
    password: String,
}

impl UtilCommand {
    pub async fn handle(self) -> Result<(), Report<Error>> {
        match self.command {
            UtilSubcommand::HashPassword(password) => {
                let hash = filigree::auth::password::new_hash(password.password)
                    .await
                    .change_context(Error::AuthSubsystem)?
                    .0;
                println!("{hash}");
            }
            UtilSubcommand::SyncTypes => sync_types()?,
        }

        Ok(())
    }
}

macro_rules! export {
    ($name:ty) => {
        <$name>::export_to_string()
            .change_context(Error::TypeExport)
            .attach_printable(stringify!($name))?
    };
}

fn sync_types() -> Result<(), Report<Error>> {
    let mut output = vec![];

    output.push(export!(crate::models::video::VideoProcessingState));

    // Each exported type string has a comment above it. Leave the first one but strip the rest
    // since they're all identical.
    let trimmed = output
        .iter()
        .enumerate()
        .map(|(i, s)| {
            if i == 0 {
                return s.as_str();
            }

            let newline = s.char_indices().find(|(_, c)| *c == '\n').map(|(i, _)| i);
            if let Some(newline) = newline {
                s[newline + 1..].trim()
            } else {
                s.as_str()
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let output_path = "../web/src/lib/api_types.ts";
    std::fs::write(output_path, trimmed.as_bytes()).change_context(Error::TypeExport)?;

    Ok(())
}
