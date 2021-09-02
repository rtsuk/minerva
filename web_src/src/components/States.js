import React from 'react';
import { stopPropogation } from './functions';
import { SendNode } from './Nodes';

// A State list element
export class State extends React.PureComponent {
  // Class constructor
  constructor(props) {
    // Collect props
    super(props);

    // Set initial state
    this.state = {
      description: "Loading ...",
    }

    // Bind the various functions
    this.updateItem = this.updateItem.bind(this);
  }

  // Helper function to update the item information
  async updateItem() {
    try {
      // Fetch the description of the status
      let response = await fetch(`getItem/${this.props.state.id}`);
      const json = await response.json();

      // If valid, save the result to the state
      if (json.item.isValid) {
        this.setState({
          description: json.item.itemPair.description,
        });
      }
    
    // Ignore errors
    } catch {
      console.log("Server inaccessible.");
    }
  }

  // On initial load, update descriptions
  componentDidMount() {
    this.updateItem();
  }

  // On change of item id, update descriptions
  componentDidUpdate(prevProps, prevState) {
    // Update the item description, if it changed
    if (this.props.state.id !== prevProps.state.id) {
      this.updateItem();
    }
  }

  // Render the completed action
  render() {
    return (
      <>
        <div className="state">
          <div className="deleteState" onClick={(e) => {stopPropogation(e); this.props.removeState()}}>X</div>
          {this.state.description}
          <SendNode type="event" onMouseDown={(e) => {stopPropogation(e); this.props.grabFocus(this.props.state.id)}}/>
        </div>
      </>
    );
  }
}

// An Unmodifiable State list element
export class UnmodifiableState extends React.PureComponent {
  // Class constructor
  constructor(props) {
    // Collect props
    super(props);

    // Set initial state
    this.state = {
      description: "Loading ...",
    }

    // Bind the various functions
    this.updateItem = this.updateItem.bind(this);
  }

  // Helper function to update the item information
  async updateItem() {
    try {
      // Fetch the description of the status
      let response = await fetch(`getItem/${this.props.state.id}`);
      const json = await response.json();

      // If valid, save the result to the state
      if (json.item.isValid) {
        this.setState({
          description: json.item.itemPair.description,
        });
      }
    
    // Ignore errors
    } catch {
      console.log("Server inaccessible.");
    }
  }

  // On initial load, update descriptions
  componentDidMount() {
    this.updateItem();
  }

  // On change of item id, update descriptions
  componentDidUpdate(prevProps, prevState) {
    // Update the item description, if it changed
    if (this.props.state.id !== prevProps.state.id) {
      this.updateItem();
    }
  }

  // Render the completed action
  render() {
    return (
      <>
        <div className="state">
          {this.state.description}
          <SendNode type="event" onMouseDown={(e) => {stopPropogation(e); this.props.grabFocus(this.props.state.id)}}/>
        </div>
      </>
    );
  }
}
