use std::io::Write;
use std::process::{Command, Stdio};
use tera::{Context, Tera};
use warp::Filter;

// Form HTML template
const FORM_TEMPLATE: &str = include_str!("form_template.html");

// Result HTML template
const RESULT_TEMPLATE: &str = include_str!("result_template.html");

// CSS stylesheet
const CSS: &str = include_str!("css.css");

#[derive(Debug)]
struct Answer {
    pub answer: String,
    pub output: String,
    pub references: Vec<String>,
}

fn call_wikirag(question: &str, model: &str, wikipages: &str) -> Result<Answer, String> {
    // Check `wikipages` is a string with a sensible number, otherwise take 1!
    let mut child = Command::new("wikirag")
        .env("AI_MODEL", model.to_string())
        .env("WIKI_PAGES", wikipages.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to execute child");
    let mut stdin = child.stdin.take().expect("failed to get stdin");
    let question = question.to_string() + "\n";
    std::thread::spawn(move || {
        stdin
            .write_all(question.as_bytes())
            .expect("failed to write to stdin");
    });

    let output = child.wait_with_output().expect("failed to wait on child");
    let mut answer = Answer {
        answer: std::str::from_utf8(&output.stdout).unwrap().to_string(),
        output: std::str::from_utf8(&output.stderr).unwrap().to_string(),
        references: vec![],
    };
    let pos = answer.answer.as_str().find("***Links***:\n");
    if let Some(pos) = pos {
        answer.references = answer.answer[pos + 13..answer.answer.len()]
            .split('\n')
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect();
        answer.answer = answer.answer[0..pos].to_string();
    }
    Ok(answer)
}

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
                let model = form_data
                    .get("model")
                    .unwrap_or(&"llama3".to_string())
                    .to_string();
                let pages = form_data
                    .get("wikipages")
                    .unwrap_or(&"1".to_string())
                    .to_string();
                let answer = call_wikirag(&question, &model, &pages).expect("should not go wrong");
                let mut context = Context::new();
                context.insert("question", &question);
                context.insert("answer", &answer.answer);
                context.insert("output", &answer.output);
                context.insert("links", &answer.references);
                let rendered = tera
                    .render("result.html", &context)
                    .expect("Error rendering template");
                warp::reply::html(rendered)
            },
        );

    // Combine the routes
    let routes = form_route.or(css_route).or(submit_route);

    // Start the warp server
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
