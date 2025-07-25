//! Privastead Config commands
//!
//! Copyright (C) 2025  Ardalan Amiri Sani
//!
//! This program is free software: you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation, either version 3 of the License, or
//! (at your option) any later version.
//!
//! This program is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//! GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License
//! along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::io;
use log::{info, error};
use serde::{Deserialize, Serialize};
use crate::mls_clients::{MlsClients, NUM_MLS_CLIENTS, MLS_CLIENT_TAGS};

/// opcodes
pub const OPCODE_HEARTBEAT_REQUEST: u8 = 0;
pub const OPCODE_HEARTBEAT_RESPONSE: u8 = 1;

pub enum HeartbeatResult {
    InvalidTimestamp,
    InvalidCiphertext,
    InvalidEpoch,
    HealthyHeartbeat(u64), //timestamp: u64
}

#[derive(Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub timestamp: u64,
    pub motion_epoch: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Heartbeat {
    pub timestamp: u64,
    pub epochs: Vec<u64>, //for motion and livestream MLS clients
    pub ciphertexts: Vec<Vec<u8>>, //for all MLS clients except for config
}

impl Heartbeat {
    pub fn generate(
        mls_clients: &mut MlsClients,
        timestamp: u64,
    ) -> io::Result<Self> {
        let mut ciphertexts: Vec<Vec<u8>> = vec![];
        let mut epochs: Vec<u64> = vec![];
        let timestamp_bytes: Vec<u8> = timestamp.to_le_bytes().to_vec();

        for i in 0..NUM_MLS_CLIENTS {
            if MLS_CLIENT_TAGS[i] != "config" {
                let ciphertext = mls_clients[i]
                    .encrypt(&timestamp_bytes)?;
                mls_clients[i].save_group_state();
                ciphertexts.push(ciphertext);
            }

            if MLS_CLIENT_TAGS[i] == "motion" || MLS_CLIENT_TAGS[i] == "livestream" {
                let epoch = mls_clients[i].get_epoch()?;
                epochs.push(epoch);
            }
        }
        
        Ok(Self {
            timestamp,
            epochs,
            ciphertexts,
        })
    }

    pub fn process(
        &self,
        mls_clients: &mut MlsClients,
        expected_timestamp: u64,
    ) -> io::Result<HeartbeatResult> {
        info!("Going to process heartbeat");
        if expected_timestamp != self.timestamp {
            error!("Unexpected timestamp");
            return Ok(HeartbeatResult::InvalidTimestamp);
        }

        let mut ciphertexts_i = 0;
        let mut epoch_i = 0;
        for i in 0..NUM_MLS_CLIENTS {
            if MLS_CLIENT_TAGS[i] != "config" {
                if MLS_CLIENT_TAGS[i] == "motion" || MLS_CLIENT_TAGS[i] == "livestream" {
                    let epoch = match mls_clients[i].get_epoch() {
                        Ok(e) => e,
                        Err(e) => {
                            // The mls client is most likely corrupted.
                            error!("Failed to get epoch of mls client: {:?}", e);
                            return Ok(HeartbeatResult::InvalidCiphertext);
                        },
                    };

                    if epoch != self.epochs[epoch_i] {
                        return Ok(HeartbeatResult::InvalidEpoch);
                    }

                    epoch_i += 1;
                }
                let plaintext = match mls_clients[i]
                    .decrypt(self.ciphertexts[ciphertexts_i].clone(), true) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Failed to decrypt ciphertext: {:?}", e);
                        return Ok(HeartbeatResult::InvalidCiphertext);
                    },
                };
                mls_clients[i].save_group_state();
                        
                info!("Checking plaintext for {}", MLS_CLIENT_TAGS[i]);
                let timestamp_bytes: [u8; 8] = match plaintext.try_into() {
                    Ok(b) => b,
                    Err(e) => {
                        error!("Failed to get timestamp bytes: {:?}", e);
                        return Ok(HeartbeatResult::InvalidCiphertext);
                    }
                };
                let timestamp = u64::from_le_bytes(timestamp_bytes);
                if timestamp != self.timestamp {
                    error!("Decrypted timestamp from the {} client is not correct.", MLS_CLIENT_TAGS[i]);
                    return Ok(HeartbeatResult::InvalidCiphertext);
                }
                ciphertexts_i += 1;
            }
        }
        info!("Heartbeat successfully processed.");

        Ok(HeartbeatResult::HealthyHeartbeat(self.timestamp))
    }
}