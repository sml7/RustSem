use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::{Arc, Mutex};
use polars::prelude::*;
use crate::data_analysis::{calculate_summary_statistics, calculate_distribution};
use polars::lazy::prelude::*;
use polars::lazy::dsl::*;








pub async fn show_gender_distribution(
    State(state): State<Arc<Mutex<AppState>>>
) -> Html<String> {
    // 1) DataFrame aus dem State holen
    let guard = state.lock().unwrap();
    let df = &guard.df;

    // 2) Spaltennamen definieren (Gender, Salary)
    let gender_col = "Gender";
    let salary_col = "Yearly brutto salary (without bonus and stocks) in EUR";

    // 3) Prüfen, ob Spalten existieren
    let col_names = df.get_column_names();
    if !col_names.iter().any(|s| s.as_str() == gender_col) ||
        !col_names.iter().any(|s| s.as_str() == salary_col) {
        return Html("<h1>ERROR: Gender or Salary column not found</h1>".to_string());
    }

    // 4) Gender-Spalte als Utf8Chunked holen
    let gender_series = df.column(gender_col).unwrap();
    let gender_utf8 = match gender_series.utf8() {
        Ok(utf) => utf,
        Err(e) => {
            return Html(format!("<h1>ERROR: {}</h1>", e));
        }
    };

    // Hilfsfunktion: Verteilung berechnen
    // (Bins und Frequenzen für ein Array von f64)
    let calc_distribution = |values: &[f64], bin_count: usize| -> (Vec<f64>, Vec<usize>) {
        if values.is_empty() {
            return (vec![], vec![]);
        }
        let min_val = values.iter().copied().fold(f64::INFINITY, f64::min);
        let max_val = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let bin_size = (max_val - min_val).max(1.0) / bin_count as f64;
        let mut freq = vec![0; bin_count];

        for &val in values {
            let mut idx = ((val - min_val) / bin_size).floor() as isize;
            if idx < 0 {
                idx = 0;
            }
            if idx as usize >= bin_count {
                idx = bin_count as isize - 1;
            }
            freq[idx as usize] += 1;
        }

        let bins = (0..bin_count)
            .map(|i| min_val + i as f64 * bin_size)
            .collect::<Vec<f64>>();
        (bins, freq)
    };

    // === Männer-Gehälter filtern ===
    let men_mask = gender_utf8.equal("M");             // -> BooleanChunked
    let men_df = df.filter(&men_mask).unwrap();        // -> DataFrame
    let men_salaries = men_df
        .column(salary_col).unwrap()
        .f64().unwrap()
        .into_no_null_iter()
        .collect::<Vec<f64>>();

    // === Frauen-Gehälter filtern ===
    let women_mask = gender_utf8.equal("F");
    let women_df = df.filter(&women_mask).unwrap();
    let women_salaries = women_df
        .column(salary_col).unwrap()
        .f64().unwrap()
        .into_no_null_iter()
        .collect::<Vec<f64>>();

    // === Verteilung berechnen ===
    let bin_count = 10;
    let (men_bins, men_freq) = calc_distribution(&men_salaries, bin_count);
    let (women_bins, women_freq) = calc_distribution(&women_salaries, bin_count);

    // 5) HTML zusammenbauen
    // - Wir betten Plotly via CDN ein
    // - Erzeugen 2 Balken (traceMen, traceWomen)
    // - Verwenden barmode: 'group'
    let html = format!(r#"
    <!DOCTYPE html>
    <html>
      <head>
        <meta charset="UTF-8"/>
        <title>Gender Salary Distribution</title>
        <script src="https://cdn.plot.ly/plotly-2.18.2.min.js"></script>
        <style>
          body {{
            font-family: Arial, sans-serif;
            background: #f4f4f9;
            color: #333;
            text-align: center;
          }}
          #chart {{
            width: 90%;
            max-width: 800px;
            height: 600px;
            margin: 0 auto;
          }}
        </style>
      </head>
      <body>
        <h1>Gehaltverteilung für M vs. F</h1>
        <div id="chart"></div>

        <script>
          // Rust-Daten einbetten
          let menBins = {men_bins:?};
          let menFreq = {men_freq:?};
          let womenBins = {women_bins:?};
          let womenFreq = {women_freq:?};

          // Plotly-Traces
          let traceMen = {{
            x: menBins,
            y: menFreq,
            type: 'bar',
            name: 'Männlich',
            marker: {{ color: 'blue' }}
          }};

          let traceWomen = {{
            x: womenBins,
            y: womenFreq,
            type: 'bar',
            name: 'Weiblich',
            marker: {{ color: 'red' }}
          }};

          let data = [traceMen, traceWomen];
          let layout = {{
            title: 'Salary Distribution by Gender',
            xaxis: {{ title: 'Salary (EUR)' }},
            yaxis: {{ title: 'Frequency' }},
            barmode: 'group'
          }};

          Plotly.newPlot('chart', data, layout);
        </script>
      </body>
    </html>
    "#);

    // 6) Als Html<String> zurückgeben
    Html(html)
}







pub async fn eda_summary(State(state): State<Arc<Mutex<AppState>>>) -> Json<serde_json::Value> {
    let guard = state.lock().unwrap();
    let summary = calculate_summary_statistics(&guard.df);
    Json(summary)
}

/// Endpunkt für Verteilungsdaten
pub async fn get_distribution_data(State(state): State<Arc<Mutex<AppState>>>) -> Json<serde_json::Value> {
    let guard = state.lock().unwrap();
    let distribution = calculate_distribution(&guard.df);
    Json(distribution)
}

pub struct AppState {
    pub df: DataFrame,
}

#[derive(Deserialize)]
struct PredictParams {
    experience: f64,
}

pub fn create_router(state: Arc<Mutex<AppState>>) -> Router {
    Router::new()
        .route("/", get(show_index)) // Hauptseite
        .route("/scatter-data", get(get_scatter_data)) // API-Route für Scatterplot-Daten
        .route("/predict", get(show_predict)) // Seite für Gehaltsvorhersage
        .route("/predict-salary", get(predict_salary))
        .route("/eda-summary", get(eda_summary)) // Statistiken
        .route("/distribution-data", get(get_distribution_data)) // Verteilung// API für Gehaltsvorhersage
        .with_state(state)
}

/// Hauptseite mit Plotly.js für den Scatterplot
/// Hauptseite mit Plotly.js für den Scatterplot und Verteilung
/// Hauptseite mit Plotly.js für Scatterplot, Verteilung und EDA-Ergebnisse
async fn show_index() -> impl IntoResponse {
    let html = r#"
    <!DOCTYPE html>
    <html lang="de">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Salary Analysis</title>
        <script src="https://cdn.plot.ly/plotly-2.18.2.min.js"></script>
        <style>
            body {
                font-family: Arial, sans-serif;
                margin: 0;
                padding: 0;
                background-color: #f4f4f9;
                color: #333;
                text-align: center;
            }
            header {
                background-color: #4CAF50;
                color: white;
                padding: 1rem;
                font-size: 1.5rem;
            }
            main {
                padding: 2rem;
            }
            #chart, #distribution-chart {
                width: 90%;
                max-width: 800px;
                height: 600px;
                margin: 0 auto;
            }
            #eda-summary {
                text-align: left;
                margin: 20px auto;
                max-width: 800px;
                padding: 10px;
                border: 1px solid #ddd;
                background-color: #fff;
            }
            button {
                margin-top: 20px;
                padding: 10px 20px;
                font-size: 16px;
                background-color: #4CAF50;
                color: white;
                border: none;
                border-radius: 5px;
                cursor: pointer;
            }
            button:hover {
                background-color: #45a049;
            }
            footer {
                margin-top: 2rem;
                padding: 1rem;
                background-color: #4CAF50;
                color: white;
                font-size: 0.9rem;
            }
        </style>
    </head>
    <body>
        <header>
            Salary Analysis Viewer
        </header>
        <main>
            <h1>Gehalt und Erfahrung</h1>
            <p>Scatterplot und Gehaltsverteilung:</p>
            <div id="chart"></div>
            <h2>Gehaltsverteilung</h2>
            <div id="distribution-chart"></div>
            <h2>EDA Ergebnisse</h2>
            <div id="eda-summary">
                <!-- EDA-Zusammenfassung wird hier eingefügt -->
            </div>
            <button onclick="location.href='/predict'">Predict Now</button>
        </main>
        <footer>
            &copy; 2025 Gehaltsanalyse. Alle Rechte vorbehalten.
        </footer>
        <script>
            // Funktion, um die Scatterplot-Daten zu laden
            async function fetchScatterData() {
                const response = await fetch('/scatter-data');
                const data = await response.json();

                const trace = {
                    x: data.experience,
                    y: data.salary,
                    mode: 'markers',
                    type: 'scatter',
                    marker: { color: 'blue' },
                };

                const layout = {
                    title: 'Salary vs. Experience',
                    xaxis: { title: 'Years of Experience' },
                    yaxis: { title: 'Salary (EUR)' },
                };

                Plotly.newPlot('chart', [trace], layout);
            }

            // Funktion, um die Verteilungsdaten zu laden
            async function fetchDistributionData() {
                const response = await fetch('/distribution-data');
                const data = await response.json();

                const trace = {
                    x: data.bins,
                    y: data.frequencies,
                    type: 'bar',
                    marker: { color: 'blue' },
                };

                const layout = {
                    title: 'Salary Distribution',
                    xaxis: { title: 'Salary (EUR)' },
                    yaxis: { title: 'Frequency' },
                };

                Plotly.newPlot('distribution-chart', [trace], layout);
            }

            // Funktion, um die EDA-Zusammenfassung zu laden und anzuzeigen
            async function fetchEDASummary() {
                const response = await fetch('/eda-summary');
                const data = await response.json();

                // Formatierte HTML-Ausgabe der EDA-Ergebnisse
                const salaryStats = `
                    <h3>Gehalt</h3>
                    <p><strong>Mittelwert:</strong> ${data.salary.mean.toFixed(2)} EUR</p>
                    <p><strong>Median:</strong> ${data.salary.median.toFixed(2)} EUR</p>
                    <p><strong>Standardabweichung:</strong> ${data.salary.std_dev.toFixed(2)}</p>
                    <p><strong>Minimum:</strong> ${data.salary.min.toFixed(2)} EUR</p>
                    <p><strong>Maximum:</strong> ${data.salary.max.toFixed(2)} EUR</p>
                `;

                const experienceStats = `
                    <h3>Erfahrung</h3>
                    <p><strong>Mittelwert:</strong> ${data.experience.mean.toFixed(2)} Jahre</p>
                    <p><strong>Median:</strong> ${data.experience.median.toFixed(2)} Jahre</p>
                    <p><strong>Standardabweichung:</strong> ${data.experience.std_dev.toFixed(2)}</p>
                    <p><strong>Minimum:</strong> ${data.experience.min.toFixed(2)} Jahre</p>
                    <p><strong>Maximum:</strong> ${data.experience.max.toFixed(2)} Jahre</p>
                `;

                // EDA-Ergebnisse in den entsprechenden Div einfügen
                document.getElementById('eda-summary').innerHTML = `
                    <h2>EDA Zusammenfassung</h2>
                    ${salaryStats}
                    ${experienceStats}
                `;
            }

            // Lade die Daten beim Start
            fetchScatterData();
            fetchDistributionData();
            fetchEDASummary();
        </script>
    </body>
    </html>
    "#;
    Html(html.to_string())
}



/// API, um Scatterplot-Daten als JSON bereitzustellen
async fn get_scatter_data(State(state): State<Arc<Mutex<AppState>>>) -> Json<serde_json::Value> {
    let guard = state.lock().unwrap();
    let df = &guard.df;

    let experience = df
        .column("Total years of experience")
        .expect("Spalte nicht gefunden")
        .f64()
        .expect("Spalte ist nicht vom Typ f64")
        .into_no_null_iter()
        .collect::<Vec<f64>>();

    let salary = df
        .column("Yearly brutto salary (without bonus and stocks) in EUR")
        .expect("Spalte nicht gefunden")
        .f64()
        .expect("Spalte ist nicht vom Typ f64")
        .into_no_null_iter()
        .collect::<Vec<f64>>();

    Json(json!({
        "experience": experience,
        "salary": salary,
    }))
}

/// Seite für Gehaltsvorhersage
async fn show_predict() -> impl IntoResponse {
    let html = r#"
    <!DOCTYPE html>
    <html lang="de">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Predict Salary</title>
        <style>
            body {
                font-family: Arial, sans-serif;
                margin: 0;
                padding: 0;
                background-color: #f4f4f9;
                color: #333;
                text-align: center;
            }
            header {
                background-color: #4CAF50;
                color: white;
                padding: 1rem;
                font-size: 1.5rem;
            }
            main {
                padding: 2rem;
            }
            input {
                padding: 10px;
                font-size: 16px;
                margin-top: 10px;
                width: 200px;
                border: 1px solid #ccc;
                border-radius: 5px;
            }
            button {
                margin-top: 10px;
                padding: 10px 20px;
                font-size: 16px;
                background-color: #4CAF50;
                color: white;
                border: none;
                border-radius: 5px;
                cursor: pointer;
            }
            button:hover {
                background-color: #45a049;
            }
            footer {
                margin-top: 2rem;
                padding: 1rem;
                background-color: #4CAF50;
                color: white;
                font-size: 0.9rem;
            }
        </style>
    </head>
    <body>
        <header>
            Salary Predictor
        </header>
        <main>
            <h1>Vorhersage des Gehalts</h1>
            <p>Geben Sie die Jahre an Erfahrung ein, um das geschätzte Gehalt zu sehen:</p>
            <input type="number" id="experience" placeholder="Jahre an Erfahrung" />
            <button onclick="predict()">Predict</button>
            <p id="result" style="margin-top: 20px; font-size: 20px; font-weight: bold;"></p>
        </main>
        <footer>
            &copy; 2025 Gehaltsanalyse. Alle Rechte vorbehalten.
        </footer>
        <script>
            async function predict() {
                const experience = document.getElementById('experience').value;
                if (!experience) {
                    alert("Bitte geben Sie die Jahre an Erfahrung ein.");
                    return;
                }

                const response = await fetch(`/predict-salary?experience=${experience}`);
                const data = await response.json();
                document.getElementById('result').innerText =
                    `Geschätztes Gehalt: ${data.predicted_salary.toFixed(2)} EUR`;
            }
        </script>
    </body>
    </html>
    "#;
    Html(html.to_string())
}

/// API, um Gehaltsvorhersage basierend auf Erfahrung zu berechnen
async fn predict_salary(
    State(state): State<Arc<Mutex<AppState>>>,
    Query(params): Query<PredictParams>
) -> Json<serde_json::Value> {
    let guard = state.lock().unwrap();
    let df = &guard.df;

    let experience = df
        .column("Total years of experience")
        .expect("Spalte nicht gefunden")
        .f64()
        .expect("Spalte ist nicht vom Typ f64")
        .into_no_null_iter()
        .collect::<Vec<f64>>();

    let salary = df
        .column("Yearly brutto salary (without bonus and stocks) in EUR")
        .expect("Spalte nicht gefunden")
        .f64()
        .expect("Spalte ist nicht vom Typ f64")
        .into_no_null_iter()
        .collect::<Vec<f64>>();

    let n = experience.len() as f64;
    let mean_x = experience.iter().sum::<f64>() / n;
    let mean_y = salary.iter().sum::<f64>() / n;

    let mut cov_xy = 0.0;
    let mut var_x = 0.0;
    for (&x, &y) in experience.iter().zip(salary.iter()) {
        let dx = x - mean_x;
        let dy = y - mean_y;
        cov_xy += dx * dy;
        var_x += dx * dx;
    }

    let slope = cov_xy / var_x;
    let intercept = mean_y - slope * mean_x;

    let predicted_salary = intercept + slope * params.experience;

    Json(json!({ "predicted_salary": predicted_salary }))
}
