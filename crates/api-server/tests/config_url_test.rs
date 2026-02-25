#[cfg(test)]
mod tests {
    use quantumtv_api::config_url::fetch_subscription;

    #[tokio::test]
    async fn test_fetch_subscription_real_url() {
        // 替换成你想测试的真实 URL
        let url = " ";

        let result = fetch_subscription(url, false).await;

        match result {
            Ok(config) => {
                // 使用 serde_json 打印整个结构体
                match serde_json::to_string_pretty(&config) {
                    Ok(json_str) => println!("订阅配置:\n{}", json_str),
                    Err(e) => println!("序列化失败: {}", e),
                }
            }
            Err(e) => panic!("读取失败: {}", e),
        }
    }
}
