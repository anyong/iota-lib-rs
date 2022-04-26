// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! [Stronghold] integration for iota.rs.
//!
//! Stronghold can be used as a multi-purpose secret service providing:
//!
//! - Smart-card-like secret vault
//! - Generic key-value, encrypted database
//!
//! [`StrongholdAdapter`] respectively implements [`DatabaseProvider`] and [`SecretManager`] for the above purposes
//! using Stronghold. Type aliases [`StrongholdDatabaseProvider`] and [`StrongholdSecretManager`] are also provided if
//! one wants to have a more consistent naming when using any of the feature sets.
//!
//! Use [`builder()`] to construct a [`StrongholdAdapter`] with customized parameters; see documentation of methods of
//! [`StrongholdAdapterBuilder`] for details. Alternatively, invoking [`new()`] (or using [`Default::default()`])
//! creates a [`StrongholdAdapter`] with default parameters. The default [`StrongholdAdapter`]:
//!
//! - is not initialized with a password
//! - is without a password clearing timeout
//! - is not associated with a snapshot file on the disk (i.e. working purely in memory)
//!
//! These default settings limit what [`StrongholdAdapter`] can do:
//!
//! - Without a password, all cryptographic operations (including database operations, as they encrypt / decrypt data)
//!   would fail.
//! - Without a password clearing timeout, the derived key would be stored in the memory for as long as possible, and
//!   could be used as an attack vector.
//! - Without a snapshot path configured, all operations would be _transient_ (i.e. all data would be lost when
//!   [`StrongholdAdapter`] is dropped).
//!
//! These configurations can also be set later using [`set_password()`], [`set_snapshot_path()`], etc.
//!
//! Stronghold is memory-based, so it's not required to use a snapshot file on the disk. Without a snapshot path set
//! (via [`StrongholdAdapterBuilder::snapshot_path()`] or [`StrongholdAdapter::set_snapshot_path()`]),
//! [`StrongholdAdapter`] will run purely in memory. If a snapshot path is set, then [`StrongholdAdapter`] would lazily
//! load the file on _the first call_ that performs some actions on Stronghold. Subsequent actions are still performed
//! in memory. If the snapshot file doesn't exist, these function calls will all fail. To load or store the Stronghold
//! state from or to a Stronghold snapshot on disk, remember to call [`read_stronghold_snapshot()`] or
//! [`write_stronghold_snapshot()`]; the latter can be used to create a snapshot file after creating a
//! [`StrongholdAdapter`] with a non-existent snapshot path.
//!
//! [Stronghold]: iota_stronghold
//! [`DatabaseProvider`]: crate::db::DatabaseProvider
//! [`SecretManager`]: crate::secret::SecretManager
//! [`StrongholdDatabaseProvider`]: crate::db::StrongholdDatabaseProvider
//! [`StrongholdSecretmanager`]: crate::signing::StrongholdSecretmanager
//! [`builder()`]: self::StrongholdAdapter::builder()
//! [`new()`]: self::StrongholdAdapter::new()
//! [`set_password()`]: self::StrongholdAdapter::set_password()
//! [`set_snapshot_path()`]: self::StrongholdAdapter::set_snapshot_path()
//! [`read_stronghold_snapshot()`]: self::StrongholdAdapter::read_stronghold_snapshot()
//! [`write_stronghold_snapshot()`]: self::StrongholdAdapter::write_stronghold_snapshot()

mod common;
mod db;
mod encryption;
mod secret;

use std::{path::PathBuf, sync::Arc, time::Duration};

use derive_builder::Builder;
use iota_stronghold::{ResultMessage, Stronghold};
use log::debug;
use riker::actors::ActorSystem;
use tokio::{sync::Mutex, task::JoinHandle};
use zeroize::{Zeroize, Zeroizing};

use self::common::{PRIVATE_DATA_CLIENT_PATH, STRONGHOLD_FILENAME};
use crate::{Error, Result};

/// A wrapper on [Stronghold].
///
/// See the [module-level documentation](self) for more details.
#[derive(Builder)]
#[builder(pattern = "owned", build_fn(skip))]
pub struct StrongholdAdapter {
    /// A stronghold instance.
    stronghold: Stronghold,

    /// A key to open the Stronghold vault.
    ///
    /// Note that in [`StrongholdAdapterBuilder`] there isn't a `key()` setter, because we don't want a user to
    /// directly set this field. Instead, [`password()`] is provided to hash a user-input password string and
    /// derive a key from it.
    ///
    /// [`password()`]: self::StrongholdAdapterBuilder::password()
    #[builder(setter(custom))]
    key: Arc<Mutex<Option<Zeroizing<Vec<u8>>>>>,

    /// An interval of time, after which `key` will be cleared from the memory.
    ///
    /// This is an extra security measure to further prevent attacks. If a timeout is set, then upon a `key` is set, a
    /// timer will be spawned in the background to clear ([zeroize]) the key after `timeout`.
    ///
    /// If a [`StrongholdAdapter`] is destroyed (dropped), then the timer will stop too.
    #[builder(setter(strip_option))]
    timeout: Option<Duration>,

    /// A handle to the timeout task.
    ///
    /// Note that this field doesn't actually have a custom setter; `setter(custom)` is only for skipping the setter
    /// generation.
    #[builder(setter(custom))]
    timeout_task: Arc<Mutex<Option<JoinHandle<()>>>>,

    /// The path to a Stronghold snapshot file.
    #[builder(setter(strip_option))]
    pub snapshot_path: Option<PathBuf>,

    /// Whether the snapshot has been loaded from the disk to the memory.
    #[builder(setter(skip))]
    snapshot_loaded: bool,
}

impl Default for StrongholdAdapter {
    fn default() -> Self {
        // XXX: we unwrap here.
        let system = ActorSystem::new().map_err(|err| err.to_string()).unwrap();
        let client_path = PRIVATE_DATA_CLIENT_PATH.to_vec();
        let options = Vec::new();

        Self {
            stronghold: Stronghold::init_stronghold_system(system, client_path, options),
            key: Arc::new(Mutex::new(None)),
            timeout: None,
            timeout_task: Arc::new(Mutex::new(None)),
            snapshot_path: None,
            snapshot_loaded: false,
        }
    }
}

/// Extra / custom builder method implementations.
impl StrongholdAdapterBuilder {
    /// Use an user-input password string to derive a key to use Stronghold.
    pub fn password(mut self, password: &str) -> Self {
        // Note that derive_builder always adds another layer of Option<T>.
        self.key = Some(Arc::new(Mutex::new(Some(self::common::derive_key_from_password(
            password,
        )))));

        self
    }

    /// Build [`StrongholdAdapter`] from the configuration.
    ///
    /// If both `key` (via [`password()`]) and `timeout` (via [`timeout()`]) are set, then an asynchronous task would be
    /// spawned in Tokio to purge ([zeroize]) `key` after `timeout`. There is a small delay (usually a few milliseconds)
    /// from the return of this function to this task actually being spawned and set in the returned
    /// [`StrongholdAdapter`].
    ///
    /// **This function must be called inside a Tokio runtime context (usually in an `async fn` invoked by a Tokio
    /// runtime, either directly or indirectly)**, as it uses [tokio::spawn()], which requires a Tokio context.
    /// Otherwise, the function would panic. If this is not desired, one needs to avoid calling [`password()`] and
    /// [`timeout()`] during the building process.
    ///
    /// [`password()`]: Self::password()
    /// [`timeout()`]: Self::timeout()
    pub fn build(mut self) -> StrongholdAdapter {
        // If both `key` and `timeout` are set, then we spawn the task and keep its join handle.
        if let (Some(key), Some(Some(timeout))) = (&self.key, self.timeout) {
            let timeout_task = Arc::new(Mutex::new(None));

            // The key clearing task, with the data it owns.
            let key = key.clone();
            let task_self = timeout_task.clone();

            // To keep this function synchronous (`fn`), we spawn a task that spawns the key clearing task here. It'll
            // however panic when this function is not in a Tokio runtime context (usually in an `async fn`), albeit it
            // itself is a `fn`. There is also a small delay from the return of this function to the task actually being
            // spawned and set in the `struct`.
            tokio::spawn(async move {
                *task_self.lock().await = Some(tokio::spawn(task_key_clear(task_self.clone(), key, timeout)));
            });

            // Keep the handle in the builder; the code below checks this.
            self.timeout_task = Some(timeout_task);
        }

        // Create the adapter per configuration and return it.
        //
        // False positive: we don't throw `None`s back!
        #[allow(clippy::question_mark)]
        StrongholdAdapter {
            stronghold: if let Some(stronghold) = self.stronghold {
                stronghold
            } else {
                // XXX: we unwrap here.
                let system = ActorSystem::new().map_err(|err| err.to_string()).unwrap();
                let client_path = PRIVATE_DATA_CLIENT_PATH.to_vec();
                let options = Vec::new();

                Stronghold::init_stronghold_system(system, client_path, options)
            },
            key: if let Some(key) = self.key {
                key
            } else {
                Arc::new(Mutex::new(None))
            },
            timeout: if let Some(timeout) = self.timeout {
                timeout
            } else {
                None
            },
            timeout_task: if let Some(timeout_task) = self.timeout_task {
                timeout_task
            } else {
                Arc::new(Mutex::new(None))
            },
            snapshot_path: if let Some(snapshot_path) = self.snapshot_path {
                snapshot_path
            } else {
                None
            },
            snapshot_loaded: false,
        }
    }
}

impl StrongholdAdapter {
    /// Create a [StrongholdAdapter] with default parameters.
    pub fn new() -> StrongholdAdapter {
        StrongholdAdapter::default()
    }

    /// Create a builder to construct a [StrongholdAdapter].
    pub fn builder() -> StrongholdAdapterBuilder {
        StrongholdAdapterBuilder::default()
    }

    /// Use an user-input password string to derive a key to use Stronghold.
    ///
    /// This function will also spawn an asynchronous task in Tokio to automatically purge the derived key from
    /// `password` after `timeout` (if set).
    pub async fn set_password(&mut self, password: &str) {
        *self.key.lock().await = Some(self::common::derive_key_from_password(password));

        // If a timeout is set, spawn a task to clear the key after the timeout.
        if let Some(timeout) = self.timeout {
            // If there has been a spawned task, stop it and re-spawn one.
            if let Some(timeout_task) = self.timeout_task.lock().await.take() {
                timeout_task.abort();
            }

            // The key clearing task, with the data it owns.
            let key = self.key.clone();
            let task_self = self.timeout_task.clone();

            *self.timeout_task.lock().await = Some(tokio::spawn(task_key_clear(task_self, key, timeout)));
        }
    }

    /// Immediately clear ([zeroize]) the stored key.
    ///
    /// If a key clearing thread has been spawned, then it'll be stopped too.
    pub async fn clear_key(&mut self) {
        // Stop a spawned task and setting it to None first.
        if let Some(timeout_task) = self.timeout_task.lock().await.take() {
            timeout_task.abort();
        }

        // Purge the key, setting it to None then.
        if let Some(mut key) = self.key.lock().await.take() {
            key.zeroize();
        }
    }

    /// Load Stronghold from a snapshot at `snapshot_path`, if it hasn't been loaded yet.
    pub async fn read_stronghold_snapshot(&mut self) -> Result<()> {
        if self.snapshot_loaded {
            return Ok(());
        }

        // The key and the snapshot path need to be supplied first.
        let locked_key = self.key.lock().await;
        let key = if let Some(key) = &*locked_key {
            key
        } else {
            return Err(Error::StrongholdKeyCleared);
        };

        let snapshot_path = if let Some(path) = &self.snapshot_path {
            path
        } else {
            return Err(Error::StrongholdSnapshotPathMissing);
        };

        match self
            .stronghold
            .read_snapshot(
                PRIVATE_DATA_CLIENT_PATH.to_vec(),
                None,
                &**key,
                Some(STRONGHOLD_FILENAME.to_string()),
                Some(snapshot_path.clone()),
            )
            .await
        {
            ResultMessage::Ok(_) => Ok(()),
            ResultMessage::Error(err) => Err(crate::Error::StrongholdProcedureError(err)),
        }?;

        self.snapshot_loaded = true;

        Ok(())
    }

    /// Persist Stronghold to a snapshot at `snapshot_path`.
    ///
    /// It doesn't unload the snapshot.
    pub async fn write_stronghold_snapshot(&mut self) -> Result<()> {
        // The key and the snapshot path need to be supplied first.
        let locked_key = self.key.lock().await;
        let key = if let Some(key) = &*locked_key {
            key
        } else {
            return Err(Error::StrongholdKeyCleared);
        };

        let snapshot_path = if let Some(path) = &self.snapshot_path {
            path
        } else {
            return Err(Error::StrongholdSnapshotPathMissing);
        };

        match self
            .stronghold
            .write_all_to_snapshot(
                &**key,
                Some(STRONGHOLD_FILENAME.to_string()),
                Some(snapshot_path.clone()),
            )
            .await
        {
            ResultMessage::Ok(_) => Ok(()),
            ResultMessage::Error(err) => Err(crate::Error::StrongholdProcedureError(err)),
        }
    }
}

/// The asynchronous key clearing task purging `key` after `timeout` spent in Tokio.
async fn task_key_clear(
    task_self: Arc<Mutex<Option<JoinHandle<()>>>>,
    key: Arc<Mutex<Option<Zeroizing<Vec<u8>>>>>,
    timeout: Duration,
) {
    tokio::time::sleep(timeout).await;

    debug!("StrongholdAdapter is purging the key");
    if let Some(mut key) = key.lock().await.take() {
        key.zeroize();
    }

    // Take self, but do nothing (we're exiting anyways).
    task_self.lock().await.take();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_clear_key() {
        let mut client = StrongholdAdapter::builder()
            .password("drowssap")
            .timeout(Duration::from_millis(100))
            .build();

        // There is a small delay between `build()` and the key clearing task being actually spawned and kept.
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Setting a password would spawn a task to automatically clear the key.
        assert!(matches!(*client.key.lock().await, Some(_)));
        assert!(matches!(client.timeout, Some(_)));
        assert!(matches!(*client.timeout_task.lock().await, Some(_)));

        // After the timeout, the key should be purged.
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(matches!(*client.key.lock().await, None));
        assert!(matches!(client.timeout, Some(_)));
        assert!(matches!(*client.timeout_task.lock().await, None));

        // Set the key again, but this time we manually purge the key.
        client.set_password("password").await;
        assert!(matches!(*client.key.lock().await, Some(_)));
        assert!(matches!(client.timeout, Some(_)));
        assert!(matches!(*client.timeout_task.lock().await, Some(_)));

        client.clear_key().await;
        assert!(matches!(*client.key.lock().await, None));
        assert!(matches!(client.timeout, Some(_)));
        assert!(matches!(*client.timeout_task.lock().await, None));
    }
}
