use anyhow::Result;
use regex::Regex;
use std::net::TcpListener;
use url::Url;

pub async fn get_random_port() -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    Ok(addr.port())
}

pub fn choose_endpoint(model: &str, ollama_endpoint: &str, openai_endpoint: &str) -> String {
    let re = Regex::new(r"^gpt-.*").unwrap();
    if re.is_match(model) {
        openai_endpoint.to_string()
    } else {
        ollama_endpoint.to_string()
    }
}

pub fn convert_to_custom_endpoint(url: &str, endpoint: &str) -> Result<String> {
    let original_url = Url::parse(url)?;
    let endpoint_url = Url::parse(endpoint)?;

    let mut new_url = original_url;
    new_url
        .set_scheme(endpoint_url.scheme())
        .map_err(|_| anyhow::anyhow!("Failed to set scheme"))?;
    new_url
        .set_host(Some(
            endpoint_url
                .host_str()
                .ok_or_else(|| anyhow::anyhow!("No host in endpoint"))?,
        ))
        .map_err(|_| anyhow::anyhow!("Failed to set host"))?;
    if let Some(port) = endpoint_url.port() {
        new_url
            .set_port(Some(port))
            .map_err(|_| anyhow::anyhow!("Failed to set port"))?;
    }

    Ok(new_url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choose_endpoint() {
        let ollama = "http://localhost:11434";
        let openai = "https://api.openai.com";

        assert_eq!(choose_endpoint("gpt-3.5", ollama, openai), openai);
        assert_eq!(choose_endpoint("llama2", ollama, openai), ollama);
    }

    #[test]
    fn test_convert_to_custom_endpoint() {
        let result = convert_to_custom_endpoint(
            "https://api.openai.com/v1/chat/completions",
            "http://localhost:11434",
        )
        .unwrap();

        assert_eq!(result, "http://localhost:11434/v1/chat/completions");
    }
}
