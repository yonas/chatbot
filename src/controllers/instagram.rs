use axum::debug_handler;
use loco_rs::prelude::*;
use crate::{
    models::instagram::InstagramParams,
    workers::instagram::{InstagramWorkerArgs, InstagramWorker},
};

/// Register function creates a new user with the given parameters and sends a
/// welcome email to the user
#[debug_handler]
async fn instagram(
    State(ctx): State<AppContext>,
    Json(params): Json<InstagramParams>,
) -> Result<Response> {
    let _ = InstagramWorker::perform_later(
        &ctx,
        InstagramWorkerArgs {
            profile: params.profile,
        },
    )
    .await
    .ok();

    format::json(())
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/instagram", post(instagram))
}
