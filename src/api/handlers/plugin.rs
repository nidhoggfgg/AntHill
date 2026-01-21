use crate::api::dto::plugin::{InstallPluginRequest, PluginResponse, PluginsListResponse};
use crate::api::routes::AppState;
use crate::error::Result;
use crate::models::PluginType;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

pub async fn list_plugins(State(state): State<AppState>) -> Result<Json<PluginsListResponse>> {
    let plugins = state.plugin_service.list_plugins().await?;
    let response = PluginsListResponse {
        data: plugins.into_iter().map(PluginResponse::from).collect(),
    };
    Ok(Json(response))
}

pub async fn get_plugin(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PluginResponse>> {
    let plugin = state.plugin_service.get_plugin(&id).await?;
    Ok(Json(PluginResponse::from(plugin)))
}

pub async fn install_plugin(
    State(state): State<AppState>,
    Json(req): Json<InstallPluginRequest>,
) -> Result<(StatusCode, Json<PluginResponse>)> {
    let plugin_type = match req.plugin_type.as_str() {
        "python" => PluginType::Python,
        "javascript" | "js" => PluginType::JavaScript,
        _ => return Err(crate::error::AppError::InvalidPluginType),
    };

    let plugin = state
        .plugin_service
        .install_plugin(
            req.name,
            req.version,
            plugin_type,
            req.description,
            req.author,
            req.package_url,
            req.entry_point,
            req.metadata,
        )
        .await?;

    Ok((StatusCode::CREATED, Json(PluginResponse::from(plugin))))
}

pub async fn uninstall_plugin(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    state.plugin_service.uninstall_plugin(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn enable_plugin(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    state.plugin_service.enable_plugin(&id).await?;
    Ok(StatusCode::OK)
}

pub async fn disable_plugin(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    state.plugin_service.disable_plugin(&id).await?;
    Ok(StatusCode::OK)
}
