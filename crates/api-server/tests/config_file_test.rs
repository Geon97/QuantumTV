#[cfg(test)]
mod tests {
    use quantumtv_api::config_file::{
        filter_adult_source_configs, load_parses_from_file, load_source_configs_from_file,
        PARSES_FILE,
    };

    #[tokio::test]
    async fn test_load_parses_from_file() {
        println!("读取文件路径: {}", *PARSES_FILE);

        match load_parses_from_file().await {
            Ok(parses) => {
                println!("读取成功！{}", *PARSES_FILE);
                // println!("解析结果: 省略");

                assert!(!parses.config.source_config.is_empty());
            }
            Err(e) => {
                println!("读取失败: {}", e);
                panic!("测试失败");
            }
        }
    }
    #[tokio::test]
    async fn test_load_source_configs_from_file() {
        println!("读取文件路径: {}", *PARSES_FILE);

        match load_source_configs_from_file().await {
            Ok(source_configs) => {
                println!("source_configs 读取成功！");
                // println!("解析结果: {:#?}", source_configs);

                assert!(!source_configs.is_empty());
            }
            Err(e) => {
                println!("读取失败: {}", e);
                panic!("测试失败");
            }
        }
    }
    #[tokio::test]
    async fn test_filter_adult_source_configs() {
        println!("读取文件路径: {}", *PARSES_FILE);

        match filter_adult_source_configs().await {
            Ok(source_configs) => {
                println!("source_configs 读取成功！");
                println!("过滤结果: {:#?}", source_configs);

                assert!(!source_configs.is_empty());
            }
            Err(e) => {
                println!("读取失败: {}", e);
                panic!("测试失败");
            }
        }
    }
}
