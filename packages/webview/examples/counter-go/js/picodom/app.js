var h = picodom.h

function UI(c) {
	return h('div', null,
					 h('div', null, c.data.value),
					 h('button', {onclick: function(){c.add(1)}}, 'Incr'));
}

var node;
function render() {
	node = picodom.patch(node, node=UI(counter))
}
counter.render = render;
render();
