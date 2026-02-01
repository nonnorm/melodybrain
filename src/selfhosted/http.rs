use std::sync::{
    Arc,
    atomic::{AtomicI32, Ordering},
};

use axum::{
    Form, Json, Router,
    extract::State,
    http::header,
    response::{IntoResponse, Response},
    routing::get,
};
use melodybrain::{COUNTRIES, WORLDWIDE, search_country};
use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;

use crate::{
    generate_seed,
    notes::{Note, NoteGenerator},
};

pub type ArcState = Arc<crate::State>;

pub fn router(state: ArcState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/data", get(data))
        .with_state(state)
}

async fn index() -> Response {
    Response::builder()
        .header(header::CONTENT_TYPE, "text/html;charset=utf-8")
        .body(include_str!("./assets/page.html").into())
        .unwrap()
}

#[derive(Debug, Serialize)]
pub struct Data {
    notes: Vec<Note>,
    seed: i32,
    connected: u32,
    heatmap: Vec<f32>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct DataForm {
    idx: u32,
    seed: SeedType,
    country: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum SeedType {
    Local,
    #[default]
    Global,
    NewLocal,
}

#[axum::debug_handler]
async fn data(State(state): State<ArcState>, Form(form): Form<DataForm>) -> Json<Data> {
    let country_code = search_country(&form.country.to_ascii_uppercase()).unwrap_or(WORLDWIDE);

    let stats = loop {
        if let Some(stats) = state.send_heartbeat(country_code).await {
            break stats;
        }
    };

    let seed = match form.seed {
        SeedType::Local => state.local_seed.load(Ordering::Relaxed),
        SeedType::Global => stats.seed,
        SeedType::NewLocal => {
            // This probably violates some rule of atomics, but at least it won't cause UB
            let new = generate_seed();
            state.local_seed.store(new, Ordering::Relaxed);
            new
        }
    };

    let heatmap: Vec<f32> = stats
        .country_heatmap
        .into_iter()
        .enumerate()
        .filter_map(|(idx, x)| COUNTRIES[idx].1.then_some(x))
        .collect();

    let notes: Vec<_> = NoteGenerator::new(form.idx, seed).take(128).collect();
    Json(Data {
        notes,
        seed,
        connected: stats.connected,
        heatmap,
    })
}
