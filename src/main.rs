use rocket::response::Redirect;
use rocket::http::{Cookie, CookieJar};
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use serde_json::json;
use json_value_merge::Merge;

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_include_static_resources;

mod middleware;
use middleware::*;

static_response_handler! {
    "/favicon.ico" => favicon => "favicon",
    "/model.json" => model => "model",
    "/group1-shard1of1.bin" => group1_shard1of1 => "group1-shard1of1",
    "/output" => output => "output",
    "/output2" => output2 => "output2",
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
        if parsed == "{\"error\":{\"code\":\"no_valid_symbols_provided\",\"message\":\"At least one valid symbol must be provided\"}}"{
            println!("Invalid API CALL");
        }else{
            cookies.add(Cookie::new(format!("{}", stock),format!("{}", stock)));
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
#[get("/to_file/<stock>")]
async fn to_file(stock: String) -> Result<Redirect,std::io::Error> {

    let body = reqwest::get(url(stock)).await.unwrap().text().await.unwrap();
    match write_to_file(&body){
        Err(e) => println!("{:?}", e),
        _ => (),
    }
    Ok(Redirect::to(uri!(index)))
}

#[get("/predict/<stock>/<data>")]
async fn predict(stock:String, data: String) -> Result<Redirect, std::io::Error>{
    let vec = url_predict(stock);

    let a = reqwest::get(&vec[0]).await.unwrap().text().await.unwrap();
    let b = reqwest::get(&vec[1]).await.unwrap().text().await.unwrap();

    let mut a1: serde_json::Value = serde_json::from_str(&a).unwrap();
    let b1: serde_json::Value = serde_json::from_str(&b).unwrap();
    a1.merge(b1);

    
    let strings: Vec<&str> = data.split(",").collect();
    let vpt = strings[0].to_string();
    let obv = strings[1].to_string().replace(" ", "");


    match write_to_file_predict(&a1.to_string(), &vpt, &obv){
        Err(e) => println!("{:?}", e),
        _ => (),
    }

    println!("Correcto!");
    Ok(Redirect::to(uri!(index)))
}

#[launch]
fn rocket() -> _ {
    
    rocket::build()
        .attach(Template::fairing())
        .attach(static_resources_initializer!(
            "favicon" => "static/favicon.ico",
            "model" => "model/model.json",
            "group1-shard1of1" => "model/group1-shard1of1.bin",
            "output" => "tmp/output",
            "output2" => "predict/output2",
        ))
        .mount("/", routes![favicon, model,group1_shard1of1, output, output2])
        .mount("/", routes![index, submit, login, logout, predict, to_file])
}