use rocket::response::Redirect;
use rocket::http::{Cookie, CookieJar};
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use rocket_sync_db_pools::{database};
use serde_json::json;

#[macro_use]
extern crate rocket;
/*
#[macro_use]
extern crate diesel;
*/

#[macro_use]
extern crate rocket_include_static_resources;

mod vpt;
use vpt::*;

static_response_handler! {
    "/favicon.ico" => favicon => "favicon",
}

#[database("stocks-db")]
struct StocksDbConn(diesel::PgConnection);

#[get("/")]
fn index() -> Template {
    let context: HashMap<&str, &str> = HashMap::new();
    Template::render("index", &context)
}

#[get("/api/<stock>")]
async fn stock(stock: String) -> Result<Template, std::io::Error> {

    let body = reqwest::get(url(stock.to_owned())).await.unwrap().text().await.unwrap();
    let parsed = json!(&body);
    let mut target: HashMap<String,String> = HashMap::new();
    
    match volumepricetrend(body.to_owned()) {
        Err(e) => println!("{:?}", e),
        _ => ()
    }
    if parsed == "{\n    \"Error Message\": \"Invalid API call. Please retry or visit the documentation (https://www.alphavantage.co/documentation/) for TIME_SERIES_DAILY.\"\n}"{
        println!("Error!");
        // context.insert("Error".to_string(), format!("Stock: {}, not found in API!", &stock));
        let context: String = format!("Error, stock: {}, doesnt exist in API!", stock);
        target.insert("e".to_string(), context);
        Ok(Template::render("stock", target))
        
    }else{
        println!("Todo bien!");
        let context: String = format!("Good, stock: {}, exists in API!", stock);
        target.insert("v".to_string(), context);
        Ok(Template::render("stock", target))
    }
}

#[post("/", format="application/x-www-form-urlencoded",data="<stock>")]
fn submit(stock: String, cookies: &CookieJar<'_>, _conn: StocksDbConn) -> Redirect {
    let strings: Vec<&str> = stock.split("name=").collect();
    let stock: &str = strings[1];
    cookies.add(Cookie::new(format!("{}",stock),format!("{}",stock)));
    Redirect::to(uri!(stock(stock)))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Template::fairing())
        .attach(StocksDbConn::fairing())
        .attach(static_resources_initializer!(
            "favicon" => "static/favicon.ico",
        ))
        .mount("/", routes![favicon])
        .mount("/", routes![index, stock, submit])

}