use rearch_rea::cdp::CdpConfig;
use std::path::PathBuf;

#[test]
fn test_cdp_config_defaults() {
    let config = CdpConfig::default();
    assert_eq!(config.port, 9222);
    assert_eq!(config.host, "127.0.0.1");
    assert!(config.headless);
    assert_eq!(config.endpoint_url(), "http://127.0.0.1:9222/json/version");

    let flags = config.to_command_args(PathBuf::from("/tmp/chrome-profile"));
    assert!(flags.contains(&"--remote-debugging-port=9222".to_string()));
    assert!(flags.contains(&"--remote-debugging-address=127.0.0.1".to_string()));
    assert!(flags.contains(&"--headless=new".to_string()));
    assert!(flags.contains(&"--user-data-dir=/tmp/chrome-profile".to_string()));
}

#[test]
fn test_cdp_config_custom_port() {
    let mut config = CdpConfig::default();
    config.port = 9333;
    config.headless = false;
    assert_eq!(config.endpoint_url(), "http://127.0.0.1:9333/json/version");

    let flags = config.to_command_args(PathBuf::from("/tmp/chrome-profile-custom"));
    assert!(flags.contains(&"--remote-debugging-port=9333".to_string()));
    assert!(!flags.contains(&"--headless=new".to_string()));
}
