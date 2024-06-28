use tera::{Context, Tera};
use warp::Filter;

#[tokio::main]
async fn main() {
    // Initialize Tera template engine
    let mut tera = Tera::default();

    // Embed templates
    tera.add_raw_template("form.html", FORM_TEMPLATE)
        .expect("Error adding form template");
    tera.add_raw_template("result.html", RESULT_TEMPLATE)
        .expect("Error adding result template");

    // Serve the HTML form
    let tera_filter = warp::any().map(move || tera.clone());

    let form_route = warp::path::end()
        .and(tera_filter.clone())
        .map(|tera: Tera| {
            let rendered = tera
                .render("form.html", &Context::new())
                .expect("Error rendering template");
            warp::reply::html(rendered)
        });

    // Serve the CSS stylesheet
    let css_route =
        warp::path("styles.css").map(|| warp::reply::with_header(CSS, "content-type", "text/css"));

    // Handle form submissions
    let submit_route = warp::path("submit")
        .and(warp::post())
        .and(warp::body::form())
        .and(tera_filter)
        .map(
            |form_data: std::collections::HashMap<String, String>, tera: Tera| {
                let question = form_data
                    .get("question")
                    .unwrap_or(&"".to_string())
                    .to_string();
                let mut context = Context::new();
                context.insert("question", &question);
                context.insert("answer", "42");
                let rendered = tera
                    .render("result.html", &context)
                    .expect("Error rendering template");
                warp::reply::html(rendered)
            },
        );

    // Combine the routes
    let routes = form_route.or(css_route).or(submit_route);

    // Start the warp server
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

// Form HTML template
const FORM_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="en">
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
	<title>Ask me anything</title>
	<link rel="stylesheet" type="text/css" href="styles.css">
</head>
<body>
      <script>
      const myButton = document.getElementById('button');
      myButton.addEventListener('keypress', (event) => {
        if (event.key === 'Enter') {
          myButton.submit();
        }
      });
      const myTextarea = document.getElementById('question');
      myTextarea.addEventListener('keypress', (event) => {
        if (event.key === 'Enter') {
          event.preventDefault();
          // Submit the form here, e.g. using a submit button or a JavaScript function
          document.getElementById('button').submit();
        }
      });
      </script>

	<h1>Ask me anything</h1>
        <div>
            This service will take your question and use an LLM to derive search topics
            for Wikipedia. It will then search some Wikipedia pages and use the LLM again
            to answer your question using the information in the Wikipedia pages. Finally,
            it will give you some references.
        </div>
            <form action="/submit" method="post">
		<label for="name">Enter your question:</label>
                <input type="text" id="question" name="question"><br><br>
		<input type="submit" id="button" value="Submit">
	</form>
</body>
</html>
"#;

// Result HTML template
const RESULT_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="en">
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
	<title>Ask me anything</title>
	<link rel="stylesheet" type="text/css" href="styles.css">
</head>
<body>
	<h1>Ask me anything</h1>
        <div>
            Your question was: {{question}}
        </div>
        <div>
            The answer is: {{answer}}
        </div>
        <div>
          <a href="/">Back to question form.</a>
        </div>
	</form>
</body>
</html>
"#;

// CSS stylesheet
const CSS: &str = r#"
body {
	background-color: #ADD8E6; /* light blue */
	font-family: Arial, sans-serif;
        font-size: 30px;
}

h1 {
	text-align: center;
	margin-top: 48px;
}

div {
    margin: 0 auto;
    width: 50%;
    text-align: justify;
}

form {
	margin-top: 20px;
	border: 1px solid #ccc;
	padding: 10px;
	width: 50%;
	margin: 0 auto;
	display: table; /* center the form vertically */
        font-size: 30px;
}

#button {
    width: 128px;
    height: 48px;
}

#question {
	font-size: 24px;
        columns: 80;
        width: 80%;
}
"#;
