use loco_rs::prelude::*;
use sea_orm::{
    prelude::Expr,
    sea_query::{self, OnConflict},
    ColumnTrait, EntityTrait, FromQueryResult, JoinType, QueryFilter, QueryOrder, QuerySelect,
    RelationTrait, Set,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    models::_entities::{kid_tags, tags, video_assignments, video_tags, videos},
    utils::structs::GenericSuccessResponse,
};

#[derive(Serialize, ToSchema)]
struct TagResponse {
    id: i32,
    name: String,
}

#[derive(Deserialize, ToSchema)]
struct AddTagsRequest {
    tags: Vec<i32>,
}

#[utoipa::path(
    get,
    path = "/api/kids/{kid_id}/tags",
    tag = "kids",
    params(
        ("kid_id" = i32, Path, description = "Kid ID", example = 1),
    ),
    responses(
        (status = 200, description = "Tags for the kid", body = Vec<TagResponse>),
    )
)]
async fn get_kid_tags(State(ctx): State<AppContext>, Path(kid_id): Path<i32>) -> Result<Response> {
    let results = kid_tags::Entity::find()
        .filter(kid_tags::Column::KidId.eq(kid_id))
        .find_also_related(tags::Entity)
        .all(&ctx.db)
        .await?;

    let tags: Vec<TagResponse> = results
        .into_iter()
        .filter_map(|(_, tag)| tag)
        .map(|t| TagResponse {
            id: t.id,
            name: t.name,
        })
        .collect();

    format::json(tags)
}

#[utoipa::path(
    post,
    path = "/api/kids/{kid_id}/tags",
    tag = "kids",
    params(
        ("kid_id" = i32, Path, description = "Kid ID", example = 1),
    ),
    request_body = AddTagsRequest,
    responses(
        (status = 200, description = "Tags added successfully", body = GenericSuccessResponse),
        (status = 500, description = "Unknown error occurred"),
    )
)]
async fn add_kid_tags(
    State(ctx): State<AppContext>,
    Path(kid_id): Path<i32>,
    Json(data): Json<AddTagsRequest>,
) -> Result<Response> {
    let models = data.tags.into_iter().map(|tag_id| kid_tags::ActiveModel {
        kid_id: Set(kid_id),
        tag_id: Set(tag_id),
        ..Default::default()
    });

    match kid_tags::Entity::insert_many(models)
        .on_conflict(
            OnConflict::columns([kid_tags::Column::KidId, kid_tags::Column::TagId])
                .do_nothing()
                .to_owned(),
        )
        .exec(&ctx.db)
        .await
    {
        Ok(_) => {}
        Err(sea_orm::DbErr::RecordNotInserted) => {
            // This means all rows already existed totally fine
        }
        Err(_) => {
            return format::json(
                serde_json::json!({"success": false, "msg": "Unknown error occurred"}),
            )
        }
    }

    format::json(serde_json::json!({
        "success": true
    }))
}

#[derive(Deserialize, ToSchema)]
pub struct CreateVideoAssignmentRequest {
    pub video_id: String,
}

#[utoipa::path(
    post,
    path = "/api/kids/{kid_id}/videos_assigned",
    tag = "kids",
    params(
        ("kid_id" = i32, Path, description = "Kid ID", example = 1),
    ),
    request_body = CreateVideoAssignmentRequest,
    responses(
        (status = 200, description = "Video assigned successfully", body = GenericSuccessResponse),
        (status = 500, description = "Unknown error occurred"),
    )
)]
async fn create_video_assignment(
    State(ctx): State<AppContext>,
    Path(kid_id): Path<i32>,
    Json(data): Json<CreateVideoAssignmentRequest>,
) -> Result<Response> {
    let model = video_assignments::ActiveModel {
        kid_id: Set(kid_id),
        video_id: Set(data.video_id),
        ..Default::default()
    };

    match video_assignments::Entity::insert(model)
        .on_conflict(
            OnConflict::columns([
                video_assignments::Column::KidId,
                video_assignments::Column::VideoId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec(&ctx.db)
        .await
    {
        Ok(_) => {}
        Err(DbErr::RecordNotInserted) => {
            // already exists → fine
        }
        Err(_) => {
            return format::json(
                serde_json::json!({"success": false, "msg": "Unknown error occurred"}),
            )
        }
    }

    format::json(serde_json::json!({
        "success": true
    }))
}

#[derive(Serialize, ToSchema)]
pub struct GetVideosResponse {
    pub success: bool,
    pub data: Vec<videos::Model>,
}

#[utoipa::path(
    get,
    path = "/api/kids/{kid_id}/videos_assigned",
    tag = "kids",
    params(
        ("kid_id" = i32, Path, description = "Kid ID", example = 1),
    ),
    responses(
        (status = 200, description = "Videos assigned to the kid", body = Vec<videos::Model>),
    )
)]
async fn get_video_assignments(
    State(ctx): State<AppContext>,
    Path(kid_id): Path<i32>,
) -> Result<Response> {
    let results = video_assignments::Entity::find()
        .filter(video_assignments::Column::KidId.eq(kid_id))
        .find_also_related(videos::Entity)
        .all(&ctx.db)
        .await?;

    let videos: Vec<videos::Model> = results.into_iter().filter_map(|(_, video)| video).collect();

    format::json(videos)
}

#[derive(Debug, Serialize, FromQueryResult, ToSchema)]
pub struct RecommendedVideo {
    pub id: String,
    pub title: Option<String>,
    pub thumbnail_url: Option<String>,
    pub duration_seconds: Option<i32>,
    pub score: i64,
}

#[derive(Serialize, ToSchema)]
pub struct RecommendationsResponse {
    pub tags: Vec<String>,
    pub recommendations: Vec<RecommendedVideo>,
}

#[utoipa::path(
    get,
    path = "/api/kids/{kid_id}/recommendations",
    tag = "kids",
    params(
        ("kid_id" = i32, Path, description = "Kid ID", example = 1),
    ),
    responses(
        (status = 200, description = "Recommended videos based on kid's tags", body = RecommendationsResponse),
    )
)]
async fn get_recommendations(
    State(ctx): State<AppContext>,
    Path(kid_id): Path<i32>,
) -> Result<Response> {
    let tags: Vec<String> = kid_tags::Entity::find()
        .select_only()
        .column(tags::Column::Name)
        .join(JoinType::InnerJoin, kid_tags::Relation::Tags.def())
        .filter(kid_tags::Column::KidId.eq(kid_id))
        .into_tuple::<String>()
        .all(&ctx.db)
        .await?;

    let results = video_tags::Entity::find()
        .select_only()
        .column(videos::Column::Id)
        .column(videos::Column::Title)
        .column(videos::Column::ThumbnailUrl)
        .column(videos::Column::DurationSeconds)
        // score = COUNT(tag matches)
        .column_as(Expr::col(video_tags::Column::TagId).count(), "score")
        // join videos
        .join(JoinType::InnerJoin, video_tags::Relation::Videos.def())
        // filter using subquery
        .filter(
            video_tags::Column::TagId.in_subquery(
                sea_query::Query::select()
                    .column(kid_tags::Column::TagId)
                    .from(kid_tags::Entity)
                    .and_where(kid_tags::Column::KidId.eq(kid_id))
                    .to_owned(),
            ),
        )
        // exclude assigned
        .filter(
            videos::Column::Id.not_in_subquery(
                sea_query::Query::select()
                    .column(video_assignments::Column::VideoId)
                    .from(video_assignments::Entity)
                    .and_where(video_assignments::Column::KidId.eq(kid_id))
                    .to_owned(),
            ),
        )
        // group
        .group_by(Expr::col((videos::Entity, videos::Column::Id)))
        .group_by(Expr::col((videos::Entity, videos::Column::Title)))
        .group_by(Expr::col((videos::Entity, videos::Column::ThumbnailUrl)))
        .group_by(Expr::col((videos::Entity, videos::Column::DurationSeconds)))
        // rank
        .order_by_desc(Expr::cust("score"))
        .limit(20)
        .into_model::<RecommendedVideo>()
        .all(&ctx.db)
        .await?;

    format::json(RecommendationsResponse {
        tags,
        recommendations: results,
    })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("kids")
        .add("/{kid_id}/tags", get(get_kid_tags))
        .add("/{kid_id}/tags", post(add_kid_tags))
        .add("/{kid_id}/videos_assigned", get(get_video_assignments))
        .add("/{kid_id}/videos_assigned", post(create_video_assignment))
        .add("/{kid_id}/recommendations", get(get_recommendations))
}
