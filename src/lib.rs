use polars::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct HistogramData {
    bins: Vec<f64>,   // X-Werte (z. B. Gehaltsklassen)
    counts: Vec<u32>, // Y-Werte (Häufigkeit in jeder Klasse)
}

pub fn calculate_histogram(df: &DataFrame, column_name: &str, bin_size: f64) -> Result<HistogramData, PolarsError> {
    let col = df.column(column_name)?.f64()?.into_no_null_iter().collect::<Vec<f64>>();
    let max_val = col.iter().cloned().fold(0.0_f64, f64::max);
    let bins_count = (max_val / bin_size).ceil() as usize;
    let mut bins = vec![0.0; bins_count + 1];
    let mut counts = vec![0; bins_count];

    for &value in &col {
        let index = (value / bin_size).floor() as usize;
        if index < bins_count {
            counts[index] += 1;
        }
    }

    for i in 0..=bins_count {
        bins[i] = i as f64 * bin_size;
    }

    Ok(HistogramData { bins, counts })
}


