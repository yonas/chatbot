use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
//use crate::initializers::chat::{self, Username, Res};

pub struct InstagramWorker {
    pub ctx: AppContext,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct InstagramWorkerArgs {
    pub profile: String,
}

#[async_trait]
impl BackgroundWorker<InstagramWorkerArgs> for InstagramWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }
    async fn perform(&self, args: InstagramWorkerArgs) -> Result<()> {
        tracing::info!(profile = args.profile, "Instagram worker begin");

        let homedir = match dirs::home_dir() {
            Some(d) => d.display().to_string(),
            None => "/home/yonas".into(),
        };
        let output = std::process::Command::new(homedir + "/bin/instagram-profile-download")
            .arg("--no-metadata-json")
            .arg("--no-compress-json")
            .arg("--no-captions")
            .arg("--no-video-thumbnails")
            .arg("--load-cookies=\"firefox\"")
            .arg(args.profile)
            .output()
            .expect("failed to execute process");

        tracing::debug!("status: {}", output.status);
        tracing::debug!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        tracing::debug!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        tracing::info!(profile = args.profile, "Instagram worker end");

        Ok(())
    }
}
