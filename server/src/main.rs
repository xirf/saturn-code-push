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
    if std::env::args().nth(1).as_deref() == Some("--import-sqlite") {
        let path = std::env::args()
            .nth(2)
            .expect("Usage: saturn-code-push --import-sqlite /path/catalog.sqlite3");
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
        tokio::task::spawn_blocking(move || {
            saturn_code_push::import_sqlite(&url, std::path::Path::new(&path))
        })
        .await
        .expect("Import task")
        .expect("Import SQLite catalog");
        println!(
            "SQLite catalog imported. Keep the original objects directory and sign in with a new account."
        );
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("--reset-password") {
        use std::io::Read;
        let username = std::env::args()
            .nth(2)
            .expect("Usage: saturn-code-push --reset-password <username> (password on stdin)");
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
        let mut password = String::new();
        std::io::stdin()
            .read_to_string(&mut password)
            .expect("Read password from stdin");
        tokio::task::spawn_blocking(move || {
            saturn_code_push::reset_password(
                &url,
                &username,
                password.trim_end_matches(['\r', '\n']),
            )
        })
        .await
        .expect("Reset task")
        .expect("Reset account password");
        println!("Account password reset. Sessions and API keys were revoked.");
        return;
    }
    let config = Config::from_env().expect("Valid server configuration");
    let bind_address = config.bind_address;
    let app = tokio::task::spawn_blocking(move || application(config))
        .await
        .expect("Startup task")
        .expect("Initialize application");
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
