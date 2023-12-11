use web_sys;
use eve_sde::*;

pub async fn get_static_data() -> Result<Vec<System>, String> {
    let baseurl = web_sys::window().ok_or_else(|| format!("Cannot get base URL"))?.origin();

    let result = reqwest::get(format!("{baseurl}/sde.json")).await
        .map_err(|_| format!("Failed to send request for sde.json"))?
        .error_for_status().map_err(|_| format!("Bad status code getting sde.json"))?
        .text().await
        .map_err(|_| format!("Failed to get bytes for sde.json"))?;

    serde_json::from_str::<Vec<System>>(&result)
        .map_err(|e| format!("Failed to parse sde.json JSON: {:?}", e))
}