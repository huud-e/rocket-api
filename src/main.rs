use rocket::response::Redirect;
use rocket::http::{Cookie, CookieJar};
use rocket::serde::{Deserialize, json::Json};
use rocket::form::Form;
use rocket_dyn_templates::Template;
use std::collections::HashMap;


#[macro_use] extern crate rocket;

#[derive(Deserialize)]
struct Stock<'r> {
    name: &'r str
}

#[derive(FromForm)]
struct SStock<'r> {
    name: &'r str
}

#[get("/")]
fn index() -> Template {
    let context: HashMap<&str, &str> = HashMap::new();
    Template::render("index", &context)
}

#[get("/<stock>")]
fn stock(stock: String) -> String {
    format!("Stock is {}", stock)
}

#[post("/sstock", data = "<user_input>")]
fn ssearch(user_input: Form<SStock>) -> Redirect { 
    Redirect::to(uri!(stock(user_input.name)))
}

#[post("/", data = "<stock>")]
fn submit(cookies: &CookieJar<'_>, stock: Json<Stock<'_>>) -> Redirect {
    cookies.add(Cookie::new(format!("{}",stock.name),format!("{}",stock.name)));
    Redirect::to(uri!(stock(stock.name)))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![stock])
        .mount("/", routes![submit])
        .mount("/", routes![ssearch])
        .attach(Template::fairing())
}