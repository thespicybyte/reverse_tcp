use std::fs;
use std::path::Path;
use tracing::info;

mod builder;
mod logging;

use mythic_rabbitmq::{
    C2ConfigCheckMessage, C2ConfigCheckMessageResponse, C2GetIOCMessage, C2GetIOCMessageResponse,
    C2GetRedirectorRuleMessage, C2GetRedirectorRuleMessageResponse, C2HostFileMessage,
    C2HostFileMessageResponse, C2OPSECMessage, C2OPSECMessageResponse, C2ProfileDefinition,
    C2SampleMessageMessage, C2SampleMessageResponse, MythicC2Container,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::init_logger("reverse_tcp_profile")?;

    info!("Starting reverse_tcp C2 profile...");

    load_c2_config()?;

    let icon_bytes = std::fs::read("profile_icon.svg").ok().map(|b| b);

    MythicC2Container::builder()
        .profile(C2ProfileDefinition {
            name: "reverse_tcp".to_string(),
            author: "@spicybyte".to_string(),
            description: "Listens on a TCP socket, receives messages from agents,
                and forwards them to the Mythic server via Push C2"
                .to_string(),
            is_p2p: false,
            is_server_routed: true,
            semver: "0.0.1".to_string(),
            agent_icon: icon_bytes,
            dark_mode_agent_icon: None,
        })
        .server_binary_path(get_binary_path())
        .server_folder_path(".")
        .parameters(builder::c2_parameters())
        .on_config_check(config_check_handler)
        .on_opsec_check(opsec_check_handler)
        .on_get_ioc(get_ioc_handler)
        .on_sample_message(sample_message_handler)
        .on_redirector_rules(get_redirector_rules_handler)
        .on_host_file(host_file_handler)
        .build()
        .start_and_run_forever()
        .await;

    info!("reverse_tcp C2 profile shutting down");
    Ok(())
}

fn get_binary_path() -> String {
    let binary_name = if cfg!(target_os = "windows") {
        "reverse_tcp_server.exe"
    } else {
        "reverse_tcp_server"
    };
    match std::env::var("PROJECT_BIN_DIR") {
        Ok(bin_dir) => {
            let path = format!("{}/{}", bin_dir, binary_name);
            info!("Using binary path from PROJECT_BIN_DIR: {}", path);
            path
        }
        Err(_) => {
            let path = format!("./{}", binary_name);
            info!("Using default binary path: {}", path);
            path
        }
    }
}

fn load_c2_config() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = "config.json";
    // let config_dir = Path::new("./reverse_tcp/c2_code");

    // if !config_dir.exists() {
    //     info!("Creating config directory: {:?}", config_dir);
    //     fs::create_dir_all(config_dir)?;
    // }

    if Path::new(config_path).exists() {
        info!("Config file found at: {}", config_path);
    } else {
        info!("Creating default config file at: {}", config_path);
        let default_config = r#"{
  "instances": [
    {
      "port": 4444,
      "debug": false,
      "bind_ip": "0.0.0.0"
    }
  ]
}"#;
        fs::write(config_path, default_config)?;
        info!("Default config file created");
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// RPC handlers - called by mythic-rabbitmq when Mythic sends a request
// ---------------------------------------------------------------------------

fn config_check_handler(message: C2ConfigCheckMessage) -> C2ConfigCheckMessageResponse {
    info!(
        "Config check received, parameters: {:?}",
        message.c2_parameters.parameters
    );
    C2ConfigCheckMessageResponse {
        success: true,
        error: String::new(),
        message: format!(
            "Called config check\n{:?}",
            message.c2_parameters.parameters
        ),
        restart_internal_server: false,
    }
}

fn opsec_check_handler(message: C2OPSECMessage) -> C2OPSECMessageResponse {
    info!(
        "OPSEC check received, parameters: {:?}",
        message.c2_parameters.parameters
    );
    C2OPSECMessageResponse {
        success: true,
        error: String::new(),
        message: format!(
            "Called opsec check:\n{:?}",
            message.c2_parameters.parameters
        ),
        restart_internal_server: false,
    }
}

fn get_ioc_handler(_message: C2GetIOCMessage) -> C2GetIOCMessageResponse {
    info!("Get IOC called");
    C2GetIOCMessageResponse {
        success: true,
        error: String::new(),
        iocs: vec![],
        restart_internal_server: false,
    }
}

fn sample_message_handler(_message: C2SampleMessageMessage) -> C2SampleMessageResponse {
    info!("Sample message called");
    C2SampleMessageResponse {
        success: false,
        error: String::new(),
        message: "Not supported".to_string(),
        restart_internal_server: false,
    }
}

fn get_redirector_rules_handler(
    _message: C2GetRedirectorRuleMessage,
) -> C2GetRedirectorRuleMessageResponse {
    info!("Get redirector rules called");
    C2GetRedirectorRuleMessageResponse {
        success: false,
        error: String::new(),
        message: "Not supported".to_string(),
        restart_internal_server: false,
    }
}

fn host_file_handler(_message: C2HostFileMessage) -> C2HostFileMessageResponse {
    info!("Host file called");
    C2HostFileMessageResponse {
        success: false,
        error: "Not supported".to_string(),
        restart_internal_server: false,
    }
}
