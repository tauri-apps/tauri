/** @jsx preact.h */

// A simple click counter component, can be a functional compontent as well
class Counter extends preact.Component {
	constructor(props) {
		super(props)
	}
  render() {
    return (
      <div>
        <div>count:{counter.data.value}</div>
				<button onClick={() => counter.add(1)}>click!</button>
      </div>
    )
  }
}

// Render top-level component, pass controller data as props
const render = () =>
	preact.render(<Counter />, document.getElementById('app'), document.getElementById('app').lastElementChild);

// Call global render() when controller changes
counter.render = render;

// Render of the first time
render();
