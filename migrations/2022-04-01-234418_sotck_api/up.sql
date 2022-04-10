-- Your SQL goes here
CREATE TABLE IF NOT EXISTS stocks (
    id SERIAL PRIMARY KEY,
    shortName VARCHAR NOT NULL,
    longName VARCHAR NOT NULL,
    volumePriceTrend INTEGER NOT NULL,
    momentumOscillator INTEGER NOT NULL,
    simpleMovingAverage INTEGER NOT NULL,
    exponentialMovingAverage INTEGER NOT NULL,
    trendline INTEGER NOT NULL
);