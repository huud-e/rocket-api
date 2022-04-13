/*
        SI EL PRECIO SUBE Y EL VOLUMEN TAMBIEN, TREND UP
        SI EL PRECIO BAJA Y EL VOLUMEN TAMBINE, TREND DOWN
        SI EL PRECIO SUBE Y EL VOLUMEN SE MANTIENE/BAJA, EL TREND DOWN NO AGUANTARA
        SI EL PRECIO BAJA Y EL VOLUMEN SE MANTIENE/SUBE, EL TREND UP NO AGUANTARA
*/
// use std::collections::HashMap;
use dotenv::dotenv;
use std::env;
use serde_json::{Result, Value};
/*
use std::fs::File;                                                                                                                                                                   
use std::io::Write;                                                                                                                                                                  
 */
use std::{error::Error, time::Duration};
use tokio::time::sleep;
use redis::{
    from_redis_value,
    streams::{StreamRangeReply, StreamReadOptions, StreamReadReply},
    AsyncCommands, Client,
};

fn return_stock_values(v: Value, opt: u8) -> Vec<serde_json::value::Value> {
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

fn volume_price_trend(stock: &String) -> Result<f32> {
    // VPT = Volume x (Today’s Closing Price – Previous Closing Price) / Previous Closing Price
    let v: Value = serde_json::from_str(&stock)?;
    
    
    let vol = return_stock_values(v.to_owned(), 5);   
    let clo = return_stock_values(v.to_owned(), 4);
    let pclo = return_stock_values(v, 4);

    
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

    println!("VPT of stock is {:?}", vpt);
    
    Ok(vpt)
}

async fn redis() -> Result<()> {
    Ok(())
}

fn on_balance_volume(stock: &String) -> Result<f32>{
    let v: Value = serde_json::from_str(&stock)?;
    
    let vol = return_stock_values(v.to_owned(), 5);   
    let clo = return_stock_values(v.to_owned(), 4);
    let pclo = return_stock_values(v, 4);

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
    println!("OBV of stock is {:?}", obv);
    Ok(obv)

}


pub fn analize(stock:String) -> Vec<f32>{
    let mut result= Vec::new();
    
    let vpt = volume_price_trend(&stock).unwrap();
    result.push(vpt);
    
    let obv = on_balance_volume(&stock).unwrap();
    result.push(obv);
    




    return result;
}