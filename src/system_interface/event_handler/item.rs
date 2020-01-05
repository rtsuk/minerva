// Copyright (c) 2017 Decode Detroit
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

//! This module implements a basic identifier system (ItemId, ItemDescription,
//! and ItemPair) to allow simple and future-proof identification of all events,
//! scenes, and other items in the program.

// Import standard library modules
use std::cmp::Ordering;
use std::fmt;
use std::hash;

/// Define the All Stop command (a.k.a. emergency stop)
const ALL_STOP: u32 = 0;

/// Define the Comm Error and Read Error ids (special ids reserved for system errors)
pub const COMM_ERROR: u32 = 1; // a special id for communication errors
pub const READ_ERROR: u32 = 0xFFFFFFFF; // a special id for read errors

/// This structure is a generic identifier to identify different aspects of the
/// configuration (e.g. events, scenes).
///
/// Inverted IDs (ItemId::new_inverted();)
/// can be used for all IDs that aren't events and leaves more id space
/// available for the the events by avoiding the lower range of IDs.
///
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct ItemId {
    id: u32, // private to avoid collisions between inverted and non-inverted IDs
}

// Implement key ItemId struct features
impl ItemId {
    /// A function to create an item id from u32, CAN version.
    ///
    /// When program is built with CAN checking ON, this function will cap valid
    /// IDs at 29 bits (i.e. 29^2 or over half a million IDs). If you need
    /// additional IDs (exceptionally rare) and are not using a CAN bus, you can
    /// disable this checking by rebuilding the program with the no_can_limit
    /// feature (see below).
    ///
    /// # Disabling CAN Checking
    ///
    /// If this documentation was provided with the program or was autogenerated
    ///
    /// ```
    /// cargo doc --open
    /// ```
    /// then your program was likely compiled with CAN checking ON. Rebuild your
    /// program without CAN checking with
    ///
    /// ```
    /// cargo build --features no_can_limit
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// let item = ItemId::new(0);
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return None if the address exceeds the 29-bit
    /// address limit or conflicts with the ALL_STOP id and Some(ItemId)
    /// otherwise.
    ///
    #[cfg(not(feature = "no_can_limit"))]
    pub fn new(id: u32) -> Option<ItemId> {
        // Verify id does not conflict with all stop
        if id == ALL_STOP {
            return None;
        }

        // Verify 29 bit limit for CAN bus
        if id >= 0x1FFFFFFF {
            return None;
        }

        // Return the new item id
        Some(ItemId { id })
    }

    /// A function to create an item id from u32, no CAN version.
    ///
    /// When program is built with CAN checking OFF, this function will not
    /// check IDs validity. If you are running a this system on a CAN bus,
    /// it is highly recommended that you turn CAN checking ON (see below).
    ///
    /// # Enabling CAN Checking
    ///
    /// If this documentation was provided with the program or was autogenerated
    ///
    /// ```
    /// cargo doc --open
    /// ```
    /// then your program was likely compiled with CAN checking OFF. Rebuild
    /// your program with CAN checking with
    ///
    /// ```
    /// cargo build
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// let item = ItemId::new(0xFFFFFFFF);
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return None if the address conflicts with the
    /// ALL_STOP id and Some(ItemId) otherwise.
    ///
    #[cfg(feature = "no_can_limit")]
    pub fn new(id: u32) -> Option<ItemId> {
        // Verify id does not conflict with all stop
        if id == ALL_STOP {
            return None;
        }

        // Return the new item id
        Some(ItemId { id })
    }

    /// A function to create an item id from u32, unchecked version.
    ///
    /// This function does not verify that the new ItemId complies with the CAN
    /// limit and does not check that the new ItemId does not collide with the
    /// all stop item id. This is useful when either (or both) of these cases
    /// are possible and desired.
    ///
    pub fn new_unchecked(id: u32) -> ItemId {
        ItemId { id }
    }

    /// A method to return the proper id of the item.
    ///
    /// This function returns the id number of the item. Internal representation
    /// is protected in case future versions make use of inverted ids.
    ///
    /// # Examples
    ///
    /// ```
    /// let id = ItemId::new(5);
    /// assert_eq!(5, id.id());
    /// ```
    ///
    pub fn id(&self) -> u32 {
        self.id
    }

    /// A method to return the proper id of the item as a string.
    ///
    /// This function returns the id number of the item. Internal representation
    /// is protected in case future versions make use of inverted ids.
    ///
    /// # Examples
    ///
    /// ```
    /// let id = ItemId::new(5);
    /// assert_eq!("5", &id.as_string());
    /// ```
    ///
    pub fn as_string(&self) -> String {
        self.id.to_string()
    }

    /// A function to return a new all stop item id. This is a reserved id for
    /// halting all active processes on the system and returning it to normal.
    ///
    pub fn all_stop() -> ItemId {
        ItemId { id: ALL_STOP }
    }
}

// Implement displaying that shows the ID
impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// This enum is a type to allow display information to fall into several types.
///
#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum DisplayType {
    /// A variant for items which are displayed in the top level control group.
    /// These items will be displayed in the story control area in order of
    /// ascending  id, or in order of ascending position if specified. The text
    /// color will match the rgb value, if specified. The text will use the
    /// highlight color for any special animations and when the highlight_state
    /// status is in the listed state (if specified). If spotlight is set,
    /// the text will flash the highlight color the specified number of times
    /// when changed to this state or when this button is present but has not
    /// yet been pressed. A value of zero implies indefinite flashing.
    ///
    DisplayControl {
        position: Option<u32>,
        color: Option<(u8, u8, u8)>,
        highlight: Option<(u8, u8, u8)>,
        highlight_state: Option<(ItemId, ItemId)>,
        spotlight: Option<u32>,
    },

    /// A variant to indicatie items which to be displayed with a specific group
    /// (and paired with the status of that group). Note that grouped events
    /// don't need to be displayed in the same group (or at all). However, items
    /// with the description DisplayWith are expected to have a corresponding
    /// group id in the configuration lookup. Events will be displayed in
    /// order of ascending id within their group, or in order of ascending
    /// position if specified. The text color will match the rgb value, if
    /// specified. The text will use the highlight color for any special
    /// animations and when the highlight_state status is in the listed state
    /// (if specified). If spotlight is set, the text will flash the highlight color
    /// the specified number of times when changed to this state or when this
    /// button is present but has not yet been pressed. A value of zero implies
    /// indefinite flashing.
    ///
    DisplayWith {
        group_id: ItemId,
        position: Option<u32>,
        color: Option<(u8, u8, u8)>,
        highlight: Option<(u8, u8, u8)>,
        highlight_state: Option<(ItemId, ItemId)>,
        spotlight: Option<u32>,
    },

    /// A variant for items which are displayed with a particular group (if
    /// specified) or with the control group, but only when the program is in
    /// debug mode. These items will be displayed in order of ascending id,
    /// or in order of ascending position if specified. The text color will
    /// match the rgb value, if specified. The text will use the highlight
    /// color for any special animations and when the highlight_state status
    /// is in the listed state (if specified). If spotlight is set,
    /// the text will flash the highlight color the specified number of times
    /// when changed to this state or when this button is present but has not
    /// yet been pressed. A value of zero implies indefinite flashing.
    ///
    DisplayDebug {
        group_id: Option<ItemId>,
        position: Option<u32>,
        color: Option<(u8, u8, u8)>,
        highlight: Option<(u8, u8, u8)>,
        highlight_state: Option<(ItemId, ItemId)>,
        spotlight: Option<u32>,
    },

    /// A variant for items which are to be displayed as a label in the control
    /// area (not as an event triggerable by the user). The text color of the
    /// label will match the rgb value if specified. The text will use the highlight
    /// color for any special animations and when the highlight_state status
    /// is in the listed state (if specified). If spotlight is set,
    /// the text will flash the highlight color the specified number of times
    /// when changed to this state or when this button is present but has not
    /// yet been pressed. A value of zero implies indefinite flashing.
    ///
    LabelControl {
        position: Option<u32>,
        color: Option<(u8, u8, u8)>,
        highlight: Option<(u8, u8, u8)>,
        highlight_state: Option<(ItemId, ItemId)>,
        spotlight: Option<u32>,
    },

    /// A variant for items which are only to be displayed as a label (not as an
    /// event triggerable by the user). The text color of the label will match
    /// the rgb value, if specified. The text will use the highlight
    /// color for any special animations and when the highlight_state status
    /// is in the listed state (if specified). If spotlight is set,
    /// the text will flash the highlight color the specified number of times
    /// when changed to this state or when this button is present but has not
    /// yet been pressed. A value of zero implies indefinite flashing.
    ///
    LabelHidden {
        position: Option<u32>,
        color: Option<(u8, u8, u8)>,
        highlight: Option<(u8, u8, u8)>,
        highlight_state: Option<(ItemId, ItemId)>,
        spotlight: Option<u32>,
    },

    /// Items which should not be displayed. Typically this includes items
    /// internal to the system or not designed to be directly accessible to the
    /// user. If this item is a label, it will be given default position
    /// (std::u32::MAX) and text color.
    ///
    Hidden,
}

// Reexport the display type variants
pub use self::DisplayType::{
    DisplayControl, DisplayDebug, DisplayWith, Hidden, LabelControl, LabelHidden,
};

/// This structure is a generic description to be paired with the ItemId
/// identifier. This scruct is a simple wrapper for String.
///
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ItemDescription {
    pub description: String,  // a wrapper for a String object
    pub display: DisplayType, // the display type for the item, ignored for types other than events
}

// Implement key ItemDescription struct features
impl ItemDescription {
    // A function to create an item id from &str and a DisplayType
    pub fn new(description: &str, display: DisplayType) -> ItemDescription {
        ItemDescription {
            description: description.to_string(),
            display,
        }
    }
}

// Implement displaying that shows the description
impl fmt::Display for ItemDescription {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

/// This structure is a generic identifier pair to combine the ItemId and
/// ItemDescription structs. This pair simply pulls these two fields for easy
/// serialization for the program configuration. This structure is also useful
/// for passing the id and description pairs to the user interface.
///
#[derive(Clone, Eq, Debug, Serialize, Deserialize)]
pub struct ItemPair {
    id: u32, // protected, as it may be inverted
    pub description: String,
    pub display: DisplayType,
}

// Implement key ItemPair features
impl ItemPair {
    /// A function to create an item pair from u32, CAN version.
    ///
    /// When program is built with CAN checking ON, this function will cap valid
    /// IDs at 29 bits (i.e. 29^2 or over half a million IDs). If you need
    /// additional IDs (exceptionally rare) and are not using a CAN bus, you can
    /// disable this checking by rebuilding the program with the no_can_limit
    /// feature (see below).
    ///
    /// # Disabling CAN Checking
    ///
    /// If this documentation was provided with the program or was autogenerated
    ///
    /// ```
    /// cargo doc --open
    /// ```
    /// then your program was likely compiled with CAN checking ON. Rebuild your
    /// program without CAN checking with
    ///
    /// ```
    /// cargo build --features no_can_limit
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// let item = ItemPair::new(1, "Example Description", DisplayType::Hidden);
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return None if the address exceeds the 29-bit
    /// address limit or conflicts with the ALL_STOP id and Some(ItemPair)
    /// otherwise.
    ///
    #[cfg(not(feature = "no_can_limit"))]
    pub fn new(id: u32, description: &str, display: DisplayType) -> Option<ItemPair> {
        // Verify id does not conflict with all stop
        if id == ALL_STOP {
            return None;
        }

        // Verify 29 bit limit for CAN bus
        if id >= 0x1FFFFFFF {
            return None;
        }

        // Return the new item pair
        Some(ItemPair {
            id,
            description: description.to_string(),
            display,
        })
    }

    /// A function to create an item pair from u32, no CAN version.
    ///
    /// When program is built with CAN checking OFF, this function will not
    /// check IDs validity. If you are running a this system on a CAN bus,
    /// it is highly recommended that you turn CAN checking ON (see below).
    ///
    /// # Enabling CAN Checking
    ///
    /// If this documentation was provided with the program or was autogenerated
    ///
    /// ```
    /// cargo doc --open
    /// ```
    /// then your program was likely compiled with CAN checking OFF. Rebuild
    /// your program with CAN checking with
    ///
    /// ```
    /// cargo build
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// let item = ItemId::new(0xFFFFFFFF, "Example Description", DisplayType::Hidden);
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return None if the address conflicts with the
    /// ALL_STOP id and Some(ItemPair) otherwise.
    ///
    #[cfg(feature = "no_can_limit")]
    pub fn new(id: u32, description: &str, display: DisplayType) -> ItemPair {
        // Verify id does not conflict with all stop
        if id == ALL_STOP {
            return None;
        }

        // Return the new item pair
        Some(ItemPair {
            id,
            description: description.to_string(),
            display,
        })
    }

    /// A function to create an item id from u32, unchecked version.
    ///
    /// This function does not verify that the new ItemPair complies with the
    /// CAN limit and does not check that the new ItemId does not collide with
    /// the all stop item id. This is useful when either (or both) of these
    /// cases are possible and desired.
    ///
    pub fn new_unchecked(id: u32, description: &str, display: DisplayType) -> ItemPair {
        ItemPair {
            id,
            description: description.to_string(),
            display,
        }
    }

    /// A function to create an item pair from ItemId and ItemDescription.
    ///
    /// This function takes a valid ItemId and ItemDescription and combines them
    /// into a single ItemPair. Both inputs are consumed in the creation of the
    /// ItemPair.
    ///
    pub fn from_item(id: ItemId, description: ItemDescription) -> ItemPair {
        ItemPair {
            id: id.id,
            description: description.description,
            display: description.display,
        }
    }

    /// A method to produce a new u32 from the ItemPair id. The internal
    /// representation is protected in case inverted ids are made in the future.
    ///
    pub fn id(&self) -> u32 {
        self.id
    }

    /// A method to produce a new ItemID from the ItemPair.
    ///
    /// This method returns a valid ItemId from this item pair.
    ///
    pub fn get_id(&self) -> ItemId {
        ItemId { id: self.id }
    }

    /// A method to produce a new string from the ItemDescription in ItemPair.
    ///
    pub fn description(&self) -> String {
        self.description.clone()
    }

    /// A method to produce a new ItemDescription from the ItemPair.
    ///
    /// This method returns a valid ItemDescription from this item pair.
    ///
    pub fn get_description(&self) -> ItemDescription {
        ItemDescription {
            description: self.description.clone(),
            display: self.display.clone(),
        }
    }

    /// A method to verify full equality of the Item Pairs, including the
    /// description and the display type.
    ///
    /// This method compares the description as well as the id of the item pairs
    /// (as contrasted with the == operator which will just compare the ids).
    ///
    pub fn truly_equal(&self, other_id: &ItemPair) -> bool {
        self == other_id
            && self.description == other_id.description
            && self.display == other_id.display
    }

    /// A function to return a new all stop item pair. This is a reserved id for
    /// halting all active processes on the system and returning it to normal.
    ///
    pub fn all_stop() -> ItemPair {
        ItemPair {
            id: ALL_STOP,
            description: "ALL STOP".to_string(),
            display: Hidden,
        }
    }
}

// Implement hashing which ignores the description
impl hash::Hash for ItemPair {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

// Implement partialEq which ignores the description
/// The default equality operation will only compare the ids of the ItemPair.
/// Use ItemPair::truely_equal() to compare the item descriptions as well.
///
impl PartialEq for ItemPair {
    fn eq(&self, rhs: &ItemPair) -> bool {
        self.id == rhs.id
    }
}

// Implement Ord which ignores the description
impl Ord for ItemPair {
    fn cmp(&self, other: &ItemPair) -> Ordering {
        self.id.cmp(&other.id)
    }
}

// Implement partialOrd which ignores the description, based off cmp
impl PartialOrd for ItemPair {
    fn partial_cmp(&self, other: &ItemPair) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Implement displaying that shows both ID and description, but not displaytype
impl fmt::Display for ItemPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.description, self.id)
    }
}

// Tests of the item module
#[cfg(test)]
mod tests {
    use super::*;

    // Test id with CAN bus limiter
    #[test]
    #[cfg_attr(not(feature = "no_can_limit"), should_panic)]
    fn create_id() {
        // Try to create an id out of CAN range
        let _e = ItemId::new(0xFFFFFFFF).unwrap();
    }

    // Test comparison of simple item pairs
    #[test]
    fn compare_events() {
        // Create several events
        let event = ItemPair::new(1, "One Event", Hidden).unwrap();
        let same_event = event.clone();
        let different_description = ItemPair::new(1, "Different Description", Hidden).unwrap();
        let different_event = ItemPair::new(2, "Two Event", Hidden).unwrap();

        // Compare the events
        assert_eq!(event, same_event);
        assert_eq!(event, different_description);
        assert!(!(event.truly_equal(&different_description)));
        assert_ne!(event, different_event);
        assert_ne!(same_event, different_event);
    }
}
