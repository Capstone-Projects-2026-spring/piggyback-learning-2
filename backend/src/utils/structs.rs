use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct GenericSuccessResponse {
    pub success: bool,
}
