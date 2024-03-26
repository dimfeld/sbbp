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

fn sync_types() -> Result<(), Report<Error>> {
    let mut output = vec![];

    let value = crate::models::video::VideoProcessingState::export_to_string()
        .change_context(Error::TypeExport)
        .attach_printable("crate::models::video::VideoProcessingState")?;
    output.push(value);

    let output = output.join("\n\n");
    let output_path = "../web/src/lib/api_types.ts";
    std::fs::write(output_path, output.as_bytes()).change_context(Error::TypeExport)?;

    Ok(())
}
