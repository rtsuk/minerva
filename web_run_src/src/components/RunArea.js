import React from 'react';
import { ItemButton } from './Buttons';
import { SceneMenu } from './Menus';
import { vh, vw } from './Functions';

// A box to contain the draggable edit area
export class ViewArea extends React.PureComponent {
  // Class constructor
  constructor(props) {
    // Collect props
    super(props);

    // Set initial state
    this.state = {
      cursorX: 0, // starting point of the cursor
      cursorY: 0,
    }

    // Bind the various functions
    this.handleMouseDown = this.handleMouseDown.bind(this);
  }
  
  // Function to respond to clicking the area
  handleMouseDown() {
    // Hide the menu
    this.setState({
      isMenuVisible: false,
      focusId: -1,
    });
  }
  
  // Render the edit area inside the viewbox
  render() {
    return (
      <>
        <SceneMenu value={this.props.currentScene.id} saveModifications={this.props.saveModifications} />
        <div className="viewArea" onMouseDown={this.handleMouseDown}>
          {this.state.sceneId !== -1 && <>
            <RunArea currentScene={this.props.currentScene} currentItems={this.props.currentItems} />
          </>}
        </div>
      </>
    );
  }
}

// The draggable run area
export class RunArea extends React.PureComponent {
  // Class constructor
  constructor(props) {
    // Collect props
    super(props);

    // Set initial state
    this.state = {
      left: 0, // horizontal offset of the area
      top: 0, // vertical offest of the area
      zoom: 1, // zoom scaling for the edit window
      cursorX: 0, // starting point of the cursor
      cursorY: 0,
    }

    // Bind the various functions
    this.handleMouseDown = this.handleMouseDown.bind(this);
    this.handleMouseMove = this.handleMouseMove.bind(this);
    this.handleMouseClose = this.handleMouseClose.bind(this);
    this.handleWheel = this.handleWheel.bind(this);
  }

  // Function to respond to clicking the area
  handleMouseDown(e) {
    // Prevent any other event handlers
    e = e || window.event;
    e.preventDefault();
   
    // Connect the mouse event handlers to the document
    document.onmousemove = this.handleMouseMove;
    document.onmouseup = this.handleMouseClose;

    // Save the cursor position, deselect any focus
    this.setState({
      cursorX: e.clientX,
      cursorY: e.clientY,
    });
  }

  // Function to respond to dragging the area
  handleMouseMove(e) {
    // Prevent the default event handler
    e = e || window.event;
    e.preventDefault();

    // Update the state
    this.setState((state) => {
      // Calculate change from old cursor position
      let changeX = state.cursorX - e.clientX;
      let changeY = state.cursorY - e.clientY;
  
      // Calculate the new location
      let left = state.left - changeX;
      let top = state.top - changeY;
  
      // Enforce bounds on the new location
      left = (left <= 0) ? left : 0;
      top = (top <= 0) ? top : 0;
  
      // Save the new location and current cursor position
      return {
        left: left,
        top: top,
        cursorX: e.clientX,
        cursorY: e.clientY,
      }
    });
  }
  
  // Function to respond to releasing the mouse
  handleMouseClose() {
    // Stop moving when mouse button is released
    document.onmousemove = null;
    document.onmouseup = null;
  }

  // Function to respond to wheel events
  handleWheel(event) {
    // Zoom out
    if (event.deltaY > 0) {
      this.setState(prevState => {
        // Decrement the zoom
        let newZoom = prevState.zoom - (event.deltaY / 5000);
        
        // Check bounds
        if (newZoom < 0.5) {
          newZoom = 0.5;
        }

        // Update
        return ({
          zoom: newZoom,
        })
      });
    
    // Zoom in
    } else {
      this.setState(prevState => {
        // Decrement the zoom
        let newZoom = prevState.zoom - (event.deltaY / 5000);
        
        // Check bounds
        if (newZoom > 1) {
          newZoom = 1;
        }

        // Update
        return ({
          zoom: newZoom,
        })
      });
    }
  }

  // Render the draggable edit area
  render() {
    // Create a box for each event
    const boxes = this.props.currentItems.map((item, index) => <ItemButton key={item.id.toString()} id={item.id} row={parseInt(index / 12)} zoom={this.state.zoom} grabFocus={this.props.grabFocus} changeScene={this.props.changeScene} saveModifications={this.props.saveModifications} saveLocation={this.props.saveLocation} removeItem={this.props.removeItem} createConnector={this.props.createConnector} />);
    
    // Render the event boxes
    return (
      <div id={`scene-${this.props.id}`} className="editArea" style={{ left: `calc(${this.state.left}px - (250% * ${1 - this.state.zoom}))`, top: `calc(${this.state.top}px - (250% * ${1 - this.state.zoom}))`, transform: `scale(${this.state.zoom})` }} onMouseDown={this.handleMouseDown} onWheel={this.handleWheel}>
        {boxes}
      </div>
    )
  }
}
