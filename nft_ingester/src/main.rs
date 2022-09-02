mod backfiller;
mod error;
mod program_transformers;
mod tasks;
mod utils;
mod metrics;

use chrono::Utc;
use plerkle_messenger::{ACCOUNT_STREAM, Messenger, MessengerConfig, RedisMessenger, TRANSACTION_STREAM};

use {
    crate::{
        backfiller::backfiller,
        program_transformers::*,
        utils::{order_instructions, parse_logs},
    },
    futures_util::TryFutureExt,
    messenger::{Messenger, RedisMessenger, ACCOUNT_STREAM, TRANSACTION_STREAM},
    plerkle_serialization::account_info_generated::account_info::root_as_account_info,
    plerkle_serialization::transaction_info_generated::transaction_info::root_as_transaction_info,
    solana_sdk::pubkey::Pubkey,
    sqlx::{self, postgres::PgPoolOptions, Pool, Postgres},
    tokio::sync::mpsc::UnboundedSender,
    serde::Deserialize,
    figment::{Figment, providers::Env},
    cadence_macros::{
        set_global_default,
        statsd_count,
        statsd_time,
    },
    cadence::{BufferedUdpMetricSink, QueuingMetricSink, StatsdClient},
    std::net::UdpSocket,
};
use blockbuster::{
    instruction::InstructionBundle
};
use messenger::MessengerConfig;
use crate::error::IngesterError;
use crate::program_handler::ProgramHandlerManager;
use crate::tasks::{BgTask, TaskManager};


// Types and constants used for Figment configuration items.
pub type DatabaseConfig = figment::value::Dict;

pub const DATABASE_URL_KEY: &str = "url";
pub const DATABASE_LISTENER_CHANNEL_KEY: &str = "listener_channel";

pub type RpcConfig = figment::value::Dict;

pub const RPC_URL_KEY: &str = "url";
pub const RPC_COMMITMENT_KEY: &str = "commitment";

// Struct used for Figment configuration items.
#[derive(Deserialize, PartialEq, Debug)]
pub struct IngesterConfig {
    pub database_config: DatabaseConfig,
    pub messenger_config: MessengerConfig,
    pub rpc_config: RpcConfig,
    pub metrics_port: u16,
    pub metrics_host: String,
}

fn setup_metrics(config: &IngesterConfig) {
    let uri = config.metrics_host.clone();
    let port = config.metrics_port.clone();
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.set_nonblocking(true).unwrap();
    let host = (uri, port);
    let udp_sink = BufferedUdpMetricSink::from(host, socket).unwrap();
    let queuing_sink = QueuingMetricSink::from(udp_sink);
    let client = StatsdClient::from_sink("das_ingester", queuing_sink);
    set_global_default(client);
}

#[tokio::main]
async fn main() {
    // Read config.
    println!("Starting DASgester");
    let config: IngesterConfig = Figment::new()
        .join(Env::prefixed("INGESTER_"))
        .extract()
        .map_err(|config_error| IngesterError::ConfigurationError {
            msg: format!("{}", config_error),
        })
        .unwrap();
    // Get database config.
    let url = config
        .database_config
        .get(&*DATABASE_URL_KEY)
        .and_then(|u| u.clone().into_string())
        .ok_or(IngesterError::ConfigurationError {
            msg: format!("Database connection string missing: {}", DATABASE_URL_KEY),
        })
        .unwrap();
    // Setup Postgres.
    let mut tasks = vec![];
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .unwrap();
    let background_task_manager =
        TaskManager::new("background-tasks".to_string(), pool.clone()).unwrap();
    // Service streams as separate concurrent processes.
    println!("Setting up tasks");
    setup_metrics(&config);
    tasks.push(service_transaction_stream::<RedisMessenger>(pool.clone(), background_task_manager.get_sender(), config.messenger_config.clone()).await);
    statsd_count!("ingester.startup", 1);

    tasks.push(backfiller::<RedisMessenger>(pool.clone(), config.clone()).await);
    // Wait for ctrl-c.
    match tokio::signal::ctrl_c().await {
        Ok(()) => {}
        Err(err) => {
            println!("Unable to listen for shutdown signal: {}", err);
            // We also shut down in case of error.
        }
    }

    // Kill all tasks.
    for task in tasks {
        task.abort();
    }
}

async fn service_transaction_stream<T: Messenger>(
    pool: Pool<Postgres>,
    tasks: UnboundedSender<Box<dyn BgTask>>,
    messenger_config: MessengerConfig,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let manager = ProgramTransformer::new(pool, tasks);
        let mut messenger = T::new(messenger_config).await.unwrap();
        println!("Setting up transaction listener");
        loop {
            // This call to messenger.recv() blocks with no timeout until
            // a message is received on the stream.
            if let Ok(data) = messenger.recv(TRANSACTION_STREAM).await {
                handle_transaction(&manager, data).await;
            }
        }
    })
}

async fn service_account_stream<T: Messenger>(
    pool: Pool<Postgres>,
    tasks: UnboundedSender<Box<dyn BgTask>>,
    messenger_config: MessengerConfig,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let manager = ProgramTransformer::new(pool, tasks);
        let mut messenger = T::new(messenger_config).await.unwrap();
        println!("Setting up account listener");
        loop {
            // This call to messenger.recv() blocks with no timeout until
            // a message is received on the stream.
            if let Ok(data) = messenger.recv(ACCOUNT_STREAM).await {
                handle_account(&manager, data).await
            }
        }
    })
}

async fn handle_account(manager: &ProgramTransformer, data: Vec<(i64, &[u8])>) {
    for (_message_id, data) in data {
        // Get root of account info flatbuffers object.
        let account_update = match root_as_account_info(data) {
            Err(err) => {
                println!("Flatbuffers AccountInfo deserialization error: {err}");
                continue;
            }
            Ok(account_update) => account_update,
        };
        statsd_count!("ingester.account_update_seen", 1);
        manager.handle_account_update(account_update).await?

    }
}

async fn handle_transaction(manager: &ProgramTransformer, data: Vec<(i64, &[u8])>) {
    for (message_id, data) in data {
        //TODO -> Dedupe the stream, the stream could have duplicates as a way of ensuring fault tolerance if one validator node goes down.
        //  Possible solution is dedup on the plerkle side but this doesnt follow our principle of getting messages out of the validator asd fast as possible.
        //  Consider a Messenger Implementation detail the deduping of whats in this stream so that
        //  1. only 1 ingest instance picks it up, two the stream coming out of the ingester can be considered deduped

        // Get root of transaction info flatbuffers object.
        let transaction = match root_as_transaction_info(data) {
            Err(err) => {
                println!("Flatbuffers TransactionInfo deserialization error: {err}");
                continue;
            }
            Ok(transaction) => transaction,
        };
        if let Some(si) = transaction.slot_index() {
            let slt_idx = format!("{}-{}", transaction.slot(), si);
            statsd_count!("ingester.transaction_event_seen", 1, "slot-idx" => &slt_idx);
        }
        let seen_at = Utc::now();
        statsd_time!("ingester.bus_ingest_time", (seen_at.timestamp_millis() - transaction.seen_at()) as u64);
        manager.handle_transaction(transaction).await?;
    }
}
// Associates logs with the given program ID
