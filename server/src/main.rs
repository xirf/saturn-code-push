use saturn_code_push::{Config, application, hash_password};

#[tokio::main]
async fn main() {
    if std::env::args().nth(1).as_deref() == Some("--hash-password") {
        use std::io::Read;
        let mut password = String::new();
        std::io::stdin()
            .read_to_string(&mut password)
            .expect("Read password from stdin");
        let password = password.trim_end_matches(['\r', '\n']);
        assert!(
            (12..=1024).contains(&password.len()),
            "Password must contain 12 to 1024 bytes"
        );
        println!("{}", hash_password(password).expect("Hash password"));
        return;
    }
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let config = Config::from_env().expect("Valid server configuration");
    let bind_address = config.bind_address;
    let app = application(config).expect("Initialize application");
    let listener = tokio::net::TcpListener::bind(bind_address)
        .await
        .expect("Bind server socket");
    tracing::info!(%bind_address, "Saturn Code Push listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await
        .expect("Serve application");
}

async fn shutdown() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut terminate = signal(SignalKind::terminate()).expect("Install SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = terminate.recv() => {},
        }
    }
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .expect("Install shutdown handler");
    }
}
