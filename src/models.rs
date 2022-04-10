#[derive(diesel::Queryable)]
pub struct Stock{
    id: u32,
    shortname: String,
    longname: String,
    volumepricetrend: u32,
    momentumoscillator: u32,
    simplemovingaverage: u32,
    exponentialmovingaverage: u32,
    trendline: u32,
}