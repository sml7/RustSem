// src/data_analysis.rs

use polars::prelude::*;
use serde::{Deserialize, Serialize};
use polars::lazy::prelude::LazyCsvReader;
use plotters::prelude::*;
use serde_json::json;



#[derive(Debug, Serialize, Deserialize)]
pub struct SurveyRecord {
    #[serde(rename = "Timestamp")]
    pub timestamp: Option<String>,

    #[serde(rename = "Age")]
    pub age: Option<u32>,

    #[serde(rename = "Gender")]
    pub gender: Option<String>,

    #[serde(rename = "City")]
    pub city: Option<String>,

    #[serde(rename = "Position")]
    pub position: Option<String>,

    #[serde(rename = "Total years of experience")]
    pub total_experience: Option<f64>,  // f64 statt f32, wenn gewünscht

    #[serde(rename = "Years of experience in Germany")]
    pub experience_germany: Option<f64>,

    #[serde(rename = "Seniority level")]
    pub seniority_level: Option<String>,

    #[serde(rename = "Your main technology / programming language")]
    pub main_tech: Option<String>,

    #[serde(rename = "Other technologies/programming languages you use often")]
    pub other_techs: Option<String>,

    #[serde(rename = "Yearly brutto salary (without bonus and stocks) in EUR")]
    pub yearly_brutto_salary: Option<f64>,  // auf f64 angepasst

    #[serde(rename = "Yearly bonus + stocks in EUR")]
    pub yearly_bonus_stocks: Option<f64>,

    #[serde(rename = "Annual brutto salary (without bonus and stocks) one year ago. Only answer if staying in the same country")]
    pub yearly_brutto_salary_last: Option<f64>,

    #[serde(rename = "Annual bonus+stocks one year ago. Only answer if staying in same country")]
    pub bonus_stocks_last: Option<f64>,

    #[serde(rename = "Number of vacation days")]
    pub vacation_days: Option<u32>,

    #[serde(rename = "Employment status")]
    pub employment_status: Option<String>,

    #[serde(rename = "Сontract duration")]
    pub contract_duration: Option<String>,

    #[serde(rename = "Main language at work")]
    pub work_language: Option<String>,

    #[serde(rename = "Company size")]
    pub company_size: Option<String>,

    #[serde(rename = "Company type")]
    pub company_type: Option<String>,

    #[serde(rename = "Have you lost your job due to the coronavirus outbreak?")]
    pub lost_job_covid: Option<String>,

    #[serde(rename = "Have you been forced to have a shorter working week (Kurzarbeit)? If yes, how many hours per week")]
    pub shorter_work_week: Option<String>,

    #[serde(rename = "Have you received additional monetary support from your employer due to Work From Home? If yes, how much in 2020 in EUR")]
    pub wfh_support: Option<f64>,
}

/// CSV mit Polars laden und in ein DataFrame konvertieren
pub fn load_data(path: &str) -> PolarsResult<DataFrame> {
    let lazy_frame = LazyCsvReader::new(path)
        .with_infer_schema_length(Some(10_000))
        .with_has_header(true)
        // falls du "No"-Zellen ignorieren möchtest, kannst du:
        // .with_null_values(Some(NullValues::AllColumns(vec!["No".to_string()])))
        // oder
        // .with_ignore_errors(true)
        .finish()?;

    let df = lazy_frame.collect()?;
    Ok(df)
}

/// Beispiel: Droppe Zeilen mit null in ALLEN Spalten
pub fn clean_data(df: &mut DataFrame) -> PolarsResult<()> {
    // Polars erfordert manchmal eine Typangabe:
    *df = df.drop_nulls::<String>(None)?;
    Ok(())
}

/// Explorative Datenanalyse (Beispiel)
pub fn eda(df: &DataFrame) {
    println!("=== EDA ===");
    println!("Shape: {:?}", df.shape());
    println!("Schema: {:?}", df.schema());

    // Gehaltsspalte als f64 verarbeiten
    if let Ok(salary_series) = df.column("Yearly brutto salary (without bonus and stocks) in EUR") {
        if let Ok(salary_f64) = salary_series.f64() {
            let mean_salary = salary_f64.mean().unwrap_or(f64::NAN);
            let std_salary = salary_f64.std(1).unwrap_or(f64::NAN);
            println!("Durchschnittliches Gehalt: {:.2}", mean_salary);
            println!("Standardabweichung (ddof=1): {:.2}", std_salary);
        } else {
            println!("'Yearly brutto salary...' ist nicht vom Typ f64.");
        }
    }
}

/// Einfache lineare Regression (Gehalt ~ Erfahrung) mit f64
pub fn simple_regression_example(df: &DataFrame) {
    let experience = df
        .column("Total years of experience")
        .unwrap()
        .f64()
        .unwrap()
        .into_no_null_iter()
        .collect::<Vec<f64>>();

    let salary = df
        .column("Yearly brutto salary (without bonus and stocks) in EUR")
        .unwrap()
        .f64()
        .unwrap()
        .into_no_null_iter()
        .collect::<Vec<f64>>();

    if experience.is_empty() || salary.is_empty() || experience.len() != salary.len() {
        println!("Nicht genügend Daten für Regression!");
        return;
    }

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

    println!("Lineare Regression (Gehalt ~ Erfahrung): slope={:.3}, intercept={:.3}", slope, intercept);

    let predicted_salary = intercept + slope * 5.0;
    println!("Geschätztes Gehalt für 5 Jahre Erfahrung = {:.2} EUR", predicted_salary);
}




pub fn create_salary_histogram_and_save(df: &DataFrame, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let col_name = "Yearly brutto salary (without bonus and stocks) in EUR";

    // Extract salary column as Vec<f32>
    let salaries = df
        .column(col_name)
        .expect("Spalte nicht gefunden")
        .f64()
        .expect("Spalte ist nicht vom Typ f64")
        .into_no_null_iter()
        .map(|v| v as f32)
        .collect::<Vec<f32>>();

    println!("DEBUG: Salaries passed to the histogram: {:?}", salaries);

    // Generate the histogram
    let width = 800;
    let height = 600;
    let root = BitMapBackend::new(file_path, (width, height))
        .into_drawing_area();
    root.fill(&WHITE)?;

    let min_sal = salaries.iter().cloned().reduce(f32::min).unwrap_or(0.0);
    let max_sal = salaries.iter().cloned().reduce(f32::max).unwrap_or(1.0);
    let bin_count = 10;
    let bin_size = (max_sal - min_sal) / bin_count as f32;
    let mut freq = vec![0; bin_count];

    for &val in salaries.iter() {
        let mut idx = ((val - min_sal) / bin_size).floor() as isize;
        if idx < 0 {
            idx = 0;
        }
        if idx as usize >= bin_count {
            idx = bin_count as isize - 1;
        }
        freq[idx as usize] += 1;
    }

    println!("DEBUG: Bin count = {}, Bin size = {}", bin_count, bin_size);
    println!("DEBUG: Frequencies = {:?}", freq);

    let max_freq = *freq.iter().max().unwrap_or(&1);

    let mut chart = ChartBuilder::on(&root)
        .caption("Salary Histogram", ("sans-serif", 20).into_font())
        .margin(5)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(min_sal..max_sal, 0..max_freq)?;

    chart.configure_mesh()
        .x_desc("Salary (EUR)")
        .y_desc("Frequency")
        .draw()?;

    for (i, &count) in freq.iter().enumerate() {
        let x0 = min_sal + i as f32 * bin_size;
        let x1 = x0 + bin_size;
        chart
            .draw_series(std::iter::once(Rectangle::new(
                [(x0, 0), (x1, count)],
                BLUE.filled(),
            )))?;
    }

    root.present()?; // Save the image
    println!("Histogram saved to {}", file_path);

    Ok(())
}

pub fn calculate_summary_statistics(df: &DataFrame) -> serde_json::Value {
    let salary_col = "Yearly brutto salary (without bonus and stocks) in EUR";
    let experience_col = "Total years of experience";

    let salary = df
        .column(salary_col)
        .expect("Spalte nicht gefunden")
        .f64()
        .expect("Spalte ist nicht vom Typ f64")
        .into_no_null_iter()
        .collect::<Vec<f64>>();

    let experience = df
        .column(experience_col)
        .expect("Spalte nicht gefunden")
        .f64()
        .expect("Spalte ist nicht vom Typ f64")
        .into_no_null_iter()
        .collect::<Vec<f64>>();

    json!({
        "salary": {
            "mean": mean(&salary),
            "median": median(&salary),
            "std_dev": std_dev(&salary),
            "min": salary.iter().cloned().fold(f64::INFINITY, f64::min),
            "max": salary.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        },
        "experience": {
            "mean": mean(&experience),
            "median": median(&experience),
            "std_dev": std_dev(&experience),
            "min": experience.iter().cloned().fold(f64::INFINITY, f64::min),
            "max": experience.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        },
    })
}

pub fn calculate_distribution(df: &DataFrame) -> serde_json::Value {
    let salary_col = "Yearly brutto salary (without bonus and stocks) in EUR";

    let salaries = df
        .column(salary_col)
        .expect("Spalte nicht gefunden")
        .f64()
        .expect("Spalte ist nicht vom Typ f64")
        .into_no_null_iter()
        .collect::<Vec<f64>>();

    let min_sal = salaries.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_sal = salaries.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let bin_count = 10;
    let bin_size = (max_sal - min_sal) / bin_count as f64;
    let mut freq = vec![0; bin_count];

    for &val in salaries.iter() {
        let mut idx = ((val - min_sal) / bin_size).floor() as isize;
        if idx < 0 {
            idx = 0;
        }
        if idx as usize >= bin_count {
            idx = bin_count as isize - 1;
        }
        freq[idx as usize] += 1;
    }

    let bins = (0..bin_count)
        .map(|i| min_sal + i as f64 * bin_size)
        .collect::<Vec<f64>>();

    json!({
        "bins": bins,
        "frequencies": freq,
    })
}

/// Hilfsfunktion: Mittelwert berechnen
fn mean(data: &Vec<f64>) -> f64 {
    data.iter().sum::<f64>() / data.len() as f64
}

/// Hilfsfunktion: Median berechnen
fn median(data: &Vec<f64>) -> f64 {
    let mut sorted = data.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

/// Hilfsfunktion: Standardabweichung berechnen
fn std_dev(data: &Vec<f64>) -> f64 {
    let mean_val = mean(data);
    (data.iter().map(|&x| (x - mean_val).powi(2)).sum::<f64>() / data.len() as f64).sqrt()
}