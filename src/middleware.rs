#![allow(unused)]
use dotenv::dotenv;
use std::env;
use serde_json::{Value};
use std::collections::HashMap;
use std::{error::Error, time::Duration};
use tokio::time::sleep;
use redis::{
    from_redis_value,
    streams::{StreamRangeReply, StreamReadOptions, StreamReadReply},
    AsyncCommands, Client,
};
use std::{
    fs::File,
    io::{BufWriter, Write},
};
use chrono::{DateTime, Utc, Duration as Duration2};

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
    let res = analize(&body).unwrap();
    // write_to_file(&body);

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
    for (key, value) in v.as_object().unwrap() {
        if key == "data" {
            let val = value.as_array().unwrap();
            for i in val{
                match opt{
                    1=> vec.push(i["open"].clone()),
                    2=> vec.push(i["high"].clone()),
                    3=> vec.push(i["low"].clone()),
                    4=> vec.push(i["close"].clone()),
                    5=> vec.push(i["volume"].clone()),
                    _ => ()
                }
            }
        }
    }
return vec;
}


pub fn url(stock: String) -> String {
    dotenv().ok();
    let api_key = env::var("API_KEY")
        .expect("API_KEY must be set");
    let url: String = format!("https://api.marketstack.com/v1/eod?access_key={KEY}&symbols={URL}&date_from=2002-04-21&date_to=2022-04-21&limit=10000", URL= stock, KEY= api_key);
    
    return url;
}

pub fn url_predict(stock: String) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    dotenv().ok();
    let api_key = env::var("API_KEY")
        .expect("API_KEY must be set");
        
        
    let today: DateTime<Utc> = Utc::now() - Duration2::days(1);;
    let url: String = format!("https://api.marketstack.com/v1/eod/{DATE}?access_key={KEY}&symbols={URL}", URL= stock, KEY= api_key, DATE=today.format("%Y-%m-%d"));
    res.push(url);
    
    let yesterday: DateTime<Utc> = Utc::now() - Duration2::days(2);
    let url2: String = format!("https://api.marketstack.com/v1/eod/{DATE}?access_key={KEY}&symbols={URL}", URL= stock, KEY= api_key, DATE=yesterday.format("%Y-%m-%d"));
    res.push(url2);
    return res;
}

fn volume_price_trend(stock: &String) -> Result<Vec<f64>, Box<dyn Error>> {
    let v: Value = serde_json::from_str(&stock)?;
    
    let vol = return_stock_values(&v, 5);   
    let clo = return_stock_values(&v, 4);
    let pclo = &clo;

    let mut i = vol.len() - 2;
    let mut ii = vol.len() - 1;
    let mut vpt = 0.0;
    let mut pvt = 0.0;

    let mut vpts = Vec::new(); 

    while i > 0{

        let volum = vol[i].as_f64().unwrap();
        let close = clo[i].as_f64().unwrap();        
        let pclose = pclo[ii].as_f64().unwrap();
        
        vpt = pvt + volum * ( close - pclose ) / pclose;
        pvt = vpt;
        
        vpts.push(vpt);

        i = i - 1;
        ii = ii - 1;
        
    }
    
    Ok(vpts)
}
fn on_balance_volume(stock: &String) -> Result<Vec<f64>, Box<dyn Error>>{
    let v: Value = serde_json::from_str(&stock)?;
    
    let vol = return_stock_values(&v, 5);   
    let clo = return_stock_values(&v, 4);
    let pclo = &clo;

    let mut i = vol.len() - 2;
    let mut ii = vol.len() - 1;
    let mut obv = 0.0;
    let mut pobv = 0.0;

    let mut fobv = Vec::new(); 
    
    while i > 0{

        let volum = vol[i].as_f64().unwrap();
        let close = clo[i].as_f64().unwrap();        
        let pclose = pclo[ii].as_f64().unwrap();
         
        if close > pclose {
            obv = pobv + volum;
        }else if close < pclose {
            obv = pobv - volum;
        }else if close == pclose{
            obv = pobv;
        }

        pobv = obv;
        fobv.push(obv);

        i = i - 1;
        ii = ii - 1;
        
    }
    Ok(fobv)
}


pub fn analize(body: &String) -> Result<Vec<f64>, Box<dyn Error>>{
    let mut result= Vec::new();
    
    let vpt = volume_price_trend(&body).unwrap();
    let len = vpt.len();
    result.push(vpt[len - 1]);
    let obv = on_balance_volume(&body).unwrap();
    let len2 = obv.len();
    result.push(obv[len2 - 1]);
    
    
    Ok(result)
}

pub fn write_to_file(body: &String) -> Result<(), Box<dyn Error>>{
    let v: Value = serde_json::from_str(&body)?;
    

    let open = return_stock_values(&v,1);
    let high = return_stock_values(&v,2);
    let low = return_stock_values(&v,3);
    let close = return_stock_values(&v,4);
    let volume = return_stock_values(&v,5);
    let vpts = volume_price_trend(&body).unwrap();
    let obvs = on_balance_volume(&body).unwrap();

    let write_file = File::create("tmp/output").unwrap();
    let mut writer = BufWriter::new(&write_file);

    write!(&mut writer, "Open,High,Low,Close,Volume,VPT,OBV,Trend\n"); 
    let mut counter = open.len() - 3;
    let mut counter2 = vpts.len() - 1;
    let mut counter3 = obvs.len() - 1;
    while counter > 0{
        write!(&mut writer, "{},{},{},{},{},{},{},{}\n", open[counter],high[counter],low[counter],close[counter],volume[counter],vpts[counter2],obvs[counter2],if open[counter].as_f64().unwrap() > open[counter - 1].as_f64().unwrap() {0} else {1});
        counter = counter - 1;
        counter2 = counter2 - 1;
    }

    Ok(())
}
pub fn write_to_file_predict(body: &String, vpt: &String, obv: &String) -> Result<(), Box<dyn Error>>{
    let v: Value = serde_json::from_str(&body)?;
    let open = return_stock_values(&v,1);
    let high = return_stock_values(&v,2);
    let low = return_stock_values(&v,3);
    let close = return_stock_values(&v,4);
    let volume = return_stock_values(&v,5);
    
    let write_file = File::create("predict/output2").unwrap();
    let mut writer = BufWriter::new(&write_file);
    write!(&mut writer, "Open,High,Low,Close,Volume,VPT,OBV,Trend\n"); 

    let mut counter = open.len() - 1;
    while counter > 0{
        write!(&mut writer, "{},{},{},{},{},{},{},{}", open[counter],high[counter],low[counter],close[counter],volume[counter],vpt,obv,if open[counter].as_f64().unwrap() > open[counter - 1].as_f64().unwrap() {0} else {1});
        counter = counter - 1;
    }

    Ok(())
}