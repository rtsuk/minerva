// Copyright (c) 2020-2021 Decode Detroit
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

//! A module to create the web interface to interface to connect the web UI
//! and endpoints to the program.

// Import crate definitions
use crate::definitions::*;

// Import standard library features
use std::str::FromStr;
use std::num::ParseIntError;

// Import tokio and warp modules
use tokio::sync::oneshot;
use warp::{http, Filter};

/*/// A structure to pass server-side updates to the user interface
/// 
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebUpdate {// FIXME change to an enum
    display_setting: Option<DisplaySetting>, // variable for to changes the display settings
    //launch_window: Option<WindowType>, // launch a special window
    notice: Option<String>, // a notice to post briefly
    // FIXME notifications: Option<Vec<Notification>>, // formatted system notifications
    // FIXME upcoming_events: Option<Vec<UpcomingEvent>>, // a list of upcoming events for the timeline
    /*
    UpdateConfig {
        scenes: Vec<ItemPair>,
        full_status: FullStatus,
    },
    UpdateWindow {
        current_scene: ItemPair,
        statuses: Vec<ItemPair>,
        window: EventWindow,
        key_map: KeyMap,
    },
    UpdateStatus {
        status_id: ItemPair, // the group to update
        new_state: ItemPair, // the new state of the group
    },
    */
}

/// A helper type definition for the web_sender
type WebSend = mpsc::Sender<(ItemId, oneshot::Sender<WebReply>)>;*/

/// Helper data types to formalize request structure
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
struct CueEvent {
    id: u32,
}
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
struct GetItem {
    id: u32,
}
/*#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
struct PlaceStone {
    id: u32,
}
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
struct EntryId {
    id: u32,
}*/

// Implement FromStr for helper data types
impl FromStr for CueEvent {
    // Interpret errors as ParseIntError
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Parse as a u32 and return the result
        let id = s.parse::<u32>()?;
        Ok(CueEvent { id })
    }
}
impl FromStr for GetItem {
    // Interpret errors as ParseIntError
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Parse as a u32 and return the result
        let id = s.parse::<u32>()?;
        Ok(GetItem { id })
    }
}

// Implement from for the helper data types
impl From<CueEvent> for UserRequest {
    fn from(cue_event: CueEvent) -> Self {
        UserRequest::CueEvent {
            event_delay: EventDelay::new(None, ItemId::new_unchecked(cue_event.id)), // FIXME allow optional delay
        }
    }
}
/*impl From<PlaceStone> for Request {
    fn from(place_stone: PlaceStone) -> Self {
        Request::Place {
            id: place_stone.id,
        }
    }
}
impl From<EntryId> for Request {
    fn from(entry_id: EntryId) -> Self {
        Request::Entry {
            id: entry_id.id,
        }
    }
}*/

/// A structure to contain the web interface and handle all updates to the
/// to the interface.
///
pub struct WebInterface {
    index_access: IndexAccess, // access point for the item index
    web_send: WebSend, // send line to the system interface
}

// Implement key Web Interface functionality
impl WebInterface {
    /// A function to create a new web interface. The send channel should
    /// connect directly to the system interface.
    ///
    pub fn new(index_access: IndexAccess, web_send: WebSend) -> Self {
        // Return the new web interface and runtime handle
        WebInterface {
            index_access,
            web_send,
        }
    }

    /// A method to listen for connections from the internet
    ///
    pub async fn run(&mut self) {
        // Create the index filter
        let readme = warp::get()
            .and(warp::path::end())
            .and(warp::fs::file("./index.html"));

        // Create the trigger event filter
        let cue_event = warp::post()
            .and(warp::path("cueEvent"))
            .and(WebInterface::with_sender(self.web_send.clone()))
            .and(warp::path::param::<CueEvent>())
            .and(warp::path::end())
            .and_then(WebInterface::handle_request);

        // Create the item information filter
        let get_item = warp::get()
            .and(warp::path("getItem"))
            .and(WebInterface::with_index(self.index_access.clone()))
            .and(warp::path::param::<GetItem>())
            .and(warp::path::end())
            .and_then(WebInterface::handle_get_item);

        // Combine the filters
        let routes = readme.or(cue_event).or(get_item);
        
        // Handle incoming requests
        warp::serve(routes)
            .run(([127, 0, 0, 1], 64637))
            .await;
    }

    /// A function to handle incoming requests
    async fn handle_request<R>(web_send: WebSend, request: R) -> Result<impl warp::Reply, warp::Rejection>
        where R: Into<UserRequest>
    {       
        // Send the message and wait for the reply
        let (reply_to, rx) = oneshot::channel();
        web_send.send(reply_to, request.into()).await;
        
        // Wait for the reply
        if let Ok(reply) = rx.await {
            // If the reply is a success
            if reply.is_success() {
                return Ok(warp::reply::with_status(
                    warp::reply::json(&reply),
                    http::StatusCode::OK,
                ));
            
            // Otherwise, note the error
            } else {
                return Ok(warp::reply::with_status(
                    warp::reply::json(&reply),
                    http::StatusCode::BAD_REQUEST,
                ));
            }
        
        // Otherwise, note the error
        } else {
            return Ok(warp::reply::with_status(
                warp::reply::json(&WebReply::failure("Unable to process request.")),
                http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    }

    /// A function to handle get item requests (processed by the index)
    async fn handle_get_item(index_access: IndexAccess, get_item: GetItem) -> Result<impl warp::Reply, warp::Rejection> {       
        // Get the item pair from the index
        let item_pair = index_access.get_pair(&ItemId::new_unchecked(get_item.id)).await;
        
        // Return the item pair (even if it is the default)
        return Ok(warp::reply::with_status(
            warp::reply::json(&WebReply::Item { is_valid: true, item_pair }),
            http::StatusCode::OK,
        ));
    }

    /// A function to extract the json body
    pub fn json_body() -> impl Filter<Extract = (ItemId,), Error = warp::Rejection> + Clone {
        // When accepting a body, we want a JSON body (reject huge payloads)
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

    // A function to add the web send to the filter
    pub fn with_sender(web_send: WebSend) -> impl Filter<Extract = (WebSend,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || web_send.clone())
    }

    // A function to add the index access  to the filter
    pub fn with_index(index_access: IndexAccess) -> impl Filter<Extract = (IndexAccess,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || index_access.clone())
    }

    /// A function to add a specific request type to the filter
    pub fn with_request(request: UserRequest) -> impl Filter<Extract = (UserRequest,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || request.clone())
    }
}
