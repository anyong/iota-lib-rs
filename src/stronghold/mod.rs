// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! [Stronghold] integration for iota.rs.
//!
//! Stronghold can be used as a multi-purpose secret service providing:
//!
//! - Smart-card-like secret vault
//! - Generic key-value, encrypted database
//!
//! [`StrongholdAdapter`] respectively implements [`DatabaseProvider`] and [`SecretManage`] for the above purposes
//! using Stronghold. Type aliases `StrongholdDatabaseProvider` and `StrongholdSecretManager` are also provided if one
//! wants to have a more consistent naming when using any of the feature sets.
//!
//! Use [`builder()`] to construct a [`StrongholdAdapter`] with customized parameters; see documentation of methods of
//! [`StrongholdAdapterBuilder`] for details. All fields are optional, but:
//!
//! - Without a password, all cryptographic operations (including database operations, as they encrypt / decrypt data)
//!   would fail.
//! - Without a password clearing timeout, the derived key would be stored in the memory for as long as possible, and
//!   could be used as an attack vector.
//! - Without a snapshot path configured, all operations would be _transient_ (i.e. all data would be lost when
//!   [`StrongholdAdapter`] is dropped, or the cached key has been cleared).
//!
//! They can also be set later on [`StrongholdAdapter`] using [`set_password()`], [`set_timeout()`], etc.
//!
//! With [`set_timeout()`], an automatic task can be spawned in the background to purge the key from memory using
//! [zeroize] after the `timeout` duration. It's used to reduce the attack vector. When the key is cleared from the
//! memory, Stronghold will be unloaded from the memory too. If no `snapshot_path` has been set at this point, then
//! secrets stored in Stronghold will be dropped and lost.
//!
//! Nevertheless, Stronghold is memory-based, so it's not required to use a snapshot file on the disk. Without a
//! snapshot path set, [`StrongholdAdapter`] will run purely in memory. If a snapshot path is set, then
//! [`StrongholdAdapter`] would lazily load the file on _the first call_ that performs some actions on Stronghold.
//! Subsequent actions are still performed in memory. If the snapshot file doesn't exist, these function calls will all
//! fail. To proactively load or store the Stronghold state from or to a Stronghold snapshot on disk, use
//! [`read_stronghold_snapshot()`] or [`write_stronghold_snapshot()`]. The latter can be used to create a snapshot file
//! after creating a [`StrongholdAdapter`] with a non-existent snapshot path.
//!
//! [Stronghold]: iota_stronghold
//! [`DatabaseProvider`]: crate::db::DatabaseProvider
//! [`SecretManage`]: crate::secret::SecretManage
//! [`builder()`]: self::StrongholdAdapter::builder()
//! [`set_password()`]: self::StrongholdAdapter::set_password()
//! [`set_timeout()`]: self::StrongholdAdapter::set_timeout()
//! [`read_stronghold_snapshot()`]: self::StrongholdAdapter::read_stronghold_snapshot()
//! [`write_stronghold_snapshot()`]: self::StrongholdAdapter::write_stronghold_snapshot()

mod common;
mod db;
mod encryption;
mod secret;

use std::{path::PathBuf, sync::Arc, time::Duration};

use derive_builder::Builder;
use iota_stronghold::{ResultMessage, Stronghold};
use log::{debug, error, warn};
use riker::actors::ActorSystem;
use tokio::{sync::Mutex, task::JoinHandle};
use zeroize::{Zeroize, Zeroizing};

use self::common::{PRIVATE_DATA_CLIENT_PATH, STRONGHOLD_FILENAME};
use crate::{db::DatabaseProvider, Error, Result};

/// A wrapper on [Stronghold].
///
/// See the [module-level documentation](self) for more details.
#[derive(Builder)]
#[builder(pattern = "owned", build_fn(skip))]
pub struct StrongholdAdapter {
    /// A stronghold instance.
    stronghold: Arc<Mutex<Stronghold>>,

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

    /// Try to build [`StrongholdAdapter`] from the configuration.
    ///
    /// The only possible error comes from [riker::system::ActorSystem::new()] for communicating with Stronghold.
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
    pub fn try_build(mut self) -> Result<StrongholdAdapter> {
        // In any case, Stronghold - as a necessary component - needs to be present at this point.
        let stronghold = if let Some(stronghold) = self.stronghold {
            stronghold
        } else {
            let system = ActorSystem::new()?;
            let client_path = PRIVATE_DATA_CLIENT_PATH.to_vec();
            let options = Vec::new();

            Arc::new(Mutex::new(Stronghold::init_stronghold_system(
                system,
                client_path,
                options,
            )))
        };

        // If both `key` and `timeout` are set, then we spawn the task and keep its join handle.
        if let (Some(key), Some(Some(timeout))) = (&self.key, self.timeout) {
            let timeout_task = Arc::new(Mutex::new(None));

            // The key clearing task, with the data it owns.
            let task_self = timeout_task.clone();
            let stronghold_cloned = stronghold.clone();
            let key = key.clone();

            // To keep this function synchronous (`fn`), we spawn a task that spawns the key clearing task here. It'll
            // however panic when this function is not in a Tokio runtime context (usually in an `async fn`), albeit it
            // itself is a `fn`. There is also a small delay from the return of this function to the task actually being
            // spawned and set in the `struct`.
            tokio::spawn(async move {
                *task_self.lock().await = Some(tokio::spawn(task_key_clear(
                    task_self.clone(), // LHS moves task_self
                    stronghold_cloned,
                    key,
                    timeout,
                )));
            });

            // Keep the task handle in the builder; the code below checks this.
            self.timeout_task = Some(timeout_task);
        }

        // Create the adapter as per configuration and return it.
        Ok(StrongholdAdapter {
            stronghold,
            key: self.key.unwrap_or_else(|| Arc::new(Mutex::new(None))),
            timeout: self.timeout.unwrap_or(None),
            timeout_task: self.timeout_task.unwrap_or_else(|| Arc::new(Mutex::new(None))),
            snapshot_path: self.snapshot_path.unwrap_or(None),
            snapshot_loaded: false,
        })
    }
}

impl StrongholdAdapter {
    /// Create a builder to construct a [StrongholdAdapter].
    pub fn builder() -> StrongholdAdapterBuilder {
        StrongholdAdapterBuilder::default()
    }

    /// Test if the key hasn't been cleared.
    pub async fn is_key_available(&self) -> bool {
        self.key.lock().await.is_some()
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
            let task_self = self.timeout_task.clone();
            let stronghold = self.stronghold.clone();
            let key = self.key.clone();

            *self.timeout_task.lock().await = Some(tokio::spawn(task_key_clear(task_self, stronghold, key, timeout)));
        }
    }

    /// Change the password of the currently loaded Stronghold.
    ///
    /// If a snapshot path has been set, then it'll be rewritten with the newly set password.
    ///
    /// The secrets (e.g. mnemonic) stored in the Stronghold vault will be preserved, but the data saved via the
    /// [`DatabaseProvier`] interface won't - they'll stay encrypted with the old password. To re-encrypt these
    /// data, provide a list of keys in `keys_to_re_encrypt`, as we have no way to list and iterate over every
    /// key-value in the Stronghold store - we'll attempt on the ones provided instead. Set it to `None` to skip
    /// re-encryption.
    pub async fn change_password(&mut self, new_password: &str, keys_to_re_encrypt: Option<&[&[u8]]>) -> Result<()> {
        // Stop the key clearing task to prevent the key from being abrubtly cleared (largely).
        if let Some(timeout_task) = self.timeout_task.lock().await.take() {
            timeout_task.abort();
        }

        // In case something goes wrong we can recover from the snapshot.
        if self.snapshot_path.is_some() {
            self.write_stronghold_snapshot().await?;
        }

        // If there are keys to re-encrypt, we iterate over the requested keys and attempt to re-encrypt the
        // corresponding values.
        //
        // Note that [`DatabaseProvider`] methods will do encryption / decryption automatically, so we collect values
        // to the memory first (decrypted with the old key), then change `self.key`, then store them back (encrypted
        // with the new key).
        let mut values = Vec::new();

        if let Some(keys_to_re_encrypt) = keys_to_re_encrypt {
            for key in keys_to_re_encrypt {
                let value = match self.get(key).await {
                    Err(err) => {
                        error!("an error occurred during the re-encryption of Stronghold Store: {err}");

                        // Recover: restart the key clearing task
                        if let Some(timeout) = self.timeout {
                            // The key clearing task, with the data it owns.
                            let task_self = self.timeout_task.clone();
                            let stronghold = self.stronghold.clone();
                            let key = self.key.clone();

                            *self.timeout_task.lock().await =
                                Some(tokio::spawn(task_key_clear(task_self, stronghold, key, timeout)));
                        }

                        return Err(err);
                    }
                    Ok(None) => continue,
                    Ok(Some(value)) => Zeroizing::new(value),
                };

                values.push((key, value));
            }
        }

        // Now we put the new key in, enabling encryption with the new key. Also, take the old key out to prevent
        // disasters.
        let old_key = {
            let mut lock = self.key.lock().await;
            let old_key = lock.take();
            *lock = Some(self::common::derive_key_from_password(new_password));

            old_key
        };

        for (key, value) in values.into_iter() {
            if let Err(err) = self.insert(key, &*value).await {
                error!("an error occurred during the re-encryption of Stronghold store: {err}");

                // Recover: put the old key back
                *self.key.lock().await = old_key;

                // Recover: forcefully reload Stronghold
                self.snapshot_loaded = false;
                self.read_stronghold_snapshot().await?;

                // Recover: restart key clearing task
                if let Some(timeout) = self.timeout {
                    // The key clearing task, with the data it owns.
                    let task_self = self.timeout_task.clone();
                    let stronghold = self.stronghold.clone();
                    let key = self.key.clone();

                    *self.timeout_task.lock().await =
                        Some(tokio::spawn(task_key_clear(task_self, stronghold, key, timeout)));
                }

                return Err(err);
            }
        }

        // Rewrite the snapshot to finish the password changing process.
        if self.snapshot_path.is_some() {
            self.write_stronghold_snapshot().await?;
        }

        // Restart the key clearing task.
        if let Some(timeout) = self.timeout {
            // The key clearing task, with the data it owns.
            let task_self = self.timeout_task.clone();
            let stronghold = self.stronghold.clone();
            let key = self.key.clone();

            *self.timeout_task.lock().await = Some(tokio::spawn(task_key_clear(task_self, stronghold, key, timeout)));
        }

        Ok(())
    }

    /// Immediately clear ([zeroize]) the stored key.
    ///
    /// If a key clearing thread has been spawned, then it'll be stopped too.
    pub async fn clear_key(&mut self) {
        // Stop a spawned task and setting it to None first.
        if let Some(timeout_task) = self.timeout_task.lock().await.take() {
            timeout_task.abort();
        }

        // Unload Stronghold first, but we can't do much about the errors.
        if let Err(err) = self.unload_stronghold_snapshot().await {
            warn!("failed to unload Stronghold while clearing the key: {err}");
        }

        // Purge the key, setting it to None then.
        if let Some(mut key) = self.key.lock().await.take() {
            key.zeroize();
        }
    }

    /// Get timeout for the key clearing task.
    pub fn get_timeout(&self) -> Option<Duration> {
        self.timeout
    }

    /// Set timeout for the key clearing task.
    ///
    /// If there has been a key clearing task running, then it will be terminated before a new one is spawned. If
    /// `new_timeout` is `None`, or the key has been purged, then no new task will be spawned (the current running task
    /// will be terminated).
    ///
    /// The key won't be cleared.
    pub async fn set_timeout(&mut self, new_timeout: Option<Duration>) {
        // In any case we terminate the current task (if there is) first.
        if let Some(timeout_task) = self.timeout_task.lock().await.take() {
            timeout_task.abort();
        }

        // Keep the new timeout.
        self.timeout = new_timeout;

        // If a new timeout is set and the key is still in the memory, spawn a new task; otherwise we do nothing.
        if let (Some(_), Some(timeout)) = (self.key.lock().await.as_ref(), self.timeout) {
            // The key clearing task, with the data it owns.
            let task_self = self.timeout_task.clone();
            let stronghold = self.stronghold.clone();
            let key = self.key.clone();

            *self.timeout_task.lock().await = Some(tokio::spawn(task_key_clear(task_self, stronghold, key, timeout)));
        }
    }

    /// Restart the key clearing task.
    ///
    /// This is equivalent to calling `set_timeout()` with the currently set `timeout`.
    pub async fn restart_key_clearing_task(&mut self) {
        self.set_timeout(self.get_timeout()).await;
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
            .lock()
            .await
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
    /// It doesn't unload the snapshot; see also [`unload_stronghold_snapshot()`].
    ///
    /// [`unload_stronghold_snapshot()`]: Self::unload_stronghold_snapshot()
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

        // Check if directory in path exists, if not create it
        if let Some(parent) = snapshot_path.parent() {
            if !parent.is_dir() {
                std::fs::create_dir_all(parent)?;
            }
        }

        match self
            .stronghold
            .lock()
            .await
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

    /// Unload Stronghold from memory.
    ///
    /// This first writes Stronghold snapshot to disk, then kills Stronghold. All secrets will be purged from the
    /// memory, so if secrets aren't written to disk (for example, no snapshot path has been provided, i.e. running
    /// Stronghold purely in memory) then secrets stored in Stronghold will be lost.
    ///
    /// To further prevent Stronghold methods to be invoked without valid key, this method will be invoked every time
    /// the cached key is cleared from the memory. In other words, if a `timeout` is set and a `snapshot_path` is not
    /// set for a [`StrongholdAdapter`], then after `timeout` Stronghold will be purged. See the [module-level
    /// documentation](self) for more details.
    pub async fn unload_stronghold_snapshot(&mut self) -> Result<()> {
        // Flush Stronghold.
        self.write_stronghold_snapshot().await?;

        // Kill Stronghold.
        match self
            .stronghold
            .lock()
            .await
            .kill_stronghold(PRIVATE_DATA_CLIENT_PATH.to_vec(), false)
            .await
        {
            ResultMessage::Ok(_) => Ok(()),
            ResultMessage::Error(err) => Err(crate::Error::StrongholdProcedureError(err)),
        }?;

        self.snapshot_loaded = false;

        Ok(())
    }
}

/// The asynchronous key clearing task purging `key` after `timeout` spent in Tokio.
async fn task_key_clear(
    task_self: Arc<Mutex<Option<JoinHandle<()>>>>,
    stronghold: Arc<Mutex<Stronghold>>,
    key: Arc<Mutex<Option<Zeroizing<Vec<u8>>>>>,
    timeout: Duration,
) {
    tokio::time::sleep(timeout).await;

    debug!("StrongholdAdapter is purging the key");
    if let Some(mut key) = key.lock().await.take() {
        key.zeroize();
    }

    debug!("StrongholdAdapter is killing Stronghold");
    stronghold
        .lock()
        .await
        .kill_stronghold(PRIVATE_DATA_CLIENT_PATH.to_vec(), false)
        .await;

    // Take self, but do nothing (we're exiting anyways).
    task_self.lock().await.take();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_clear_key() {
        let timeout = Duration::from_millis(100);

        let mut adapter = StrongholdAdapter::builder()
            .password("drowssap")
            .timeout(timeout)
            .try_build()
            .unwrap();

        // There is a small delay between `build()` and the key clearing task being actually spawned and kept.
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Setting a password would spawn a task to automatically clear the key.
        assert!(matches!(*adapter.key.lock().await, Some(_)));
        assert_eq!(adapter.get_timeout(), Some(timeout));
        assert!(matches!(*adapter.timeout_task.lock().await, Some(_)));

        // After the timeout, the key should be purged.
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(matches!(*adapter.key.lock().await, None));
        assert_eq!(adapter.get_timeout(), Some(timeout));
        assert!(matches!(*adapter.timeout_task.lock().await, None));

        // Set the key again, but this time we manually purge the key.
        let timeout = None;
        adapter.set_timeout(timeout).await;

        adapter.set_password("password").await;
        assert!(matches!(*adapter.key.lock().await, Some(_)));
        assert_eq!(adapter.get_timeout(), timeout);
        assert!(matches!(*adapter.timeout_task.lock().await, None));

        adapter.clear_key().await;
        assert!(matches!(*adapter.key.lock().await, None));
        assert_eq!(adapter.get_timeout(), timeout);
        assert!(matches!(*adapter.timeout_task.lock().await, None));

        // Even if we attempt to restart the task, it won't.
        adapter.restart_key_clearing_task().await;
        assert!(matches!(*adapter.key.lock().await, None));
        assert_eq!(adapter.get_timeout(), timeout);
        assert!(matches!(*adapter.timeout_task.lock().await, None));
    }
}
