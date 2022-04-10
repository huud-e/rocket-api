//use rocket::response::Redirect;
//use rocket::http::{Cookie, CookieJar};
use rocket::serde::{Serialize, Deserialize, json::Json};
//use rocket::form::Form;
use rocket_dyn_templates::Template;
use std::collections::HashMap;

#[macro_use] extern crate rocket;
#[derive(Serialize, Deserialize, FromForm, Debug)]
struct Stock{
    name: String
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

#[post("/", format="application/x-www-form-urlencoded",data="<stock>")]
fn submit(stock: String) -> String {
    // cookies: &CookieJar<'_>,
    // cookies.add(Cookie::new(format!("{}",&stock.name),format!("{}",&stock.name)));
    
    
    //if stock.name.is_empty() {
    //    Redirect::to(uri!(stock(format!("NO"))))
    //} else {
        format!("Stock is {}",stock)
        //Redirect::to(uri!(stock(split)))
    //}
    
}
#[post("/", data = "<stock>", rank=2)]
fn submit2(stock: Json<Stock>) -> String {
    format!("{}", &stock.name)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/", routes![stock])
        .mount("/", routes![submit])
        .mount("/", routes![submit2])
        .attach(Template::fairing())
}