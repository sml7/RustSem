async fn show_predict(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
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
