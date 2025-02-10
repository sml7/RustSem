use plotters::prelude::*;

pub fn create_salary_histogram(salaries: &[f32]) -> Vec<u8> {
    let width = 800;
    let height = 600;
    let mut buffer = vec![0u8; (width * height * 4) as usize];

    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height))
            .into_drawing_area();
        root.fill(&WHITE).unwrap();

        let max_sal = salaries.iter().fold(0.0_f32, |acc, &x| acc.max(x));
        let min_sal = salaries.iter().fold(f32::MAX, |acc, &x| acc.min(x));

        let mut chart = ChartBuilder::on(&root)
            .caption("Salary Histogram", ("sans-serif", 20).into_font())
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(min_sal..max_sal, 0..(salaries.len()))
            .unwrap();

        chart.configure_mesh()
            .x_desc("Gehalt (EUR)")
            .y_desc("Häufigkeit")
            .draw()
            .unwrap();

        let bin_count = 10;
        let bin_size = (max_sal - min_sal) / bin_count as f32;
        let mut freq = vec![0; bin_count];

        for &val in salaries {
            let mut idx = ((val - min_sal) / bin_size).floor() as isize;
            if idx < 0 { idx = 0; }
            if idx as usize >= bin_count { idx = bin_count as isize -1; }
            freq[idx as usize] += 1;
        }

        for (i, &count) in freq.iter().enumerate() {
            let x0 = min_sal + i as f32 * bin_size;
            let x1 = x0 + bin_size;
            chart
                .draw_series(std::iter::once(Rectangle::new(
                    [(x0, 0), (x1, count)],
                    RED.filled(),
                )))
                .unwrap();
        }
    }

    buffer
}
