use super::config::Configs;
use anyhow::{bail, Result};
use graphql_client::{GraphQLQuery, Response};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};

pub struct GQLClient;

impl GQLClient {
    pub fn new_authorized(configs: &Configs) -> Result<Client> {
        let mut headers = HeaderMap::new();
        if let Some(token) = configs.root_config.fly_pat.clone() {
            headers.insert(
                "Authorization",
                HeaderValue::from_str(format!("Bearer {}", token).as_str())?,
            );
        } else {
            bail!("No fly personal access token configured");
        }
        let client = Client::builder().default_headers(headers).build()?;
        Ok(client)
    }
}

pub async fn post_graphql<Q: GraphQLQuery, U: reqwest::IntoUrl>(
    client: &reqwest::Client,
    url: U,
    variables: Q::Variables,
) -> Result<Response<Q::ResponseData>, reqwest::Error> {
    let body = Q::build_query(variables);
    let reqwest_response = client.post(url).json(&body).send().await?;

    Ok(reqwest_response.json().await?)
}
