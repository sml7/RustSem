mod data_analysis;
mod plots;
mod web_app;

use std::sync::{Arc, Mutex};
use axum::Server;
use axum::{Router, routing::get};
use data_analysis::*;
use web_app::*;
use web_app::{eda_summary, get_distribution_data, AppState};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let csv_path = "cleaned_data.csv";
    let image_path = "salary_histogram.png";

    println!("Lade Daten aus {csv_path}...");
    let mut df = load_data(csv_path)?;
    println!("Daten geladen. Shape: {:?}", df.shape());

    clean_data(&mut df)?;
    println!("Nach clean_data: Shape: {:?}", df.shape());

    eda(&df);

    create_salary_histogram_and_save(&df, image_path)?;

    println!("Bild wurde erfolgreich unter {image_path} gespeichert.");

    let shared_state = Arc::new(Mutex::new(AppState { df }));
    let app = create_router(shared_state);

    let addr = "0.0.0.0:3000";
    println!("Server läuft auf http://{addr}");
    Server::bind(&addr.parse()?).serve(app.into_make_service()).await?;
    Ok(())
}
