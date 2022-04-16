#![allow(unused)]
use dotenv::dotenv;
use std::env;
use serde_json::{Value};
/*
use std::fs::File;                                                                                                                                                                   
use std::io::Write;                                                                                                                                                                  
*/
use std::collections::HashMap;
use std::{error::Error, time::Duration};
use tokio::time::sleep;
use redis::{
    from_redis_value,
    streams::{StreamRangeReply, StreamReadOptions, StreamReadReply},
    AsyncCommands, Client,
};

pub async fn exists_in_redis(stock: &String) -> Result<bool, Box<dyn Error>> {
    let client = Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_tokio_connection().await?;
    let mut exists = false;
    let result: Option<StreamRangeReply> = con.xrevrange_count("redis_stream", "+", "-", 10).await?;
	if let Some(reply) = result {
		for stream_id in reply.ids {
			for (name, value) in stream_id.map.iter() {                              
                // VPT
                let strings: Vec<&str> = name.split("vpt").collect();
                let nvpt: &str = strings[0];
                if nvpt == stock {
                    exists = true;
                }
                // OBV
                let strings: Vec<&str> = stock.split("obv").collect();
                let nobv: &str = strings[0];
                
                if nobv == name {
                    exists = true;
                }
                
			}
		}
	}
    Ok(exists)
}

pub async fn add_to_redis(stock: &String) -> Result<(), Box<dyn Error>> {

    let client = Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_tokio_connection().await?;
    
    let body = reqwest::get(url(stock.to_string())).await.unwrap().text().await.unwrap();
    let res = analize(body);

    sleep(Duration::from_millis(100)).await;
    con.xadd("redis_stream", "*", &[(format!("{}vpt",stock), format!("{}", res[0])),(format!("{}obv",stock), format!("{}", res[1]))]).await?;

    Ok(())
}
pub async fn return_of_redis(stock: &String) -> Result<Vec<String>, Box<dyn Error>> {
    let client = Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_tokio_connection().await?;
    let mut res: Vec<String> = Vec::new();
    let mut vpt: String = "Error: Coudn't parse vpt of stock".to_string();
    let mut obv: String = "Error: Coudn't parse obv of stock".to_string();
    let result: Option<StreamRangeReply> = con.xrevrange_all("redis_stream").await?;
	if let Some(reply) = result {
		for stream_id in reply.ids {
			for (name, value) in stream_id.map.iter() {
                let strings: Vec<&str> = name.split("vpt").collect();
                let nvpt: &str = strings[0];
                if nvpt == stock {
                    vpt = from_redis_value::<String>(value)?;
                }
                let strings: Vec<&str> = name.split("obv").collect();
                let nvpt: &str = strings[0];
                if nvpt == stock {
                    obv = from_redis_value::<String>(value)?;
                } 
                               
			}
		}
        res.push(obv);
        res.push(vpt);
	}
    
    Ok(res)
}
fn return_stock_values(v: &Value, opt: u8) -> Vec<serde_json::value::Value> {
    let mut vec: Vec<serde_json::value::Value> = Vec::new();
    for (_key, value) in v.as_object().unwrap() {
        for (_key2,value2) in value.as_object().unwrap() {
            // key 2022-04-05 value Object
            match opt {
                1 => vec.push(value2["1. open"].clone()),
                2 => vec.push(value2["2. high"].clone()),
                3 => vec.push(value2["3. low"].clone()),
                4 => vec.push(value2["4. close"].clone()),
                5 => vec.push(value2["5. volume"].clone()),
                _ => (),
            }
        }
    }
    return vec;
}
pub fn url(stock: String) -> String {
    dotenv().ok();
    let api_key = env::var("API_KEY")
        .expect("API_KEY must be set");
    let url: String = format!("https://www.alphavantage.co/query?function=TIME_SERIES_DAILY&symbol={URL}&outputsize=full&apikey=", URL= stock);
    let url = url.to_owned();
    let key: String = api_key.to_owned();
    let furl = format!("{}{}", url,key);
    return furl;
}

fn volume_price_trend(stock: &String) -> Result<f32, Box<dyn Error>> {
    // VPT = Volume x (Today’s Closing Price – Previous Closing Price) / Previous Closing Price
    let v: Value = serde_json::from_str(&stock)?;
    
    
    let vol = return_stock_values(&v, 5);   
    let clo = return_stock_values(&v, 4);
    let pclo = return_stock_values(&v, 4);

    
    let mut i = vol.len() - 2;
    let mut ii = vol.len() - 1;
    
    let mut vpt = 0.0;
    let mut pvt = 0.0;

    let mut vpts = Vec::new(); 

    while i > 4{

        // println!("{}{}{}",vol[i], clo[i], i);
        let vol = vol[i].as_str().unwrap();
        let vol: f32 = vol.parse().unwrap();

        let clo = clo[i].as_str().unwrap();
        let clo: f32 = clo.parse().unwrap();
        
        let pclo = pclo[ii].as_str().unwrap();
        let pclo: f32 = pclo.parse().unwrap();
        
        vpt = pvt + vol * ( clo - pclo ) / pclo;
        pvt = vpt;
        
        vpts.push(vpt);

        i = i - 1;
        ii = ii - 1;
        
    }

    Ok(vpt)
}

fn on_balance_volume(stock: &String) -> Result<f32, Box<dyn Error>>{
    let v: Value = serde_json::from_str(&stock)?;
    
    let vol = return_stock_values(&v, 5);   
    let clo = return_stock_values(&v, 4);
    let pclo = return_stock_values(&v, 4);

    let mut i = vol.len() - 2;
    let mut ii = vol.len() - 1;
    
    let mut obv = 0.0;
    let mut pobv = 0.0;

    let mut fobv = Vec::new(); 
    
    while i > 4{

        // println!("{}{}{}",vol[i], clo[i], i);
        let vol = vol[i].as_str().unwrap();
        let vol: f32 = vol.parse().unwrap();

        let clo = clo[i].as_str().unwrap();
        let clo: f32 = clo.parse().unwrap();
        
        let pclo = pclo[ii].as_str().unwrap();
        let pclo: f32 = pclo.parse().unwrap();
         
        if clo > pclo {
            obv = pobv + vol;
        }else if clo < pclo {
            obv = pobv - vol;
        }else if clo == pclo{
            obv = pobv;
        }

        pobv = obv;
        fobv.push(obv);

        i = i - 1;
        ii = ii - 1;
        
    }
    Ok(obv)

}


pub fn analize(body:String) -> Vec<f32>{
    let mut result= Vec::new();
    
    let vpt = volume_price_trend(&body).unwrap();
    result.push(vpt);
    
    let obv = on_balance_volume(&body).unwrap();
    result.push(obv);
    
    return result;
}