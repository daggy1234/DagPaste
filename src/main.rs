#![feature(proc_macro_hygiene, decl_macro)]
mod counter;
use rocket_prometheus::PrometheusMetrics;
mod id;
#[macro_use]
extern crate serde_derive;

use counter::{Counter, Token};
use id::{valid_id, PasteID};
use rocket::fairing::AdHoc;
use rocket::http::{Method, Status};
use rocket::request::{FlashMessage, Form};
use rocket::response::{status, Flash, Redirect};
use rocket::FromForm;
use rocket::{catch, catchers, get, post, routes, Data};
use rocket_contrib::{json::Json, serve::StaticFiles, templates::Template};
use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Serialize)]
struct Context<'a, 'b> {
    msg: Option<(&'a str, &'b str)>,
}

impl<'a, 'b> Context<'a, 'b> {
    pub fn err(msg: &'a str) -> Context<'static, 'a> {
        Context {
            msg: Some(("error", msg)),
        }
    }

    pub fn raw(msg: Option<(&'a str, &'b str)>) -> Context<'a, 'b> {
        Context { msg: msg }
    }
}

#[derive(FromForm, Debug, Serialize, Deserialize)]
pub struct CodeOut {
    pub code: String,
    pub file: String,
    pub ext: String,
    pub islang: bool,
}

#[derive(FromForm, Debug, Serialize, Deserialize)]
pub struct Code {
    pub code: String,
}

#[get("/")]
fn index() -> &'static str {
    r#"
        Welcome too the DagPaste a revolutionary new paste service thats fast and open source!\n 
            
        The Following API routes exist for users to interact with the service
        
        BROWSER USEAGE:
            
            https://dagbot.daggy.tech/web
        
        API REFERENCE
            
            POST https://paste.daggy.tech
                  If the response code is 200, then the entire paste was
                  uploaded. If the response is 206 (PARTIAL), then the paste exceeded
                  the server's maximum upload size, and only part of the paste was
                  uploaded. If the response code is anything else, an error has
                  occurred
            
            GET https://paste.daggy.tech/<id>.<format>
                Get the HTML web viewer for a paste with syntax higlighting of the 
                specified format (if a format was specified)
            
            GET https://paste.daggy.tech/raw/<id>
                get a raw plain tetx response of a paste with no sntax highlighting
            
            GET https://paste.daggy.tech/document/<id>
                get a Json response with the title for a particular document.
            "#
}
#[get("/robots.txt")]
fn rbot_txt() -> &'static str {
    "User-agent: *\nAllow: /"
}

#[get("/web")]
fn web_ui(msg: Option<FlashMessage>) -> Template {
    Template::render(
        "index",
        &match msg {
            Some(ref msg) => Context::raw(Some((msg.name(), msg.msg()))),
            None => Context::raw(None),
        },
    )
}
#[derive(Serialize, Deserialize)]
struct Out {
    file: String,
    length: String,
}

#[derive(Serialize, Deserialize)]
struct ErrorMessage {
    message: String,
}
#[catch(404)]
fn not_found(req: &rocket::Request) -> Json<ErrorMessage> {
    Json(ErrorMessage {
        message: format!("{} is not found", req.uri()),
    })
}

#[catch(500)]
fn erar(_req: &rocket::Request) -> Json<ErrorMessage> {
    Json(ErrorMessage {
        message: format!("Internal Server Error"),
    })
}

#[post("/form", data = "<form>")]
fn upload_form(form: Form<Code>) -> Flash<Redirect> {
    println!("{:?}", form);
    let code = form.into_inner();
    if code.code.is_empty() {
        Flash::error(Redirect::to("/web"), "Code should not be empty")
    } else {
        let rand_string = PasteID::generate();
        let filename = format!("pastes/{}", rand_string);
        let mut file = File::create(Path::new(&filename)).unwrap();
        file.write_all(code.code.as_bytes()).unwrap();
        Flash::success(
            Redirect::to(format!("/{}", rand_string)),
            "Code should not be empty",
        )
    }
}

#[post("/upload", format = "plain", data = "<data>")]
fn upload(data: Data) -> Json<Out> {
    let rand_string = PasteID::generate();
    let filename = format!("pastes/{}", rand_string);
    let out = data
        .stream_to_file(Path::new(&filename))
        .map(|n| n.to_string())
        .unwrap();
    Json(Out {
        file: format!("https://paste.daggy.tech/{}", rand_string),
        length: out,
    })
}

// #[get("/<id>")]
// fn retrieve(id: id::PasteID) -> Template {
//     let file = format!("./pastes/{}", id);
//     let s = fs::read_to_string(file)
//         .unwrap()
//         .replace("&", "&amp;")
//         .replace(">", "&gt;")
//         .replace("<", "&lt")
//         .replace("\"", "&quot;");
//
//     let context = Code { code: s };
//     Template::render("main", &context)
//}

#[get("/<phrase>")]
fn retrieve(phrase: String) -> Template {
    let out = phrase.split(".").collect::<Vec<&str>>();
    println!("{}", valid_id(out[0]));
    if !valid_id(out[0]) {
        Template::render("index", Context::raw(None))
    } else {
        let file = format!("./pastes/{}", out[0]);
        let s = fs::read_to_string(file).unwrap();
        println!("{:#?}", s);
        let mut context = CodeOut {
            code: s.to_string(),
            file: out[0].to_string(),
            ext: "".to_string(),
            islang: false,
        };
        if out.len() == 2 {
            context = CodeOut {
                code: s,
                file: out[0].to_string(),
                ext: out[1].to_string(),
                islang: true,
            };
        }
        Template::render("main", &context)
    }
}

#[get("/document/<id>")]
fn retrieve_doc(id: id::PasteID) -> Json<CodeOut> {
    let file = format!("./pastes/{}", id);
    let s = fs::read_to_string(&file).unwrap();
    Json(CodeOut {
        code: s,
        file: format!("{}", id),
        ext: "".to_string(),
        islang: false,
    })
}

#[get("/raw/<id>")]
fn retrieve_raw(id: id::PasteID) -> String {
    let file = format!("./pastes/{}", id);
    let s = fs::read_to_string(&file).unwrap();
    s
}

fn main() {
    let prometheus = PrometheusMetrics::new();
    rocket::ignite()
        .attach(prometheus.clone())
        .attach(Counter::default())
        .attach(AdHoc::on_attach("Token State", |rocket| {
            println!("Adding token managed state...");
            let token_val = rocket.config().get_int("token").unwrap_or(-1);
            Ok(rocket.manage(Token(token_val)))
        }))
        .attach(AdHoc::on_launch("Launch Message", |_| {
            println!("Rocket is about to launch!");
        }))
        .attach(AdHoc::on_request("PUT Rewriter", |req, _| {
            println!("    => Incoming request: {}", req);
        }))
        .attach(AdHoc::on_response("Response Rewriter", |req, res| {}))
        .mount("/metrics", prometheus)
        .mount(
            "/",
            routes![
                rbot_txt,
                index,
                upload,
                retrieve,
                retrieve_doc,
                retrieve_raw,
                upload_form,
                web_ui
            ],
        )
        .mount("/", StaticFiles::from("static/"))
        .register(catchers![not_found, erar])
        .attach(Template::fairing())
        .launch();
}
