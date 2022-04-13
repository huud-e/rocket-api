use rocket::response::Redirect;
use rocket::http::{Cookie, CookieJar};
use rocket_dyn_templates::Template;
use std::collections::HashMap;
// use rocket_sync_db_pools::{database};
use serde_json::json;

#[macro_use] extern crate rocket;
/*
#[macro_use]
extern crate diesel;
*/

#[macro_use] extern crate rocket_include_static_resources;

mod vpt;
use vpt::*;

static_response_handler! {
    "/favicon.ico" => favicon => "favicon",
}
/*
#[database("rocket-db")]
struct StocksDbConn(diesel::PgConnection);
 */
#[get("/")]
async fn index(cookies: &CookieJar<'_> ) -> Result<Template, std::io::Error> {
    let mut target: HashMap<String, Vec<f32>> = HashMap::new();
    
    for c in cookies.iter() {
        let body = reqwest::get(url(c.name().to_string())).await.unwrap().text().await.unwrap();
        target.insert(format!("{}", c.name()), analize(body));
    }
    let mut vec: HashMap<String, HashMap<String, Vec<f32>>> = HashMap::new();
    vec.insert(format!("stocks"), target);

    Ok(Template::render("index", vec))
}


#[post("/", format="application/x-www-form-urlencoded",data="<stock>")]
async fn submit(stock: String, cookies: &CookieJar<'_> ) -> Result<Redirect,std::io::Error> {
    // _conn: StocksDbConn
    let strings: Vec<&str> = stock.split("name=").collect();
    let stock: &str = strings[1];

    let mut bool = false;
    for c in cookies.iter(){
        if c.name().to_string() == stock{
            bool = true;
        }
    }
    if bool == false{
        let body = reqwest::get(url(stock.to_owned())).await.unwrap().text().await.unwrap();
        let parsed = json!(&body);
        let mut target: HashMap<String,String> = HashMap::new();
        if parsed == "{\n    \"Error Message\": \"Invalid API call. Please retry or visit the documentation (https://www.alphavantage.co/documentation/) for TIME_SERIES_DAILY.\"\n}"{
            println!("Invalid API CALL");
        }else{
            cookies.add(Cookie::new(format!("{}", stock),format!("{}", stock)));
            let context: String = format!("Good, stock: {}, exists in API!", stock);
            target.insert("v".to_string(), context);
            println!("Valid API CALL"); 
        }
    }
    Ok(Redirect::to(uri!(index)))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Template::fairing())
        // .attach(StocksDbConn::fairing())
        .attach(static_resources_initializer!(
            "favicon" => "static/favicon.ico",
        ))
        .mount("/", routes![favicon])
        .mount("/", routes![index, submit])
}