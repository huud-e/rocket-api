use rocket::response::Redirect;
use rocket::http::{Cookie, CookieJar};
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use serde_json::json;

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_include_static_resources;

mod vpt;
use vpt::*;

static_response_handler! {
    "/favicon.ico" => favicon => "favicon",
}

#[get("/")]
async fn index(cookies: &CookieJar<'_> ) -> Result<Template, std::io::Error> {
    let mut target: HashMap<String, Vec<String>> = HashMap::new();
    for c in cookies.iter() {
        if c.name().to_string() != "admin"{
            if !exists_in_redis(&c.name().to_string()).await.unwrap() {
                match add_to_redis(&c.value().to_string()).await {
                    Err(e) => println!("{:?}", e),
                    _ => ()
                }
                target.insert(format!("{}", c.name()), return_of_redis(&c.name().to_string()).await.unwrap());
                println!("Doesnt exist in redis db, adding stock: {}", c.name().to_string());
            }else{
                target.insert(format!("{}", c.name()), return_of_redis(&c.name().to_string()).await.unwrap());
            }
        }
    }

    let mut vec: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
    vec.insert(format!("stocks"), target);

    Ok(Template::render("index", vec))
}


#[post("/", format="application/x-www-form-urlencoded",data="<stock>")]
async fn submit(stock: String, cookies: &CookieJar<'_> ) -> Result<Redirect,std::io::Error> {
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

#[get("/login")]
fn login(cookies: &CookieJar<'_>) -> Redirect{
    cookies.add_private(Cookie::new(format!("admin"),format!("true")));
    Redirect::to(uri!(index))
}

#[get("/logout")]
fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove_private(Cookie::named("admin"));
    Redirect::to(uri!(index))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Template::fairing())
        .attach(static_resources_initializer!(
            "favicon" => "static/favicon.ico",
        ))
        .mount("/", routes![favicon])
        .mount("/", routes![index, submit, login, logout])
}