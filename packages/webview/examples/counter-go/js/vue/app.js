var vm = new Vue({
  el: '#app',
  template: '<div><div class="counter">{{c.data.value}}</div><button v-on:click="incr">Incr</button></div>',
  data: {c: counter},
  methods: {
    incr: function() { counter.add(1); },
  },
});
