// Copyright (c) 2019-2021 Decode Detroit
// Author: Patton Doyle
// Licence: GNU GPLv3
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

//! A module to monitor send and receive event updates from the rest of the
//! system.
//!
//! Event updates are received on the provided input line and echoed to the
//! rest of the system. Updates from the system are passed back to the event
//! handler system via the event_send line.

// Define private submodules
mod comedy_comm;
mod dmx_out;
mod media_out;
mod zmq_comm;

// Import crate definitions
use crate::definitions::*;

// Import other definitions
use self::comedy_comm::ComedyComm;
use self::dmx_out::DmxOut;
use self::media_out::{MediaOut};
use self::zmq_comm::{ZmqBind, ZmqConnect};

// Import standard library features
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::thread;

// Import the failure features
use failure::Error;

// Import program constants
use super::POLLING_RATE; // the polling rate for the system

// Define communication constants
enum ReadResult {
    // A variant for a successful event read
    Normal(ItemId, u32, u32),

    // A variant for errors when writing data
    WriteError(Error),

    // A variant for errors when reading data
    ReadError(Error),
}

// Implement key connection type features
impl ConnectionType {
    /// An internal method to create a Live Connection from this Connection
    /// Type. This method estahblishes the connection to the underlying system.
    /// If the connection fails, it will return the Error.
    ///
    async fn initialize(&self, internal_send: &InternalSend) -> Result<LiveConnection, Error> {
        // Switch between the different connection types
        match self {
            // Connect to a live version of the comedy serial port
            &ConnectionType::ComedySerial { ref path, ref baud } => {
                // Create the new comedy connection
                let connection = ComedyComm::new(path, baud.clone(), POLLING_RATE)?;
                Ok(LiveConnection::ComedySerial { connection })
            }

            // Connect to a live version of the zmq port
            &ConnectionType::ZmqPrimary {
                ref send_path,
                ref recv_path,
            } => {
                // Create the new zmq connection
                let connection = ZmqBind::new(send_path, recv_path)?;
                Ok(LiveConnection::ZmqPrimary { connection })
            }

            // Connect to a live version of the zmq port
            &ConnectionType::ZmqSecondary {
                ref send_path,
                ref recv_path,
            } => {
                // Create a new zmq to main connection
                let connection = ZmqConnect::new(send_path, recv_path)?;
                Ok(LiveConnection::ZmqSecondary { connection })
            }

            // Connect to a live version of the DMX serial port
            &ConnectionType::DmxSerial {
                ref path,
                ref all_stop_dmx,
                ref dmx_map,
            } => {
                // Create the new dmx connection
                let connection = DmxOut::new(path, all_stop_dmx.clone(), dmx_map.clone())?;
                Ok(LiveConnection::DmxSerial { connection })
            }

            // Connect to a live version of the Media output
            &ConnectionType::Media {
                ref all_stop_media,
                ref media_map,
                ref channel_map,
                ref window_map,
                ref apollo_params,
            } => {
                // Create the new media connection
                let connection = MediaOut::new(
                    internal_send.clone(),
                    all_stop_media.clone(),
                    media_map.clone(),
                    channel_map.clone(),
                    window_map.clone(),
                    apollo_params.clone(),
                ).await?;
                Ok(LiveConnection::Media { connection })
            }
        }
    }
}

/// An internal enum to hold the different types of a system connection.
/// Unlike the Connection Type, this structure holds a fully initialized
/// connection to the underlying system.
///
enum LiveConnection {
    /// A variant to connect with a ComedyComm serial port. This implementation
    /// assumes the serial connection uses the ComedyComm protocol.
    ComedySerial {
        connection: ComedyComm, // the comedy connection
    },

    /// A variant to create a ZeroMQ connection. This connection type allows
    /// messages to be the sent and received. Received messages are echoed back
    /// to the send line so that all recipients will see the message
    ZmqPrimary {
        connection: ZmqBind, // the zmq connection
    },

    /// A variant to connect to an existing ZeroMQ connection over ZMQ.
    /// This connection presumes that a fully-functioning Minerva instance is
    /// is operating at the other end of the connection.
    ZmqSecondary {
        connection: ZmqConnect, // the zmq connection
    },

    /// A variant to connect with a DMX serial port. This connection type only
    /// allows messages to be the sent.
    DmxSerial {
        connection: DmxOut, // the DMX serial connection
    },

    /// A variant to connect with system media. This connection type only allows
    /// messages to be sent.
    Media {
        connection: MediaOut, // the system media connection
    },
}

// Implement event connection for LiveConnection
impl EventConnection for LiveConnection {
    /// The read event method
    fn read_events(&mut self) -> Vec<ReadResult> {
        // Read from the interior connection
        match self {
            &mut LiveConnection::ComedySerial { ref mut connection } => connection.read_events(),
            &mut LiveConnection::ZmqPrimary { ref mut connection } => connection.read_events(),
            &mut LiveConnection::ZmqSecondary { ref mut connection } => connection.read_events(),
            &mut LiveConnection::DmxSerial { ref mut connection } => connection.read_events(),
            &mut LiveConnection::Media { ref mut connection } => connection.read_events(),
        }
    }

    /// The write event method (does not check duplicates)
    fn write_event(&mut self, id: ItemId, data1: u32, data2: u32) -> Result<(), Error> {
        // Write to the interior connection
        match self {
            &mut LiveConnection::ComedySerial { ref mut connection } => {
                connection.write_event(id, data1, data2)
            }
            &mut LiveConnection::ZmqPrimary { ref mut connection } => {
                connection.write_event(id, data1, data2)
            }
            &mut LiveConnection::ZmqSecondary { ref mut connection } => {
                connection.write_event(id, data1, data2)
            }
            &mut LiveConnection::DmxSerial { ref mut connection } => {
                connection.write_event(id, data1, data2)
            }
            &mut LiveConnection::Media { ref mut connection } => {
                connection.write_event(id, data1, data2)
            }
        }
    }

    /// The echo event method (checks for duplicates from recently read events)
    fn echo_event(&mut self, id: ItemId, data1: u32, data2: u32) -> Result<(), Error> {
        // Echo events to the interior connection
        match self {
            &mut LiveConnection::ComedySerial { ref mut connection } => {
                connection.echo_event(id, data1, data2)
            }
            &mut LiveConnection::ZmqPrimary { ref mut connection } => {
                connection.echo_event(id, data1, data2)
            }
            &mut LiveConnection::ZmqSecondary { ref mut connection } => {
                connection.echo_event(id, data1, data2)
            }
            &mut LiveConnection::DmxSerial { ref mut connection } => {
                connection.echo_event(id, data1, data2)
            }
            &mut LiveConnection::Media { ref mut connection } => {
                connection.echo_event(id, data1, data2)
            }
        }
    }
}

/// An private enum to send broadcast events to the system connection
///
enum ConnectionUpdate {
    /// A variant to indicate an event should be broadcast
    ///
    Broadcast(ItemId, Option<u32>),

    /// A variant to indicate that the connection process should stop
    Stop,
}

/// A structure to handle all the input and output with the rest of the system.
///
pub struct SystemConnection {
    internal_send: InternalSend, // sending structure for new events from the system
    connection_send: Option<mpsc::Sender<ConnectionUpdate>>, // receiving structure for new events from the program
                                                             //connection: Option<LiveConnection>, // an element that implements both read and write
    is_broken: bool, // flag to indicate if one or more connections failed to establish
}

// Implement key Logger struct features
impl SystemConnection {
    /// A function to create a new system connection instance.
    ///
    /// This function requires a general update line for passing events from the
    /// system back to the event handler.
    ///
    /// # Errors
    ///
    /// This function will raise an error if a connection type was provided and
    /// it was unable to connect to the underlying system.
    ///
    /// Like all SystemInterface functions and methods, this function will fail
    /// gracefully by warning the user and returning a default system connection.
    ///
    pub async fn new(
        internal_send: InternalSend,
        connections: Option<(ConnectionSet, Identifier)>,
    ) -> SystemConnection {
        // Create an empty system connection
        let mut system_connection = SystemConnection {
            internal_send,
            connection_send: None,
            is_broken: false,
        };

        // Try to update the system connection using the provided connection type(s)
        system_connection
            .update_system_connection(connections)
            .await;

        // Return the system connection
        system_connection
    }

    /// A method to update the system connection type. This method returns false
    /// if it was unable to connect to the underlying system and warns the user.
    ///
    pub async fn update_system_connection(
        &mut self,
        connections: Option<(ConnectionSet, Identifier)>,
    ) -> bool {
        // Close the existing connection, if it exists
        if let Some(ref conn_send) = self.connection_send {
            conn_send.send(ConnectionUpdate::Stop).unwrap_or(());
        }

        // Reset the connection
        self.connection_send = None;
        self.is_broken = false;

        // Check to see if there is a provided connection set
        if let Some((conn_set, identifier)) = connections {
            // Initialize the system connections
            let mut live_connections = Vec::new();
            for connection in conn_set {
                // Attempt to initialize each connection
                match connection.initialize(&self.internal_send).await {
                    Ok(conn) => live_connections.push(conn),

                    // If it fails, warn the user
                    Err(e) => {
                        log!(err self.internal_send => "System Connection Error: {}", e);
                        self.is_broken = true;
                    }
                };
            }

            // Spin a new thread with the connection(s)
            let (conn_send, conn_recv) = mpsc::channel();
            let internal_send = self.internal_send.clone();
            thread::spawn(move || {
                // Loop indefinitely
                SystemConnection::run_loop(live_connections, internal_send, conn_recv, identifier);
            });

            // Update the system connection
            self.connection_send = Some(conn_send);

            // Indicate whether the connections were successfully established
            return !self.is_broken;
        }

        // Otherwise, leave the system disconnected
        true
    }

    /// A method to send messages between the underlying system and the program.
    ///
    pub async fn broadcast(&mut self, new_event: ItemId, data: Option<u32>) {
        // Extract the connection, if it exists
        if let Some(ref mut conn) = self.connection_send {
            // Send the new event
            let result = conn.send(ConnectionUpdate::Broadcast(new_event, data));
            if let Err(e) = result {
                log!(err &self.internal_send => "Unable To Connect: {}", e);
            }

            // Warn if one or more connections were not established
            if self.is_broken {
                log!(err &self.internal_send => "Unable To Reach One Or More System Connections.");
            }
        }
    }

    /// An internal function to run a loop of the system connection
    ///
    fn run_loop(
        mut connections: Vec<LiveConnection>,
        internal_send: InternalSend,
        conn_recv: mpsc::Receiver<ConnectionUpdate>,
        identifier: Identifier,
    ) {
        // Run the loop until there is an error or instructed to quit
        loop {
            // Save the start time of the loop
            let loop_start = Instant::now();
            
            // Read all results from the system connections
            let mut results = Vec::new();
            for connection in connections.iter_mut() {
                results.append(&mut connection.read_events());
            }

            // Read all the results from the list
            for result in results.drain(..) {
                // Sort by the type of result
                match result {
                    // For a normal result
                    ReadResult::Normal(id, game_id, data2) => {
                        // Echo the event to every connection
                        for connection in connections.iter_mut() {
                            connection
                                .echo_event(id.clone(), game_id.clone(), data2.clone())
                                .unwrap_or(());
                        }

                        // If an identifier was specified
                        if let Some(identity) = identifier.id {
                            // Verify the game id is correct
                            if identity == game_id {
                                // Send the event to the program
                                internal_send.blocking_send(InternalUpdate::ProcessEvent {
                                    event: id,
                                    check_scene: true,
                                    broadcast: false,
                                }); // don't broadcast
                            // FIXME Handle incoming data

                            // Otherwise send a notification of an incorrect game number
                            } else {
                                // Format the warning string
                                let tmp = format!("Game Id Does Not Match. Event Ignored. ({})", id);

                                // Send the warning to the mpsc line
                                internal_send.blocking_send(InternalUpdate::Update(LogUpdate::Warning(tmp, None)));
                                
                                // FIXME Move to an async context to use log!
                                // log!(warn &internal_send => "Game Id Does Not Match. Event Ignored. ({})", id);
                            }

                        // Otherwise, send the event to the program
                        } else {
                            internal_send.blocking_send(InternalUpdate::ProcessEvent {
                                event: id,
                                check_scene: true,
                                broadcast: false,
                            }); // don't broadcast
                        }
                    }

                    // For a write error, notify the system
                    ReadResult::WriteError(error) => {                        
                        // Format the error string
                        let tmp = format!("Communication Write Error: {}", error);

                        // Send the warning to the mpsc line
                        internal_send.blocking_send(InternalUpdate::Update(LogUpdate::Error(tmp, None)));
                        
                        // FIXME Move to an async context to use log!
                        // log!(err &internal_send => "Communication Write Error: {}", error);
                    }

                    // For a read error, notify the system
                    ReadResult::ReadError(error) => {
                        // Format the error string
                        let tmp = format!("Communication Read Error: {}", error);

                        // Send the warning to the mpsc line
                        internal_send.blocking_send(InternalUpdate::Update(LogUpdate::Error(tmp, None)));
                        
                        // FIXME Move to an async context to use log!
                        // log!(err &internal_send => "Communication Read Error: {}", error);
                    }
                }
            }

            // Send any new events to the system
            let update = conn_recv.try_recv();
            match update {
                // Send the new event
                Ok(ConnectionUpdate::Broadcast(id, data)) => {
                    // Use the identifier or zero for the game id
                    let game_id = match identifier.id {
                        Some(id) => id,
                        None => 0,
                    };

                    // Translate the data to a placeholder, if necessary
                    let data2 = match data {
                        Some(number) => number,
                        None => 0,
                    };

                    // Try to send the new event to every connection
                    for connection in connections.iter_mut() {
                        // Catch any write errors
                        if let Err(error1) = connection.write_event(id, game_id, data2) {
                            // Throw the error
                            // Format the error string
                            let tmp = format!("Communication Error: {}", error1);

                            // Send the warning to the mpsc line
                            internal_send.blocking_send(InternalUpdate::Update(LogUpdate::Error(tmp, None)));
                            
                            // FIXME Move to an async context to use log!
                            // log!(err &internal_send => "Communication Error: {}", error1);

                            // Wait a little bit and try again
                            thread::sleep(Duration::from_millis(POLLING_RATE));
                            if let Err(error2) = connection.write_event(id, game_id, data2) {
                                // If failed twice in a row, notify the system
                                // Format the error string
                                let tmp = format!("Persistent Communication Error: {}", error2);

                                // Send the warning to the mpsc line
                                internal_send.blocking_send(InternalUpdate::Update(LogUpdate::Error(tmp, None)));

                                // FIXME Move to an async context to use log!
                                // log!(err &internal_send => "Persistent Communication Error: {}", error2);
                            }
                        }
                    }
                }

                // Quit when instructed or when there is an error
                Ok(ConnectionUpdate::Stop) => break,
                Err(mpsc::TryRecvError::Disconnected) => break,

                // Otherwise continue
                _ => (),
            }

            // Make sure that some time elapses in each loop
            if Duration::from_millis(POLLING_RATE) > loop_start.elapsed() {
                thread::sleep(Duration::from_millis(POLLING_RATE));
            }
        }
    }
}

/// Define the EventConnection Trait
///
/// This is a convience trait to standardize reading from and writing to the
/// event connection across all event connection types.
///
trait EventConnection {
    /// The read event method
    fn read_events(&mut self) -> Vec<ReadResult>;

    /// The write event method (does not check duplicates)
    fn write_event(&mut self, id: ItemId, data1: u32, data2: u32) -> Result<(), Error>;

    /// The echo event method (checks for duplicates from recently read events)
    fn echo_event(&mut self, id: ItemId, data1: u32, data2: u32) -> Result<(), Error>;
}
