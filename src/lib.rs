use chrono::{DateTime, Utc};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Client to upload personal weather station observations to windy.com
#[derive(Clone)]
pub struct WindyStation {
    /// User's API key.
    api_key: String,

    /// Request client.
    client: Client,

    /// Base url.
    base_url: String,
}

impl WindyStation {
    /// Creates a new API instance with the default client.
    pub fn new(api_key: String) -> Self {
        Self::with_client(api_key, Client::new())
    }

    /// Creates a new API instance with the specified client and API key.
    pub fn with_client(api_key: String, client: Client) -> Self {
        WindyStation {
            api_key,
            client,
            base_url: "https://stations.windy.com/pws/update".to_string(),
        }
    }

    /// Register the specified stations.
    pub async fn register_stations(&self, stations: &[Station]) -> Result<(), Box<dyn Error>> {
        #[derive(Serialize)]
        struct RegisterStationsRequest<'a> {
            stations: &'a [Station],
        }

        let request = RegisterStationsRequest { stations };

        self.post_request_builder()
            .json(&request)
            .send()
            .await?
            .error_for_status()
            .map(|_response| ())
            .map_err(|e| e.into())
    }

    /// Records the specified observations.
    pub async fn record_observations(
        &self,
        observations: &[Observation],
    ) -> Result<(), Box<dyn Error>> {
        #[derive(Serialize)]
        struct RecordObservationsRequest<'a> {
            observations: &'a [Observation],
        }

        let request = RecordObservationsRequest { observations };

        self.post_request_builder()
            .json(&request)
            .send()
            .await?
            .error_for_status()
            .map(|_response| ())
            .map_err(|e| e.into())
    }

    fn post_request_builder(&self) -> RequestBuilder {
        self.client
            .post(&format!("{}/{}", self.base_url, self.api_key))
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Station {
    /// Identifies the station if multiple stations are registered with an account.
    #[serde(rename = "station")]
    pub id: u32,

    /// Data sharing policy
    pub visibility: StationVisibility,

    /// Name of the station
    pub name: String,

    /// North–south position on the Earth`s surface, in degrees.
    pub latitude: f32,

    /// East-west position on the Earth's surface, in degrees.
    pub longitude: f32,

    /// Elevation above sea level (reference geoid), in meters.
    pub elevation: u32,

    /// Temperature sensor height above the surface, in meters.
    #[serde(rename = "tempheight")]
    pub temp_height: u32,

    /// Wind sensor height above the surface, in meters.
    #[serde(rename = "windheight")]
    pub wind_height: u32,
}

/// Defines data sharing policy.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum StationVisibility {
    /// Data is shared publicly with windy and its partners.
    Open,

    /// Data is only shared with windy, but is available publicly.
    #[serde(rename = "Only Windy")]
    OnlyWindy,

    /// Data is private to the user's account.
    Private,
}

/// An observation recorded by the station.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Observation {
    /// Station identifier.
    #[serde(rename = "station")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub station_id: Option<u32>,

    /// Time of the measurement.
    /// When not present, current server time is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<DateTime<Utc>>,

    /// Air temperature, in celsius.
    #[serde(rename = "temp")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Wind speed, in m/s.
    #[serde(rename = "wind")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wind_speed: Option<f32>,

    /// Wind direction, in degrees.
    #[serde(rename = "winddir")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wind_direction: Option<u16>,

    /// Wind gust, in m/s.
    #[serde(rename = "gust")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wind_gust: Option<f32>,

    /// Relative humidity, in %.
    #[serde(rename = "rh")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relative_humidity: Option<f32>,

    /// Dew point, in celsius.
    #[serde(rename = "dewpoint")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dew_point: Option<f32>,

    /// Atmospheric pressure, in Pa.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pressure: Option<f32>,

    /// Precipitation over the past hour, in mm.
    #[serde(rename = "precip")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precipitation: Option<f32>,

    /// UV index.
    #[serde(rename = "uv")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uv_index: Option<u8>,
}

#[cfg(test)]
mod tests {
    use crate::{Observation, Station, StationVisibility, WindyStation};
    use chrono::{FixedOffset, TimeZone, Utc};
    use mockito::mock;
    use reqwest::Client;
    use std::error::Error;
    use std::fs::read_to_string;

    #[tokio::test]
    async fn register_stations() -> Result<(), Box<dyn Error>> {
        let _ = env_logger::try_init();

        let body = read_to_string("test/request/register_stations.json")?;

        let mock = mock("POST", "/test-api-key")
            .with_status(200)
            .with_header("content-type", "text/html; charset=utf-8")
            .with_body(read_to_string("test/response/default.txt")?)
            .match_body(body.as_str())
            .create();

        {
            get_api()
                .register_stations(&[Station {
                    id: 0,
                    visibility: StationVisibility::Open,
                    name: "test-station".to_string(),
                    latitude: 49.282730,
                    longitude: -123.120735,
                    elevation: 62,
                    temp_height: 1,
                    wind_height: 2,
                }])
                .await?;
        }

        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn simple_report() -> Result<(), Box<dyn Error>> {
        let _ = env_logger::try_init();

        let body = read_to_string("test/request/simple_report.json")?;

        let mock = mock("POST", "/test-api-key")
            .with_status(200)
            .with_header("content-type", "text/html; charset=utf-8")
            .with_body(read_to_string("test/response/default.txt")?)
            .match_body(body.as_str())
            .create();

        {
            get_api()
                .record_observations(&[Observation {
                    temperature: Some(-1.2_f32),
                    relative_humidity: Some(99_f32),
                    ..Default::default()
                }])
                .await?;
        }

        mock.assert();

        Ok(())
    }

    #[tokio::test]
    async fn complete_report() -> Result<(), Box<dyn Error>> {
        let _ = env_logger::try_init();

        let body = read_to_string("test/request/complete_report.json")?;

        let mock = mock("POST", "/test-api-key")
            .with_status(200)
            .with_header("content-type", "text/html; charset=utf-8")
            .with_body(read_to_string("test/response/default.txt")?)
            .match_body(body.as_str())
            .create();

        {
            get_api()
                .record_observations(&[Observation {
                    station_id: Some(1),
                    time: Some(
                        FixedOffset::west(4 * 3600)
                            .ymd(2014, 10, 23)
                            .and_hms_micro(20, 3, 41, 636000)
                            .with_timezone(&Utc),
                    ),
                    temperature: Some(-1.2),
                    wind_speed: Some(25.0),
                    wind_direction: Some(182),
                    wind_gust: Some(35.0),
                    relative_humidity: Some(96.0),
                    dew_point: Some(1.0),
                    pressure: Some(1021000.0),
                    precipitation: Some(2.4),
                    uv_index: Some(1),
                }])
                .await?;
        }

        mock.assert();

        Ok(())
    }

    fn get_api() -> WindyStation {
        WindyStation {
            api_key: "test-api-key".to_string(),
            client: Client::new(),
            base_url: mockito::server_url(),
        }
    }
}
