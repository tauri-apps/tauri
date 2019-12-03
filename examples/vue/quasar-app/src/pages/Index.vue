<template>
  <q-page class="flex flex-center">
    <h1>{{ msg }}</h1>
    <q-btn @click="eventToRust()">SEND MSG</q-btn>
  </q-page>
</template>

<script>
import Tauri from '../../src-tauri/tauri.js'
window.tauri = Tauri

export default {
  name: 'HelloWorld',
  data () {
    return {
      msg: 'waiting for rust'
    }
  },
  mounted () {
    window.tauri.setup()
    window.tauri.addEventListener('reply', res => {
      this.msg = res.payload.msg
    })
  },
  methods: {
    // set up an event listener
    eventToRust () {
      window.tauri.emit('hello', 'passthrough message')
    }
  }
}
</script>
